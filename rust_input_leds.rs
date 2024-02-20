use kernel::prelude::*;

module! {
    type: InputLedsModule,
    name: "input_leds",
    license: "GPL",
}

struct InputLedsModule;

#[cfg(feature = "config_vt")]
fn vt_trigger(name: &'static str) -> Option<&'static str> {
    Some(name)
}

#[cfg(not(feature = "config_vt"))]
fn vt_trigger(_name: &'static str) -> Option<&'static str> {
    None
}

fn get_input_led_info() -> [(Option<&'static str>, Option<&'static str>); kernel::bindings::LED_CNT as usize] {
    [
        (Some("numlock"), vt_trigger("kbd-numlock")),
        (Some("capslock"), vt_trigger("kbd-capslock")),
        (Some("scrolllock"), vt_trigger("kbd-scrolllock")),
        (Some("compose"), None),
        (Some("kana"), vt_trigger("kbd-kanalock")),
        (Some("sleep"), None),
        (Some("suspend"), None),
        (Some("mute"), None),
        (Some("misc"), None),
        (Some("mail"), None),
        (Some("charging"), None),
        (None, None),
        (None, None),
        (None, None),
        (None, None),
        (None, None),
    ]
}

#[derive(Copy, Clone)]
struct InputLed {
    cdev: kernel::bindings::led_classdev,
    handle: *mut kernel::bindings::input_handle,
    code: u32, // Assuming LED_* constants are u32
}

impl Default for InputLed {
    fn default() -> Self {
        Self {
            cdev: unsafe { core::mem::zeroed() },
            handle: core::ptr::null_mut(),
            code: 0,
        }
    }
}

struct InputLeds {
    handle: kernel::bindings::input_handle,
    num_leds: usize,
    leds: [InputLed; 10],
}

macro_rules! container_of {
    ($ptr:expr, $type:path, $field:ident) => {
        ($ptr as *const _ as usize - offset_of!($type, $field)) as *mut $type
    };
}

macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        unsafe {
            let instance = core::mem::MaybeUninit::<$ty>::uninit();
            let base_addr = instance.as_ptr() as usize;
            let field_addr = &(*instance.as_ptr()).$field as *const _ as usize;
            field_addr - base_addr
        }
    };
}

unsafe extern "C" fn input_leds_brightness_get(cdev: *mut kernel::bindings::led_classdev)
    -> kernel::bindings::led_brightness
{
    unsafe {
        let led = container_of!(cdev, InputLed, cdev);
        let input = (*(*led).handle).dev;

        if (*input).led[0] & (1 << (*led).code) != 0 {
            return (*cdev).max_brightness;
        }

        0
    }
}

unsafe extern "C" fn input_leds_brightness_set(cdev: *mut kernel::bindings::led_classdev,
    brightness: kernel::bindings::led_brightness) 
{
    let led = container_of!(cdev, InputLed, cdev);
    unsafe {
        kernel::bindings::input_inject_event((*led).handle, kernel::bindings::EV_LED, (*led).code, (brightness != 0) as i32);
    }
}

extern "C" fn input_leds_event(_handle: *mut kernel::bindings::input_handle, _type: u32, _code: u32, _value: i32) {}

unsafe extern "C" fn input_leds_get_count(dev: *mut kernel::bindings::input_dev) -> i32 {
    let mut count = 0;
    
    unsafe {
        for (index, led_code) in (*dev).ledbit.iter().filter(|&bit| *bit != 0).enumerate() {
            if get_input_led_info()[index].0.is_some() {
                count += 1;
            }
        }
    }

    count
}

// #[warn(improper_ctypes_definitions)]
extern "C" fn input_leds_connect(handler: *mut kernel::bindings::input_handler,
    dev: *mut kernel::bindings::input_dev, _id: *const kernel::bindings::input_device_id) -> i32 {
    unsafe {
        let num_leds: i32 = input_leds_get_count(dev).try_into().unwrap();
        if num_leds == 0 {
            return kernel::bindings::ENXIO.try_into().unwrap();
        }
        
        let mut leds = InputLeds {
            handle: kernel::bindings::input_handle {
                private: core::ptr::null_mut(),
                open: 0,
                name: b"leds\0".as_ptr() as *const i8,
                dev: dev, // assuming 'dev' is defined elsewhere
                handler: handler, // assuming 'handler' is defined elsewhere
                d_node: kernel::bindings::list_head { next: core::ptr::null_mut(), prev: core::ptr::null_mut() },
                h_node: kernel::bindings::list_head { next: core::ptr::null_mut(), prev: core::ptr::null_mut() },
            },
            num_leds: num_leds.try_into().unwrap(),
            leds: [InputLed::default(); 10],
        };

        let error = kernel::bindings::input_register_handle(&mut leds.handle as *mut _);
        if error != 0 {
            return error;
        }

        let error = kernel::bindings::input_open_device(&mut leds.handle as *mut _);
        if error != 0 {
            kernel::bindings::input_unregister_handle(&mut leds.handle as *mut _);
            return error;
        }

        for (index, &led_code) in (*dev).ledbit.iter().filter(|&bit| *bit != 0).enumerate() {
            if get_input_led_info()[index].0.is_none() {
                continue;
            }

            let mut buffer = [0u8; 50];  // Adjust the size as needed
            let dev_name = kernel::bindings::dev_name(&(*dev).dev as *const kernel::bindings::device);
            let led_info = get_input_led_info()[index].0.unwrap();
            // write!(&mut buffer, "{}::{}", dev_name, led_info).unwrap();

            static DEFAULT_TRIGGER: &'static str = "default_trigger\0";

            let default_trigger_str = get_input_led_info()[index].1;
            let default_trigger = if let Some(s) = default_trigger_str {
                if s == "default_trigger" {
                    DEFAULT_TRIGGER.as_ptr() as *const i8
                } else {
                    core::ptr::null()
                }
            } else {
                core::ptr::null()
            };

            let mut led = InputLed {
                handle: &mut leds.handle,
                code: led_code as u32,
                cdev: kernel::bindings::led_classdev {
                    name: buffer.as_ptr() as *const i8,
                    max_brightness: 1,
                    brightness_get: Some(input_leds_brightness_get as unsafe extern "C" fn(cdev: *mut kernel::bindings::led_classdev) -> kernel::bindings::led_brightness),
                    brightness_set: Some(input_leds_brightness_set as unsafe extern "C" fn(cdev: *mut kernel::bindings::led_classdev, brightness: kernel::bindings::led_brightness)),
                    default_trigger: default_trigger,
                    ..core::mem::zeroed()
                },
            };

            let error = kernel::bindings::led_classdev_register(&mut (*dev).dev as *mut _, &mut led.cdev);
            if error != 0 {
                // dev_err!(&dev.dev, "failed to register LED {}: {}\n", led.cdev.name, error);
                return error;
            }

            if leds.num_leds < leds.leds.len() {
                leds.leds[leds.num_leds] = led;
                leds.num_leds += 1;
            } else {
                return kernel::bindings::ENOMEM.try_into().unwrap();
            }
        }
        0   
    }        
}

