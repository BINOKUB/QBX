// QBX Commande - Révision 0.1
// Fichier : src/commandes/tmps.rs
// Description : Lecture de l'horloge temps réel (RTC CMOS) via les ports I/O 0x70 et 0x71

use crate::println;
use x86_64::instructions::port::Port;

// --- [FONCTION 1 : lire_registre_rtc] ---
// Description : Lit un registre spécifique dans la mémoire CMOS.
fn lire_registre_rtc(registre: u8) -> u8 {
    unsafe {
        let mut port_index = Port::<u8>::new(0x70);
        let mut port_donnee = Port::<u8>::new(0x71);

        port_index.write(registre);
        let valeur = port_donnee.read();
        
        // Conversion BCD vers Décimal
        (valeur & 0x0F) + ((valeur / 16) * 10)
    }
}

// --- [FONCTION 2 : executer] ---
// Description : Extrait l'heure, les minutes et les secondes du composant RTC.
pub fn executer() {
    let secondes = lire_registre_rtc(0x00);
    let minutes = lire_registre_rtc(0x02);
    let heures = lire_registre_rtc(0x04);

    println!("Horloge système (RTC) : {:02}h {:02}m {:02}s", heures, minutes, secondes);
}
