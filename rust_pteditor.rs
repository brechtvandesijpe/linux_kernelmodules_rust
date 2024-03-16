use kernel::prelude::*;
use kernel::io_buffer::{IoBufferReader, IoBufferWriter};
use kernel::sync::{smutex::Mutex, Arc, ArcBorrow};
use kernel::miscdev;
use kernel::fs::Registration;
use kernel::error::code;
use kernel::file;
use core::sync::atomic::{AtomicBool, Ordering};
use core::ptr;

use kernel::bindings::mm_struct;
use kernel::bindings::task_struct;
use kernel::bindings::pid;
use kernel::bindings::rw_semaphore;


module! {
    type: PtEditor,
    name: "Pt_Editor",
    license: "GPL",
}


// structs:
#[repr(C)]
pub struct ptedit_entry_t {
    pid: usize,
    vaddr: usize,

    pgd: PgdPml5Union,
    p4d: P4dPml4Union,
    pud: PudPdptUnion,
    pmd: PmdPdUnion,

    pte: usize,
    valid: usize,
}

#[repr(C)]
pub enum PgdPml5Union {
    Pgd(usize),
    Pml5(usize),
}

#[repr(C)]
pub enum P4dPml4Union {
    P4d(usize),
    Pml4(usize),
}

#[repr(C)]
pub enum PudPdptUnion {
    Pud(usize),
    Pdpt(usize),
}

#[repr(C)]
pub enum PmdPdUnion {
    Pmd(usize),
    Pd(usize),
}

impl ptedit_entry_t {
    pub fn new() -> ptedit_entry_t {
        ptedit_entry_t {
            pid: 0,
            vaddr: 0,
            pgd: PgdPml5Union::Pgd(0),
            p4d: P4dPml4Union::P4d(0),
            pud: PudPdptUnion::Pud(0),
            pmd: PmdPdUnion::Pmd(0),
            pte: 0,
            valid: 0,
        }
    }
}

#[repr(C)]
pub struct vm_t {
    pid: usize,
    pgd: *mut kernel::bindings::pgd_t,
    p4d: *mut kernel::bindings::p4d_t,
    pud: *mut kernel::bindings::pud_t,
    pmd: *mut kernel::bindings::pmd_t,
    pte: *mut kernel::bindings::pte_t,
    valid: usize,
}

impl vm_t {
    // Constructor function
    pub fn new() -> vm_t {
        vm_t {
            pid: 0,
            pgd: ptr::null_mut(),
            p4d: ptr::null_mut(),
            pud: ptr::null_mut(),
            pmd: ptr::null_mut(),
            pte: ptr::null_mut(),
            valid: 0,
        }
    }
}



// globals:
static DEVICE_BUSY: AtomicBool = AtomicBool::new(false);
static MM_IS_LOCKED: AtomicBool = AtomicBool::new(false);

const PTEDIT_VALID_MASK_PGD: usize = 1 << 0;
const PTEDIT_VALID_MASK_P4D: usize = 1 << 1;
const PTEDIT_VALID_MASK_PUD: usize = 1 << 2;
const PTEDIT_VALID_MASK_PMD: usize = 1 << 3;
const PTEDIT_VALID_MASK_PTE: usize = 1 << 4;

struct Device {
    number: usize,
    contents: Mutex<Vec<u8>>,
}

struct PtEditor {
    _dev: Pin<Box<miscdev::Registration<PtEditor>>>,
}



// hulpfunctions
fn pte_unmap(_pte: *mut kernel::bindings::pte_t ) {
    
}

fn get_mm(pid: usize) -> *mut mm_struct { 
    let mut task: *mut task_struct = &mut task_struct::default() as *mut task_struct;
    let mut vpid: *mut pid = &mut pid::default() as *mut pid;
    
    task = unsafe {kernel::bindings::get_current()};
    if pid != 0 {
        vpid = unsafe {kernel::bindings::find_vpid(pid.try_into().unwrap())};
        if vpid.is_null() {
            return ptr::null_mut();
        }
        task = unsafe {kernel::bindings::pid_task(vpid, kernel::bindings::pid_type_PIDTYPE_PID)};
        if vpid.is_null() {
            return ptr::null_mut();
        }
    }
    unsafe {
        if !((*task).mm).is_null() {
            return (*task).mm;
        }
        else {
            return (*task).active_mm;
        }
    }
}



