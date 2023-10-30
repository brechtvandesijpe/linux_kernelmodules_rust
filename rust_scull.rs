use kernel::prelude::*;

module! {
    type: Scull,
    name: "scull",
    license: "GPL",
}

struct Scull;

impl kernel::Module for Scull {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello scull!");
        Ok(Self)
    }
}