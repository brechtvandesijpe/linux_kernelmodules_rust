use kernel::prelude::*;
use kernel::bindings::{ proc_mkdir, proc_create, copy_to_user, copy_from_user, proc_ops, proc_dir_entry, loff_t };
use kernel::bindings::file as c_file;
use core::ptr::null_mut;
use kernel::sync::Mutex;
use core::ffi::{ c_char, c_void };

static GLOBAL_DATA: Mutex<Vec<u8>> = unsafe{ Mutex::new(Vec::new()) };

module! {
    type: ProcFsTest,
    name: "rust_procfs_test",
    license: "GPL",
}

struct ProcFsTest;

const FOPS: proc_ops = proc_ops {
    proc_flags: 0,
    proc_open: None,
    proc_read: Some(my_read),
    proc_read_iter: None,
    proc_write: Some(my_write),
    proc_lseek: None,
    proc_release: None,
    proc_poll: None,
    proc_ioctl: None,
    proc_mmap: None,
    proc_get_unmapped_area: None,
};

unsafe extern "C" fn my_read(_file: *mut c_file, user_buffer: *mut c_char, count: usize, offs: *mut loff_t) -> isize {
    let offset = unsafe { *offs as usize };
    let data = GLOBAL_DATA.lock();

    // If the offset is beyond the end of the string, return 0
    if offset >= data.len() {
        return 0;
    }

    // Otherwise, copy the remaining data to the user buffer
    let remaining = &data[offset..];
    let to_copy = core::cmp::min(count as usize, remaining.len());
    let not_copied;

    unsafe {
        not_copied = copy_to_user(user_buffer as *mut c_void, remaining.as_ptr() as *const _, to_copy.try_into().unwrap());
    }

    let delta = to_copy - not_copied as usize;
    unsafe { *offs += delta as i64 };
    delta.try_into().unwrap()
}

unsafe extern "C" fn my_write(_file: *mut c_file, user_buffer: *const c_char, count: usize, offs: *mut loff_t) -> isize {
    let mut buffer = [0u8, 255];
    let to_copy = core::cmp::min(count as usize, buffer.len());
    let not_copied;
    
    unsafe {
        not_copied = copy_from_user(buffer.as_mut_ptr() as *mut c_void, user_buffer as *const _, to_copy as u64);
        pr_info!("procfs_test - You have written {} to me\n", core::str::from_utf8_unchecked(&buffer));
    }

    let delta = to_copy - not_copied as usize;
    unsafe { *offs += delta as i64 };

    let mut data = GLOBAL_DATA.lock();
    data.try_extend_from_slice(&buffer[..delta]);

    delta.try_into().unwrap()
}

impl kernel::Module for ProcFsTest {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello ProcFsTest!\n");
        
        let proc_dir: ProcDir = ProcDir::new("hello").unwrap();
        let proc_file: ProcFile = ProcFile::new(proc_dir, "dummy").unwrap();

        Ok(Self)
    }
}

struct ProcDir {
    proc_folder: *mut proc_dir_entry,
}

impl ProcDir {
    fn new(name: &str) -> Result<Self> {
        let mut name_bytes = [0u8; 256]; // Adjust the size as needed
        for (i, byte) in name.bytes().enumerate() {
            name_bytes[i] = byte;
        }
        name_bytes[name.len()] = 0; // null termination
        let folder_name: *const c_char = name_bytes.as_ptr() as *const c_char;
        let mut folder: *mut proc_dir_entry;

        unsafe {
            folder = proc_mkdir(folder_name , null_mut());
        } 
        if folder == null_mut() {
            pr_info!("folder is null\n");
            // return Err(kernel::Error::from_kernel_errno(libc::EINVAL));
        }
        pr_info!("proc_folder: {:?}\n", folder);

        Ok(Self {
            proc_folder: folder
        })
    }
}

struct ProcFile {
    proc_file: *mut proc_dir_entry,
}

impl ProcFile {
    fn new(dir: ProcDir, name: &str) -> Result<Self> {
        let mut name_bytes = [0u8; 256]; // Adjust the size as needed
        for (i, byte) in name.bytes().enumerate() {
            name_bytes[i] = byte;
        }
        name_bytes[name.len()] = 0; // null termination
        let file_name: *const c_char = name_bytes.as_ptr() as *const c_char;
        let mut file: *mut proc_dir_entry;
        
        unsafe {
            file = proc_create(file_name, 0o644, dir.proc_folder, &FOPS);
        }
        if file == null_mut() {
            pr_info!("file is null\n");
            // return Err(kernel::Error::from_kernel_errno(libc::EINVAL));
        }
        pr_info!("proc_file: {:?}\n", file);

        Ok(Self {
            proc_file: file,
        })
    }
}