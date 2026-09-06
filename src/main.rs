// QBX Core - Révision 0.2
// Fichier : src/main.rs
// Description : Point d'entrée du noyau bare-metal x86-64

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod vga_buffer;
mod interrupts;
mod shell;
mod power;
mod commandes;

pub mod fs;

use core::panic::PanicInfo;

// --- [FONCTION 1 : gestionnaire_panic] ---
// Description : Handler exécuté en cas de panic dans le noyau.
#[panic_handler]
fn gestionnaire_panic(information: &PanicInfo) -> ! {
    println!("{}", information);
    loop {}
}

// --- [FONCTION 2 : _start] ---
// Description : Point d'entrée principal appelé par le bootloader.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("=== QBX (Québec UNIX) v0.1 ===");
    println!("Initialisation du système...");

    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();


   println!("Système prêt.\n");
    print!("qbx> ");

    loop {
        x86_64::instructions::hlt();
    }
}
