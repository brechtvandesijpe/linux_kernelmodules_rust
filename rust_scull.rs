use kernel::io_buffer::{ IoBufferReader, IoBufferWriter };
use kernel::prelude::*;
use kernel::{ file, miscdev };
use kernel::sync::{Arc, ArcBorrow};

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
}

#[vtable]
impl file::Operations for Scull {
    type OpenData = Arc<Device>;

    fn open(context: &Arc<Device>, _file: &file::File) -> Result<Self::Data> {
        pr_info!("File was opened!\n");
        Ok(())
    }

    fn read(
        _data: (),
        _file: &file::File,
        _writer: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("File was read\n");
        Ok(0)
    }    

    fn write(
        _data: (),
        _file: &file::File,
        reader: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        pr_info!("File was written\n");
        Ok(reader.len())
    }
}

impl kernel::Module for Scull {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello scull!\n");

        let dev = Arc::try_new(Device { number: 0})?;
        let reg = miscdev::Registration::new_pinned(fmt!("scull"), dev)?;
        
        Ok(Self { _dev: reg })
    }
}