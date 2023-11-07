cmd_samples/rust/rust_vdev.mod := printf '%s\n'   rust_vdev.o | awk '!x[$$0]++ { print("samples/rust/"$$0) }' > samples/rust/rust_vdev.mod
