use kernel::prelude::*;
use kernel::file;

module! {
    type: MiscDev,
    name: "rust_misc_dev",
    license: "GPL",
}

struct MiscDev;

impl file::Operations for MiscDev {
    fn open(context, file) -> Result<_, _> {
        Ok()
    }

    fn write() -> Result<_, _> {
        Ok()
    }

    fn read() -> Result<_, _> {
        Ok()
    }
}

impl kernel::Module for MiscDev {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello rust_misc_dev!");
        Ok(Self)
    }
}