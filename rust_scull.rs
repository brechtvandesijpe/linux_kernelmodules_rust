use kernel::prelude::*;
use kernel::file;

module! {
    type: Scull,
    name: "scull",
    license: "GPL",
}

struct Scull;

[#vtable]
impl file::Operations for Scull {
    fn open(_context: &(), _file: &file::file) -> Result {
        pr_info("File was opened!\n");
        Ok(())
    }
}

impl kernel::Module for Scull {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello scull!\n");
        Ok(Self)
    }
}