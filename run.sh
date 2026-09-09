#!/usr/bin/env bash
set -e

# Création automatique du disque virtuel s'il n'existe pas encore
if [ ! -f disk.img ]; then
    echo "[RUN] disk.img introuvable, création d'une image brute de 1 Go..."
    qemu-img create -f raw disk.img 1G
fi

# Lancement de l'émulateur x86_64
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-qbx.bin \
    -device e1000,netdev=net0 \
    -netdev user,id=net0 \
    -drive id=disk,file=disk.img,format=raw,if=none \
    -device ahci,id=ahci \
    -device ide-hd,drive=disk,bus=ahci.0
