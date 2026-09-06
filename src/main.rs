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

    interrupts::init_idt();
    println!("Table IDT : OK");

    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
    println!("Interruptions matérielles : OK");

    println!("\nPrêt ! Tapez au clavier :");

    loop {
        x86_64::instructions::hlt();
    }
}