fn resolve_vm(addr: usize, entry: *mut vm_t, lock: bool) -> i32 {
    let mut mm: *mut mm_struct = &mut mm_struct::default() as *mut mm_struct;

    if entry.is_null() {
        return 1;
    }
    unsafe {
        (*entry).pud = ptr::null_mut();
        (*entry).pmd = ptr::null_mut();
        (*entry).pgd = ptr::null_mut();
        (*entry).pte = ptr::null_mut();
        (*entry).p4d = ptr::null_mut();
        (*entry).valid = 0;
    }

    mm = get_mm(unsafe {(*entry).pid});

    /* Lock mm */
    if lock {
        let mmap_sem: *mut rw_semaphore = unsafe {&mut (*mm).__bindgen_anon_1.mmap_lock as *mut rw_semaphore};
        unsafe {kernel::bindings::down_read(mmap_sem)};
    }

    /* Return PGD (page global directory) entry */
    unsafe {
        (*entry).pgd = kernel::bindings::pgd_offset_pgd((*mm).__bindgen_anon_1.pgd , addr.try_into().unwrap());
        let none = kernel::bindings::pgd_none(*(*entry).pgd);
        let bad = kernel::bindings::pgd_bad(*(*entry).pgd);
        if none != 0 || bad != 0 {
            (*entry).pgd = ptr::null_mut();
            return 1;
        }
        (*entry).valid |= PTEDIT_VALID_MASK_PGD;
    }

    /* Return p4d offset */
    unsafe {
        (*entry).p4d = kernel::bindings::p4d_offset((*entry).pgd , addr.try_into().unwrap());
        let none = kernel::bindings::p4d_none(*(*entry).p4d);
        let bad = kernel::bindings::p4d_bad(*(*entry).p4d);
        if none != 0 || bad != 0 {
            (*entry).p4d = ptr::null_mut();
            return 1;
        }
        (*entry).valid |= PTEDIT_VALID_MASK_P4D;
    }

    /* Get offset of PUD (page upper directory) */
    unsafe {
        (*entry).pud = kernel::bindings::pud_offset((*entry).p4d , addr.try_into().unwrap());
        let none = kernel::bindings::pud_none(*(*entry).pud);
        if none != 0 {
            (*entry).pud = ptr::null_mut();
            return 1;
        }
        (*entry).valid |= PTEDIT_VALID_MASK_PUD;
    }

    /* Get offset of PMD (page middle directory) */
    unsafe {
        (*entry).pmd = kernel::bindings::pmd_offset((*entry).pud , addr.try_into().unwrap());
        let none = kernel::bindings::pmd_none(*(*entry).pmd);
        let large = kernel::bindings::pud_large(*(*entry).pud);
        if none != 0 || large != 0 {
            (*entry).pmd = ptr::null_mut();
            return 1;
        }
        (*entry).valid |= PTEDIT_VALID_MASK_PMD;
    }

    /* Map PTE (page table entry) */
    unsafe {
        (*entry).pte = kernel::bindings::pte_offset_kernel((*entry).pmd , addr.try_into().unwrap());
        let large = kernel::bindings::pmd_large(*(*entry).pmd);
        if (*entry).pte.is_null() || large !=0 {
            (*entry).pte = ptr::null_mut();
            return 1;
        }
        (*entry).valid |= PTEDIT_VALID_MASK_PTE;
    }
    unsafe {pte_unmap((*entry).pte)};

    if lock {
        let mmap_sem: *mut rw_semaphore = unsafe {&mut (*mm).__bindgen_anon_1.mmap_lock as *mut rw_semaphore};
        unsafe {kernel::bindings::up_write(mmap_sem)};
    }

    0
}


