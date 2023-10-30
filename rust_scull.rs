use kernel::prelude::*;
use kernel::{file, miscdev};

module! {
    type: Scull,
    name: "scull",
    license: "GPL",
}

struct Scull {
    _dev: Pin<Box<miscdev::Registration<Scull>>>,
}

#[vtable]
impl file::Operations for Scull {
    fn open(_context: &Self::OpenData, _file: &file::File) -> Result {
        pr_info!("File was opened!\n");
        Ok(())
    }

    fn read()
}

impl kernel::Module for Scull {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello scull!\n");
        let reg = miscdev::Registration::<Scull>::new_pinned(fmt!("scull"), ())?;
        Ok(Self { _dev: reg })
    }
}