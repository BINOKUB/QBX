use x86_64::instructions::port::Port;

pub fn eteindre() -> ! {
    unsafe {
        // Envoie de la commande de shutdown ACPI/QEMU (port 0x604)
        let mut shutdown_port = Port::new(0x604);
        shutdown_port.write(0x2000u16);

        // Fallback APM/QEMU ancien (port 0xB004)
        let mut apm_port = Port::new(0xB004);
        apm_port.write(0x2000u16);
    }

    // Boucle d'arrêt de sécurité
    loop {
        x86_64::instructions::hlt();
    }
}
