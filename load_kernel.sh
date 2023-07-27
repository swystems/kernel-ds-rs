cd ~/git-repo/linux
usr/gen_init_cpio .github/workflows/qemu-initramfs.desc > qemu-initramfs.img
qemu-system-x86_64 \
    -kernel arch/x86/boot/bzImage \
    -initrd qemu-initramfs.img \
    -M pc \
    -m 1G \
    -cpu Cascadelake-Server \
    -smp $(nproc) \
    -nographic \
    -vga none \
    -append 'console=ttyS0' \
    -no-reboot
