#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod vga_buffer;
mod interrupts;
mod shell;
mod power;
mod commandes;
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
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    println!("Système prêt.\n");
    print!("qbx> ");

    loop {
        x86_64::instructions::hlt();
    }
}
