cmd_samples/rust/built-in.a := rm -f samples/rust/built-in.a; echo rust_vdev.o | sed -E 's:([^ ]+):samples/rust/\1:g' | xargs llvm-ar cDPrST samples/rust/built-in.a
