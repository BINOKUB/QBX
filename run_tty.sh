#!/usr/bin/bash
set -e

# 1. Compiler l'image bootable du noyau
# cargo bootimage

# 2. Lancer QEMU en mode texte pur (sans interface graphique X11)
# qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-qbx.bin -display curses

#!/usr/bin/bash
set -e

# cargo bootimage

qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-qbx.bin \
    -device e1000,netdev=net0 \
    -netdev user,id=net0 \
    -drive id=disk,file=disk.img,format=raw,if=none \
    -device ahci,id=ahci \
    -device ide-hd,drive=disk,bus=ahci.0 \
    -display curses
