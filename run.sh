cd /home/guest/linux
make
qemu-system-x86_64 -nographic -kernel vmlinux -initrd initrd.img -nic user