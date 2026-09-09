// QBX Centurion - Sonde active de reconnaissance réseau
// Fichier : src/commandes/probe.rs
// Description : Sonde ARP furtive (IP 0.0.0.0) avec résolution d'hôte ou balayage de segment

use crate::println;
use crate::net::e1000;
use alloc::vec::Vec;

pub fn executer(args: &[&str]) {
    let mac_source = e1000::adresse_mac();
    if mac_source == [0u8; 6] {
        println!("[ERREUR] Interface réseau e1000 non initialisée.");
        return;
    }

    if args.is_empty() {
        println!("Usage :");
        println!("  probe <ip>            -> Sonde un hôte précis (ex: probe 10.0.2.2)");
        println!("  probe scan <prefixe>  -> Balaye un sous-réseau (ex: probe scan 10.0.2)");
        return;
    }

    if args[0] == "scan" {
        if args.len() < 2 {
            println!("Usage : probe scan <prefixe> (ex: probe scan 10.0.2)");
            return;
        }
        scanner_plage(args[1], mac_source);
    } else {
        sonder_hote_unique(args[0], mac_source);
    }
}

/// Parse une chaîne IPv4 "A.B.C.D" en tableau de 4 octets
fn parser_ip(ip_str: &str) -> Option<[u8; 4]> {
    let mut parties = [0u8; 4];
    let segments: Vec<&str> = ip_str.split('.').collect();
    if segments.len() != 4 {
        return None;
    }
    for i in 0..4 {
        match segments[i].parse::<u8>() {
            Ok(val) => parties[i] = val,
            Err(_) => return None,
        }
    }
    Some(parties)
}

/// Sonde une seule adresse IP
fn sonder_hote_unique(cible_str: &str, mac_source: [u8; 6]) {
    let ip_cible = match parser_ip(cible_str) {
        Some(ip) => ip,
        None => {
            println!("[ERREUR] Format IPv4 invalide (attendu: X.X.X.X)");
            return;
        }
    };

    println!("Sonde furtive ARP vers {}.{}.{}.{}...", ip_cible[0], ip_cible[1], ip_cible[2], ip_cible[3]);
    let trame = forger_requete_arp(mac_source, [0, 0, 0, 0], ip_cible);

    if !e1000::envoyer(&trame) {
        println!("[ERREUR] Échec d'émission DMA.");
        return;
    }

    if let Some((ip, mac)) = attendre_reponse_arp(ip_cible, 500_000) {
        println!("--------------------------------------------------");
        println!("[RÉPONSE] Hôte actif détecté : {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
        println!("          Adresse MAC         : {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
        println!("--------------------------------------------------");
    } else {
        println!("Aucune réponse (hôte inactif ou filtré).");
    }
}

/// Balaye une plage réseau (ex: "10.0.2" de .1 à .10)
fn scanner_plage(prefixe: &str, mac_source: [u8; 6]) {
    let segments: Vec<&str> = prefixe.split('.').collect();
    if segments.len() != 3 {
        println!("[ERREUR] Préfixe réseau invalide (attendu: X.X.X, ex: 10.0.2)");
        return;
    }

    let a = match segments[0].parse::<u8>() { Ok(v) => v, Err(_) => return };
    let b = match segments[1].parse::<u8>() { Ok(v) => v, Err(_) => return };
    let c = match segments[2].parse::<u8>() { Ok(v) => v, Err(_) => return };

    println!("--- Scan ARP de reconnaissance sur {}.{}.{}.1 à .15 ---", a, b, c);

    let mut detectes = 0;
    for hote in 1..=15 {
        let ip_cible = [a, b, c, hote];
        let trame = forger_requete_arp(mac_source, [0, 0, 0, 0], ip_cible);

        if !e1000::envoyer(&trame) {
            continue;
        }

        if let Some((ip, mac)) = attendre_reponse_arp(ip_cible, 100_000) {
            println!(
                "[+] {}.{}.{}.{}\t-> {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                ip[0], ip[1], ip[2], ip[3],
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
            );
            detectes += 1;
        }
    }
    println!("Fin du scan : {} machine(s) active(s) répertoriée(s).", detectes);
}

/// Attend une réponse ARP ciblant l'IP demandée
fn attendre_reponse_arp(ip_cible: [u8; 4], timeout: usize) -> Option<([u8; 4], [u8; 6])> {
    for _ in 0..timeout {
        core::hint::spin_loop();

        if let Some(paquet) = e1000::scruter() {
            if paquet.donnees.len() >= 42 && paquet.donnees[12] == 0x08 && paquet.donnees[13] == 0x06 {
                let op = ((paquet.donnees[20] as u16) << 8) | (paquet.donnees[21] as u16);
                if op == 2 { // ARP Reply
                    let ip_emetteur = [
                        paquet.donnees[28],
                        paquet.donnees[29],
                        paquet.donnees[30],
                        paquet.donnees[31],
                    ];

                    if ip_emetteur == ip_cible {
                        let mut mac = [0u8; 6];
                        mac.copy_from_slice(&paquet.donnees[22..28]);
                        return Some((ip_emetteur, mac));
                    }
                }
            }
        }
    }
    None
}

/// Forge une trame ARP standard avec bourrage à 60 octets
fn forger_requete_arp(mac_src: [u8; 6], ip_src: [u8; 4], ip_dst: [u8; 4]) -> Vec<u8> {
    let mut trame = Vec::with_capacity(60);

    // En-tête Ethernet
    trame.extend_from_slice(&[0xFF; 6]);    // Broadcast destination
    trame.extend_from_slice(&mac_src);      // MAC source
    trame.extend_from_slice(&[0x08, 0x06]); // EtherType ARP

    // Charge utile ARP
    trame.extend_from_slice(&[0x00, 0x01]); // Ethernet (1)
    trame.extend_from_slice(&[0x08, 0x00]); // IPv4 (0x0800)
    trame.push(6);                          // MAC len
    trame.push(4);                          // IP len
    trame.extend_from_slice(&[0x00, 0x01]); // Request (1)

    trame.extend_from_slice(&mac_src);      // MAC source
    trame.extend_from_slice(&ip_src);       // IP source (0.0.0.0 pour mode furtif)
    trame.extend_from_slice(&[0x00; 6]);    // MAC destination inconnue
    trame.extend_from_slice(&ip_dst);       // IP cible

    while trame.len() < 60 {
        trame.push(0x00);
    }
    trame
}
