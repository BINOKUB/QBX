echo #!/bin/bash
echo cargo bootimage && qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-qbx.bin -usb -device usb-tablet -display gtk,show-cursor=on 
chmod +x run.sh
