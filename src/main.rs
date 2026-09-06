#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

#[panic_handler]
fn gestionnaire_panic(information: &PanicInfo) -> ! {
    println!("{}", information);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("=== QBX (Québec UNIX) v0.1 ===");
    println!("Initialisation du système...");
    println!("Affichage VGA : OK");
    println!("Chargement du noyau bare-metal terminé.");

    loop {}
}
