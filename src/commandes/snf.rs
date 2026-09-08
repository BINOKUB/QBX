// QBX - Commande snf (Sonde d'interception réseau en mode Promiscuous)
// Fichier : src/commandes/snf.rs

use crate::println;
use crate::net::e1000;

pub fn executer() {
    println!("--- Sonde QBX active (Mode Promiscuous) ---");
    println!("Scrutateur armé. Attente de trames réseau...");

    let mut capturees = 0;
    let tentatives_max = 50_000;

    for _ in 0..tentatives_max {
        if let Some(paquet) = e1000::scruter() {
            capturees += 1;
            analyser_trame(&paquet.donnees, capturees);
            if capturees >= 5 {
                break;
            }
        }
    }

    if capturees == 0 {
        println!("Aucune trame détectée sur le segment pendant la fenêtre d'écoute.");
    } else {
        println!("-------------------------------------------");
        println!("Capture terminée : {} trame(s) interceptée(s).", capturees);
    }
}

fn analyser_trame(donnees: &[u8], index: usize) {
    if donnees.len() < 14 {
        return;
    }

    let mac_dest = &donnees[0..6];
    let mac_src  = &donnees[6..12];
    let ethertype = ((donnees[12] as u16) << 8) | (donnees[13] as u16);

    let proto = match ethertype {
        0x0800 => "IPv4",
        0x0806 => "ARP",
        0x86DD => "IPv6",
        _ => "Inconnu",
    };

    println!(
        "[{}] {} octets | {} -> {} | Proto: 0x{:04X} ({})",
        index,
        donnees.len(),
        formater_mac(mac_src),
        formater_mac(mac_dest),
        ethertype,
        proto
    );
}

fn formater_mac(mac: &[u8]) -> alloc::string::String {
    alloc::format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    )
}
