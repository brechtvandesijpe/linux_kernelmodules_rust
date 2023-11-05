use kernel::prelude::*;

module! {
    type: ProcFsTest,
    name: "rust_procfs_test",
    license: "GPL",
}

struct ProcFsTest;

impl kernel::Module for ProcFsTest {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello ProcFsTest!\n");
        Ok(Self)
    }
}