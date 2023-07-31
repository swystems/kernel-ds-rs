#!/bin/bash

linuxdir=~/git-repo/linux

gcc userprog.c -static -o ${linuxdir}/userprog && \
make LLVM=-15 ARCH=x86_64 && \
cp rustdev.ko ${linuxdir}/rustdev.ko && \
cp test-script ${linuxdir}/ && \
cd ${linuxdir} && pwd && \
usr/gen_init_cpio .github/workflows/qemu-initramfs.desc > test.img && \
qemu-system-x86_64 \
    -kernel arch/x86/boot/bzImage \
    -initrd test.img \
    -M pc \
    -m 4G \
    -cpu host \
    -smp 4 \
    -nographic \
    -vga none \
    -append 'console=ttyS0' \
    -no-reboot \
    -s \
    -machine ubuntu,accel=kvm \