use kernel::io_buffer::{ IoBufferReader, IoBufferWriter };
use kernel::prelude::*;
use kernel::{ file, miscdev };
use kernel::sync::{Arc, ArcBorrow, Mutex};

module! {
    type: Scull,
    name: "scull",
    license: "GPL",
}

struct Scull {
    _dev: Pin<Box<miscdev::Registration<Scull>>>,
}

struct Device {
    number: usize,
    contents: Mutex<Vec<u8>>,
}

#[vtable]
impl file::Operations for Scull {
    type OpenData = Arc<Device>;
    type Data = Arc<Device>;

    fn open(context: &Arc<Device>, _file: &file::File) -> Result<Arc<Device>> {
        pr_info!("File for device {} was opened\n", context.number);
        Ok(context.clone())
    }

    fn read(
        data: ArcBorrow<'_, Device>,
        _file: &file::File,
        writer: &mut impl IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        pr_info!("File for device {} was read\n", data.number);
        let vec = data.contents.lock();
        let offset = offset.try_into()?;
    }    

    fn write(
        data: ArcBorrow<'_, Device>,
        _file: &file::File,
        reader: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("File for device {} was written\n", data.number);

        let copy =  reader.read_all()?;
        let len = copy.len();

        let mut contents = data.contents.lock();
        *contents = copy;

        Ok(len)
    }
}

impl kernel::Module for Scull {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello scull!\n");

        let dev = Arc::try_new(Device { 
            number: 0,
            contents: unsafe { Mutex::new(Vec::<u8>::new()) },
        })?;

        let reg = miscdev::Registration::new_pinned(fmt!("scull"), dev)?;
        
        Ok(Self { _dev: reg })
    }
}