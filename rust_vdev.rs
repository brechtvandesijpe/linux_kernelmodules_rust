use kernel::prelude::*;
// use kernel::file;
// use kernel::file::Operations;

use kernel::bindings::{proc_mkdir, proc_dir_entry, proc_create, proc_ops, copy_to_user, loff_t, inode};
use kernel::bindings::file as c_file;
use core::ptr::{null, null_mut};
use core::ffi::{c_char, c_void, c_int};



module! {
    type: Jof,
    name: b"joffrey",
    license: b"GPL",
}

struct Jof {
    text: *const c_char,
}

// static buffer: *const c_char = b"y" as *const c_char; 



unsafe extern "C" fn proc_open_wrapper(_arg1: *mut inode, _file: *mut c_file) -> c_int {
    pr_info!("Open\n");
    let x: c_int = 0 as c_int;
    x

}

unsafe extern "C" fn proc_read_wrapper(_file: *mut c_file, user_buffer: *mut c_char, count: usize, offset: *mut loff_t) -> isize {
    unsafe {
        let start_offset: isize = *offset as isize;
        let text = "blabla\n";
        let text_len = text.len();

        if start_offset >= text_len as isize {
            return 0;
        }

        let to_copy = custom_min(count as u64, (text_len - start_offset as usize) as u64);

        *offset += to_copy as loff_t;

        // conversie om text te kunnen gebruiken in copy_to_user
        let c_text = text[start_offset as usize..].as_ptr() as *const c_char;
        let c_text_ptr = c_text as *mut c_void;

        let mut not_copied = 0;
        not_copied = copy_to_user(user_buffer as *mut c_void, c_text_ptr, to_copy);

        let delta = to_copy - not_copied as u64;
        delta as isize
    }

}


unsafe extern "C" fn proc_write_wrapper(_file: *mut c_file, user_buffer: *const c_char, count: usize, offset: *mut loff_t) -> isize {
    pr_info!("Write\n");
    let x: isize = 0 as isize;
    x

    // unsafe {
    //     let start_offset: isize = *offset as isize;
    //     let text: *const c_char = b'a' as *const c_char;
    //     pr_info!("Write\n");

        
    //     if start_offset >= count as isize {
    //         return 0;
    //     }


    //     let to_copy = custom_min(count as u64, (count - start_offset as usize) as u64);

    //     *offset += to_copy as loff_t;

    //     // let c_text = text[start_offset as usize..].as_ptr() as *const c_char;
    //     let c_text_ptr = text as *mut c_void;

    //     let mut not_copied = 0;
    //     not_copied = copy_to_user(c_text_ptr , user_buffer as *mut c_void, to_copy);
    //     pr_info!("not copied: {}", not_copied);

    //     let delta = to_copy - not_copied as u64;
    //     delta as isize
    // }



}

const fops: proc_ops = proc_ops {
    proc_flags: 0,
    proc_open: Some(proc_open_wrapper),
    proc_read: Some(proc_read_wrapper),
    proc_read_iter: None,
    proc_write: Some(proc_write_wrapper),
    proc_lseek: None,
    proc_release: None,
    proc_poll: None,
    proc_ioctl: None,
    proc_mmap: None,
    proc_get_unmapped_area: None,
};



impl kernel::Module for Jof {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello world!\n");
        pr_info!("JEEEEEEEEEEEEEEEEEEE\n\n");

        
        let proc_folder: *mut proc_dir_entry;
        let proc_file: *mut proc_dir_entry;
        

        let folder_name: c_char = b'A' as c_char;
        let file_name: c_char = b'B' as c_char;

        unsafe {
            proc_folder = proc_mkdir(&folder_name , null_mut());
        } 
        if proc_folder == null_mut() {
            pr_info!("proc_folder is null\n");
        }
        pr_info!("proc_folder: {:?}\n", proc_folder);

    
        unsafe {
            proc_file = proc_create(&file_name, 0o644, proc_folder, &fops);
        }
        if proc_folder == null_mut() {
            pr_info!("proc_file is null\n");
        }
        pr_info!("proc_file: {:?}\n", proc_file);
        
        Ok(Self {})
    } 
}



fn custom_min<T: Ord>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}