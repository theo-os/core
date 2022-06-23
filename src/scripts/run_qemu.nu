#!/bin/nu

qemu-system-x86_64 -m 1G -hda build/rootfs.img -netdev user,id=user.0 -device e1000,netdev=user.0  -bios /usr/share/edk2-ovmf/x64/OVMF_CODE.fd -accel kvm -nographic -serial mon:stdio
