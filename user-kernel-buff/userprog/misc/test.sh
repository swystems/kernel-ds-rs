#!/bin/sh

linux_dir=~/git-repo/linux
mod_dir=/home/tong/git-repo/rust-mod
mod=${1}
userprogram=${2}
shared_dir=~/git-repo/kernel-ds-rs/ko-userporg

cd ${mod_dir} || echo "cannot exec 'cd ${mod_dir}'" && \
make LLVM=-15 ARCH=x86_64 && \
cp "${mod}" ${linux_dir}/rustdev.ko && \
cp "${mod}" ${shared_dir}/"${mod}" && \
make clean && \
cp /home/tong/git-repo/userproj/target/x86_64-unknown-linux-musl/release/userproj ~/git-repo/linux/userprog && \
cp /home/tong/git-repo/userproj/target/x86_64-unknown-linux-musl/release/userproj ${shared_dir}/"${userprogram}" && \
cp /home/tong/git-repo/userproj/misc/test-script ~/git-repo/linux/test-script && \
cd ${linux_dir} || echo "cannot exec 'cd ${linux_dir}'" && \
usr/gen_init_cpio .github/workflows/qemu-initramfs.desc > test.img && \
qemu-system-x86_64 \
    -kernel arch/x86/boot/bzImage \
    -initrd test.img \
    -M pc \
    -m 12G \
    -cpu host \
    -smp 4 \
    -nographic \
    -vga none \
    -append 'console=ttyS0' \
    -no-reboot \
    -s \
    -machine ubuntu,accel=kvm