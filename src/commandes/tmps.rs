// QBX Commande - Révision 0.2
// Fichier : src/commandes/tmps.rs
// Description : Lecture de l'horloge temps réel (RTC CMOS) via les ports I/O 0x70 et 0x71

use crate::println;
use x86_64::instructions::port::Port;

// --- [FONCTION 1 : lire_registre_rtc] ---
// Description : Lit un registre spécifique dans la mémoire CMOS.
pub fn lire_registre_rtc(registre: u8) -> u8 {
    unsafe {
        let mut port_index = Port::<u8>::new(0x70);
        let mut port_donnee = Port::<u8>::new(0x71);

        port_index.write(registre);
        let valeur = port_donnee.read();
        
        // Conversion BCD vers Décimal
        (valeur & 0x0F) + ((valeur / 16) * 10)
    }
}

// --- [FONCTION 2 : obtenir_heure_actuelle] ---
// Description : Retourne un triplet (heures, minutes, secondes) depuis la RTC.
pub fn obtenir_heure_actuelle() -> (u8, u8, u8) {
    let secondes = lire_registre_rtc(0x00);
    let minutes = lire_registre_rtc(0x02);
    let heures = lire_registre_rtc(0x04);
    (heures, minutes, secondes)
}

// --- [FONCTION 3 : executer] ---
pub fn executer() {
    let (heures, minutes, secondes) = obtenir_heure_actuelle();
    println!("Horloge système (RTC) : {:02}h {:02}m {:02}s", heures, minutes, secondes);
}
