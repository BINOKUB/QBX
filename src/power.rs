// QBX Power Module - Révision 0.2
// Fichier : src/power.rs
// Description : Gestion de l'extinction matérielle de la machine virtuelle via bus ACPI et APM

use x86_64::instructions::port::Port;

// --- [FONCTION 1 : eteindre] ---
// Description : Envoie le signal de coupure de courant sur les ports ACPI/APM et entre en boucle d'arrêt.
pub fn eteindre() -> ! {
    unsafe {
        // Commande d'extinction ACPI/QEMU (port 0x604)
        let mut shutdown_port = Port::new(0x604);
        shutdown_port.write(0x2000u16);

        // Fallback APM/QEMU ancien (port 0xB004)
        let mut apm_port = Port::new(0xB004);
        apm_port.write(0x2000u16);
    }

    // Boucle d'arrêt de sécurité si l'extinction matérielle échoue
    loop {
        x86_64::instructions::hlt();
    }
}
