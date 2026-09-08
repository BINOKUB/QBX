// QBX - Commande net (Statut de l'interface et surveillance réseau)
// Fichier : src/commandes/net.rs

use crate::println;
use crate::net::e1000::PILOTE_E1000;

pub fn executer() {
    let mut guard = PILOTE_E1000.lock();

    match guard.as_mut() {
        Some(carte) => {
            carte.rafraichir_statut();
            println!("--- Interface Réseau eth0 (Intel 82540EM) ---");
            println!(
                "  Adresse MAC  : {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                carte.mac[0], carte.mac[1], carte.mac[2],
                carte.mac[3], carte.mac[4], carte.mac[5]
            );
            println!(
                "  État liaison : {}",
                if carte.lien_actif { "ACTIF / CONNECTÉ (Full Duplex 1000 Mbps)" } else { "DÉCONNECTÉ (Down)" }
            );
            println!("  Pilote       : MMIO Direct (BAR0)");
            println!("---------------------------------------------");
        }
        None => {
            println!("net: Aucune interface réseau active détectée.");
        }
    }
}
