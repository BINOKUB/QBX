// QBX Pager - Révision 0.3
// Fichier : src/commandes/pager.rs
// Description : Téléavertisseur (pager) VGA avec lecture contrôlée du port PS/2 (0x64 / 0x60)

use alloc::vec::Vec;
use crate::{print, println};
use x86_64::instructions::port::Port;

/// Lit un scancode frais depuis le contrôleur clavier PS/2 en vérifiant le statut
fn lire_scancode_bloquant() -> u8 {
    let mut port_statut: Port<u8> = Port::new(0x64);
    let mut port_donnees: Port<u8> = Port::new(0x60);

    loop {
        unsafe {
            // Le bit 0 du port 0x64 indique si des données sont prêtes dans le buffer de sortie
            if (port_statut.read() & 0x01) != 0 {
                return port_donnees.read();
            }
        }
        core::hint::spin_loop();
    }
}

/// Purge tout scancode résiduel (ex: la touche Entrée ayant lancé la commande)
fn purger_tampon_clavier() {
    let mut port_statut: Port<u8> = Port::new(0x64);
    let mut port_donnees: Port<u8> = Port::new(0x60);

    unsafe {
        while (port_statut.read() & 0x01) != 0 {
            let _ = port_donnees.read();
        }
    }
}

pub fn afficher_avec_pagination(texte: &str) {
    let lignes: Vec<&str> = texte.lines().collect();
    let mut ligne_courante = 0;
    let hauteur_page = 22; // 22 lignes affichées pour laisser la place au prompt sur un écran 80x25

    while ligne_courante < lignes.len() {
        let fin = (ligne_courante + hauteur_page).min(lignes.len());
        for i in ligne_courante..fin {
            println!("{}", lignes[i]);
        }

        ligne_courante = fin;

        if ligne_courante < lignes.len() {
            // Purge préalable pour éviter de consommer l'Entrée du shell
            purger_tampon_clavier();

            print!("-- [ESPACE = Page suivante | ENTREE = 1 ligne | Q = Quitter] --");

            loop {
                let scancode = lire_scancode_bloquant();

                // Ignorer les scancodes de relâchement (break codes avec bit 7 à 1)
                if scancode & 0x80 != 0 {
                    continue;
                }

                match scancode {
                    0x39 => {
                        // ESPACE : page suivante complète
                        println!();
                        break;
                    }
                    0x1C => {
                        // ENTREE : descend d'une seule ligne
                        println!();
                        if ligne_courante < lignes.len() {
                            println!("{}", lignes[ligne_courante]);
                            ligne_courante += 1;
                        }
                        if ligne_courante < lignes.len() {
                            purger_tampon_clavier();
                            print!("-- [ESPACE = Page suivante | ENTREE = 1 ligne | Q = Quitter] --");
                            continue;
                        } else {
                            break;
                        }
                    }
                    0x10 => {
                        // Q : quitter immédiatement le pager
                        println!();
                        return;
                    }
                    _ => {}
                }
            }
        }
    }
}
