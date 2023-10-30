# Setup
git clone https://github.com/Rust-for-Linux/linux
cd linux
git fetch --depth=1 origin
git checkout origin/rust
rustup override set $(scripts/min-tool-version.sh rustc)
rustup component add rust-src
make allnoconfig qemu-busybox-min.config rust.config
make