use kernel::prelude::*;
use kernel::bindings::{ proc_mkdir, proc_create, copy_to_user, copy_from_user, proc_ops, proc_dir_entry, loff_t };
use kernel::bindings::file as c_file;
use core::ptr::null_mut;
use core::ffi::{ c_char, c_void };

module! {
    type: ProcFsTest,
    name: "rust_procfs_test",
    license: "GPL",
}

struct ProcFsTest;

// struct ProcFsTest {
//     fops: proc_ops,
//     proc_folder: *mut proc_dir_entry,
//     proc_file: *mut proc_dir_entry,
// };

const fops: proc_ops = proc_ops {
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
    let text = "Hello from a procfs file!\n";
    let to_copy = core::cmp::min(count as usize, text.len());
    let not_copied;

    unsafe {
        not_copied = copy_to_user(user_buffer as *mut c_void, text.as_ptr() as *const _, to_copy.try_into().unwrap());
    }

    let delta = to_copy - not_copied as usize;
    delta.try_into().unwrap()
}

unsafe extern "C" fn my_write(_file: *mut c_file, user_buffer: *const c_char, count: usize, offs: *mut loff_t) -> isize {
    let mut text = [0u8, 255];
    let to_copy = core::cmp::min(count as usize, text.len());
    let not_copied;
    
    unsafe {
        not_copied = copy_from_user(text.as_ptr() as *mut c_void, user_buffer as *mut c_void, to_copy as u64);
        pr_info!("procfs_test - You have written {} to me\n", core::str::from_utf8_unchecked(&text));
    }

    let delta = to_copy - not_copied as usize;
    delta.try_into().unwrap()
}

impl kernel::Module for ProcFsTest {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello ProcFsTest!\n");
        
        let proc_folder: *mut proc_dir_entry;
        let proc_file: *mut proc_dir_entry;

        let folder_name: *const c_char = b"hello\0".as_ptr() as *const i8;
        let file_name: *const c_char = b"dummy\0".as_ptr() as *const i8;

        unsafe {
            proc_folder = proc_mkdir(folder_name , null_mut());
        } 
        if proc_folder == null_mut() {
            pr_info!("proc_folder is null\n");
        }
        pr_info!("proc_folder: {:?}\n", proc_folder);

    
        unsafe {
            proc_file = proc_create(file_name, 0o644, proc_folder, &fops);
        }
        if proc_folder == null_mut() {
            pr_info!("proc_file is null\n");
        }
        pr_info!("proc_file: {:?}\n", proc_file);

        Ok(Self)
    }
}