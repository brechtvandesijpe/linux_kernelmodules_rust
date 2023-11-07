use kernel::prelude::*;
use kernel::bindings::{proc_mkdir, proc_create, file_operations, proc_ops};
use kernel::c_str;

module! {
    type: ProcFsTest,
    name: "rust_procfs_test",
    license: "GPL",
}

struct ProcFsTest;

struct ProcFile {
    proc_folder: *mut kernel::bindings::proc_dir_entry,
    proc_file: *mut kernel::bindings::proc_dir_entry,
}

impl ProcFile {
    fn new() -> Result<Self> {
        let proc_folder = unsafe { proc_mkdir(c_str!("hello").as_ptr(), core::ptr::null_mut()); };
        let proc_file = unsafe { proc_create(c_str!("dummy").as_ptr(), 0o644, proc_folder, &mut FILE_OPERATIONS); };
        Ok(Self {
            proc_folder,
            proc_file,
        })
    }

    fn read(&self) {
    
    }
    
    fn write(&self) {
    
    }
}

unsafe extern "C" fn read_callback() {
    // Get a reference to the ProcFile instance and call its read method
    // You'll need to store the ProcFile instance somewhere accessible from this function
}

unsafe extern "C" fn write_callback() {
    // Get a reference to the ProcFile instance and call its write method
    // You'll need to store the ProcFile instance somewhere accessible from this function
}

static mut FILE_OPERATIONS: kernel::bindings::proc_ops = kernel::bindings::proc_ops {
    proc_read: Some(read_callback),
    proc_write: Some(write_callback),
};

impl kernel::Module for ProcFsTest {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello ProcFsTest!\n");
        let _proc_file = ProcFile::new();
        Ok(Self)
    }
}