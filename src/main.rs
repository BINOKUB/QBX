#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod vga_buffer;
mod interrupts;

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

    // Initialisation des interruptions
    interrupts::init_idt();
    println!("Table IDT chargée : OK");

    // Test d'interruption logicielle
    x86_64::instructions::interrupts::int3();

    println!("Chargement du noyau bare-metal terminé.");

    loop {}
}