fn vm_to_user(user: *mut ptedit_entry_t, vm: *mut vm_t) {
    unsafe {
        if !(*vm).p4d.is_null() {
            (*user).p4d = P4dPml4Union::P4d((*(*vm).p4d).p4d.try_into().unwrap());
        }
        if !(*vm).pgd.is_null() {
            (*user).pgd = PgdPml5Union::Pgd((*(*vm).pgd).pgd.try_into().unwrap());
        }
        if !(*vm).pmd.is_null() {
            (*user).pmd = PmdPdUnion::Pmd((*(*vm).pmd).pmd.try_into().unwrap());
        }
        if !(*vm).pud.is_null() {
            (*user).pud = PudPdptUnion::Pud((*(*vm).pud).pud.try_into().unwrap());
        }
        if !(*vm).pte.is_null() {
            (*user).pte = (*(*vm).pud).pud.try_into().unwrap();
        }
        (*user).valid = (*vm).valid;
    }
}


fn update_vm(new_entry: *mut ptedit_entry_t, lock: bool) -> i32 {
    let mut old_entry = vm_t::new();
    let addr = unsafe {(*new_entry).vaddr};
    let mut mm: *mut mm_struct = unsafe {get_mm((*new_entry).pid)};
    if mm.is_null() {
        return 1;
    }

    /* Lock mm */
    if lock {
        let mmap_sem: *mut rw_semaphore = unsafe {&mut (*mm).__bindgen_anon_1.mmap_lock as *mut rw_semaphore};
        unsafe {kernel::bindings::down_read(mmap_sem)};
    }

    if (old_entry.valid & PTEDIT_VALID_MASK_PGD) != 0 && (unsafe {(*new_entry).valid} & PTEDIT_VALID_MASK_PGD) != 0 {
        pr_warn!("Updating PGD\n");
        unsafe {
            let new_pgd_value = match (*new_entry).pgd {
                PgdPml5Union::Pgd(value) => value,
                PgdPml5Union::Pml5(value) => value,
            };
            kernel::bindings::native_set_pgd(old_entry.pgd, kernel::bindings::native_make_pgd(new_pgd_value.try_into().unwrap()));
        }
    }

    if (old_entry.valid & PTEDIT_VALID_MASK_P4D) != 0 && (unsafe {(*new_entry).valid} & PTEDIT_VALID_MASK_P4D) != 0 {
        pr_warn!("Updating P4D\n");
        unsafe {
            let new_p4d_value = match (*new_entry).p4d {
                P4dPml4Union::P4d(value) => value,
                P4dPml4Union::Pml4(value) => value,
            };
            kernel::bindings::native_set_p4d(old_entry.p4d, kernel::bindings::native_make_p4d(new_p4d_value.try_into().unwrap()));
        }
    }

    if (old_entry.valid & PTEDIT_VALID_MASK_PUD) != 0 && (unsafe {(*new_entry).valid} & PTEDIT_VALID_MASK_PUD) != 0 {
        pr_warn!("Updating PUD\n");
        unsafe {
            let new_pud_value = match (*new_entry).pud {
                PudPdptUnion::Pud(value) => value,
                PudPdptUnion::Pdpt(value) => value,
            };
            kernel::bindings::native_set_pud(old_entry.pud, kernel::bindings::native_make_pud(new_pud_value.try_into().unwrap()));
        }
    }

    if (old_entry.valid & PTEDIT_VALID_MASK_PMD) != 0 && (unsafe {(*new_entry).valid} & PTEDIT_VALID_MASK_PMD) != 0 {
        pr_warn!("Updating PMD\n");
        unsafe {
            let new_pmd_value = match (*new_entry).pmd {
                PmdPdUnion::Pmd(value) => value,
                PmdPdUnion::Pd(value) => value,
            };
            kernel::bindings::native_set_pmd(old_entry.pmd, kernel::bindings::native_make_pmd(new_pmd_value.try_into().unwrap()));
        }
    }

    if (old_entry.valid & PTEDIT_VALID_MASK_PTE) != 0 && (unsafe {(*new_entry).valid} & PTEDIT_VALID_MASK_PTE) != 0 {
        pr_warn!("Updating PTE\n");
        unsafe {
            kernel::bindings::native_set_pte(old_entry.pte, kernel::bindings::native_make_pte((*new_entry).pte.try_into().unwrap()));
        }
    }

    // invalidate_tlb(addr);

    if lock {
        let mmap_sem: *mut rw_semaphore = unsafe {&mut (*mm).__bindgen_anon_1.mmap_lock as *mut rw_semaphore};
        unsafe {kernel::bindings::up_write(mmap_sem)};
    }
    0
}



