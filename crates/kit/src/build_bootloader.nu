#!/bin/nu

# TODO: rewrite this script in pure rust?

mkdir build build/mnt build/boot

dd if=/dev/zero bs=1M count=0 seek=8096 of=build/rootfs.img
parted -s build/rootfs.img mklabel gpt
parted -s build/rootfs.img mkpart ESP fat32 1MiB 301MiB
parted -s build/rootfs.img mkpart ROOT ext4 301MiB 100%
parted -s build/rootfs.img set 1 esp on

limine-deploy build/rootfs.img

sudo losetup -Pf --show build/rootfs.img
sudo mkfs.fat -F32 /dev/loop0p1
sudo mkfs.ext4 /dev/loop0p2

sudo mount /dev/loop0p1 ./build/boot
sudo mount /dev/loop0p2 ./build/mnt

sudo mkdir ./build/boot/efi ./build/boot/efi/boot
sudo cp -r kit_rootfs/* ./build/mnt/
sudo cp /usr/share/limine/limine.sys ./build/boot/
sudo cp /usr/share/limine/BOOTX64.EFI ./build/boot/efi/boot/
sudo cp ./limine.cfg ./build/boot/
sudo cp ./services.json ./build/mnt/

sudo umount /dev/loop0p1
sudo umount /dev/loop0p2
sudo losetup -D
