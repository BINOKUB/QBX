// QBX Pager - Révision 0.2
// Fichier : src/commandes/pager.rs
// Description : Utilitaire modulaire de pagination d'écran (-ppp) avec attente clavier active via le port PS/2 (0x60)

use alloc::vec::Vec;
use crate::{print, println};
use x86_64::instructions::port::Port;

// --- [FONCTION 1 : afficher_avec_pagination] ---
// Description : Découpe un flux textuel et l'affiche par tranches d'écran (mode page par page) en suspendant l'affichage jusqu'à l'appui sur Espace/Entrée ou Q.
pub fn afficher_avec_pagination(texte: &str) {
    let lignes: Vec<&str> = texte.lines().collect();
    let mut ligne_courante = 0;
    let hauteur_ecran = 20;

    while ligne_courante < lignes.len() {
        let fin = (ligne_courante + hauteur_ecran).min(lignes.len());
        for i in ligne_courante..fin {
            println!("{}", lignes[i]);
        }

        ligne_courante = fin;

        if ligne_courante < lignes.len() {
            print!("-- [ESPACE = Page suivante | Q = Quitter] --");
            
            // Boucle de scrutation active du clavier PS/2 (Port 0x60)
            let mut port_clavier = Port::new(0x60);
            loop {
                unsafe {
                    let scancode: u8 = port_clavier.read();
                    // Scancodes Set 1 (Make codes) :
                    // Espace = 0x39, Entrée = 0x1C, Q = 0x10
                    match scancode {
                        0x39 | 0x1C => { // Espace ou Entrée -> Page suivante
                            println!();
                            break;
                        }
                        0x10 => { // Q -> Quitter l'affichage en cours
                            println!();
                            return;
                        }
                        _ => {}
                    }
                }
                // Pause CPU pour éviter de saturer le bus I/O pendant l'attente
                for _ in 0..10000 {
                    core::hint::spin_loop();
                }
            }
        }
    }
}