#[vtable]
impl file::Operations for PtEditor {
    type OpenData = Arc<Device>;
    type Data = Arc<Device>;

    fn open(context: &Arc<Device>, _file: &file::File) -> Result<Arc<Device>> {
        let is_device_busy = DEVICE_BUSY.load(Ordering::Relaxed);
        if is_device_busy {
            return Err(code::EBUSY);
        }
        DEVICE_BUSY.store(true, Ordering::Relaxed);

        Ok(context.clone())
    }


    fn release(_data_arg: Arc<Device>, _file: &file::File) {
        DEVICE_BUSY.store(false, Ordering::Relaxed);
    }


    fn ioctl(_data: ArcBorrow<'_, Device>, _file: &file::File, cmd: &mut file::IoctlCommand) -> Result<i32> {
        let (ioctl_num, mut ioctl_param) = cmd.raw();
        match ioctl_num {
            1 => {
                let mut vm_user = ptedit_entry_t::new();
                let mut vm = vm_t::new();
                let _ = wrapper_copy_from_user(&mut vm_user, ioctl_param as *const usize);
                vm.pid = vm_user.pid;
                let is_locked = MM_IS_LOCKED.load(Ordering::Relaxed);
                resolve_vm(vm_user.vaddr, &mut vm as *mut vm_t, !is_locked);
                vm_to_user(&mut vm_user as *mut ptedit_entry_t, &mut vm as *mut vm_t);
                let _ = wrapper_copy_to_user(&mut ioctl_param, &vm_user as *const ptedit_entry_t);
                
                Ok(0)
            },
            2 => {
                let mut vm_user = ptedit_entry_t::new();
                let _ = wrapper_copy_from_user(&mut vm_user, ioctl_param as *const usize);
                let is_locked = MM_IS_LOCKED.load(Ordering::Relaxed);
                update_vm(&mut vm_user as *mut ptedit_entry_t, !is_locked);
                Ok(0)
            },
            _ => {
                Ok(0)
            }
        }
    }


}

impl kernel::Module for PtEditor {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("PTEDITOR IN RUST\n\n\n");
        let dev = Arc::try_new(Device { 
            number: 0,
            contents: unsafe { Mutex::new(Vec::<u8>::new()) },
        })?;
        let reg = miscdev::Registration::new_pinned(fmt!("ptedit"), dev)?;
        Ok(Self { _dev: reg })
    } 
}



// wrappers:
fn wrapper_copy_from_user<T, U>(dst: &mut T, src: *const U) -> Result<i32> {
    let result = unsafe {
        kernel::bindings::_copy_from_user(
            dst as *mut T as *mut core::ffi::c_void,
            src as *const core::ffi::c_void,
            core::mem::size_of::<T>() as core::ffi::c_ulong,
        )
    };

    if result == 0 {
        Ok(0)
    } else {
        Err(code::EINVAL)
    }
}

fn wrapper_copy_to_user<T, U>(dst: &mut T, src: *const U) -> Result<i32> {
    let result = unsafe {
        kernel::bindings::_copy_to_user(
            dst as *mut T as *mut core::ffi::c_void,
            src as *const core::ffi::c_void,
            core::mem::size_of::<T>() as core::ffi::c_ulong,
        )
    };

    if result == 0 {
        Ok(0)
    } else {
        Err(code::EINVAL)
    }
}