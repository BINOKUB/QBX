#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn gestionnaire_panic(_information: &PanicInfo) -> ! {
    loop {}
}

static BONJOUR_QBX: &[u8] = b"Bonjour QBX ! Bienvenue dans le noyau.";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in BONJOUR_QBX.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0x0f;
        }
    }

    loop {}
}
