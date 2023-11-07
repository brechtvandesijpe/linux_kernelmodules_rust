cmd_samples/rust/rust_vdev.ko := ld.lld -r -m elf_x86_64 --build-id=sha1  -T scripts/module.lds -o samples/rust/rust_vdev.ko samples/rust/rust_vdev.o samples/rust/rust_vdev.mod.o;  true
