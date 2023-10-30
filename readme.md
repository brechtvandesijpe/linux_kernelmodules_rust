# Setup
git clone https://github.com/Rust-for-Linux/linux

cd linux

git fetch --depth=1 origin

git checkout origin/rust

rustup override set $(scripts/min-tool-version.sh rustc)

rustup component add rust-src

make allnoconfig qemu-busybox-min.config rust.config

make

# Modify Kconfig and Makefile

make menuconfig

-> / and search the newly added module in Kconfig and Makefile

-> enable each menu you have to follow and enter it, then enable your module

touch samples/rust/module-name.rs

# Build & Run

Build is in ./build.sh

Run is in ./run.sh