extern "C" fn input_leds_disconnect(handle: *mut kernel::bindings::input_handle) {
    unsafe {
        let leds = (*handle).private as *mut InputLeds;

        for led in &mut (*leds).leds {
            kernel::bindings::led_classdev_unregister(&mut led.cdev);
        }

        kernel::bindings::input_close_device(handle);
        kernel::bindings::input_unregister_handle(handle);
    }
}

static INPUT_LEDS_IDS: [kernel::bindings::input_device_id; 2] = [
    kernel::bindings::input_device_id {
        flags: kernel::bindings::INPUT_DEVICE_ID_MATCH_EVBIT as u64,
        bustype: 0,
        vendor: 0,
        product: 0,
        version: 0,
        evbit: [1 << kernel::bindings::EV_LED; (kernel::bindings::INPUT_DEVICE_ID_EV_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        keybit: [0; (kernel::bindings::INPUT_DEVICE_ID_KEY_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        relbit: [0; (kernel::bindings::INPUT_DEVICE_ID_REL_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        absbit: [0; (kernel::bindings::INPUT_DEVICE_ID_ABS_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        mscbit: [0; (kernel::bindings::INPUT_DEVICE_ID_MSC_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        ledbit: [0; (kernel::bindings::INPUT_DEVICE_ID_LED_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        sndbit: [0; (kernel::bindings::INPUT_DEVICE_ID_SND_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        ffbit: [0; (kernel::bindings::INPUT_DEVICE_ID_FF_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        swbit: [0; (kernel::bindings::INPUT_DEVICE_ID_SW_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        propbit: [0; (kernel::bindings::INPUT_DEVICE_ID_PROP_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        driver_info: 0,
    },
    kernel::bindings::input_device_id {
        flags: 0,
        bustype: 0,
        vendor: 0,
        product: 0,
        version: 0,
        evbit: [0; (kernel::bindings::INPUT_DEVICE_ID_EV_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        keybit: [0; (kernel::bindings::INPUT_DEVICE_ID_KEY_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        relbit: [0; (kernel::bindings::INPUT_DEVICE_ID_REL_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        absbit: [0; (kernel::bindings::INPUT_DEVICE_ID_ABS_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        mscbit: [0; (kernel::bindings::INPUT_DEVICE_ID_MSC_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        ledbit: [0; (kernel::bindings::INPUT_DEVICE_ID_LED_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        sndbit: [0; (kernel::bindings::INPUT_DEVICE_ID_SND_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        ffbit: [0; (kernel::bindings::INPUT_DEVICE_ID_FF_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        swbit: [0; (kernel::bindings::INPUT_DEVICE_ID_SW_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        propbit: [0; (kernel::bindings::INPUT_DEVICE_ID_PROP_MAX / kernel::bindings::BITS_PER_LONG + 1) as usize],
        driver_info: 0,
    },
];

static mut INPUT_LEDS_HANDLER: kernel::bindings::input_handler = kernel::bindings::input_handler {
    private: core::ptr::null_mut(),
    event: Some(input_leds_event as unsafe extern "C" fn(*mut kernel::bindings::input_handle, u32, u32, i32)),
    events: None,
    filter: None,
    match_: None,
    connect: Some(input_leds_connect as unsafe extern "C" fn(*mut kernel::bindings::input_handler, *mut kernel::bindings::input_dev, *const kernel::bindings::input_device_id) -> i32),
    disconnect: Some(input_leds_disconnect as unsafe extern "C" fn(handle: *mut kernel::bindings::input_handle)),
    start: None,
    legacy_minors: false,
    minor: 0,
    name: b"leds\0".as_ptr() as *const i8,
    id_table: INPUT_LEDS_IDS.as_ptr(),
    h_list: kernel::bindings::list_head { next: core::ptr::null_mut(), prev: core::ptr::null_mut() },
    node: kernel::bindings::list_head { next: core::ptr::null_mut(), prev: core::ptr::null_mut() },
};

impl kernel::Module for InputLedsModule {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        unsafe {
            kernel::bindings::input_register_handler(&mut INPUT_LEDS_HANDLER);
        }
        Ok(Self {}) 
    }
}