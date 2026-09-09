// QBX Commande : ls (Lister)
// Fichier : src/commandes/ls.rs
// Description : Liste le contenu d'un répertoire (courant ou cible) en mode compact ou détaillé (-l)

use crate::println;
use alloc::vec::Vec;

pub fn executer(arguments: &str) {
    let tokens: Vec<&str> = arguments.split_whitespace().collect();
    let mut mode_long = false;
    let mut chemin_cible = "";

    for token in tokens {
        if token.starts_with('-') {
            if token == "-l" || token == "-la" || token == "-al" {
                mode_long = true;
            }
        } else if chemin_cible.is_empty() {
            chemin_cible = token;
        }
    }

    let elements = match crate::fs::lister_cible(chemin_cible) {
        Ok(el) => el,
        Err("est_fichier") => {
            println!("ls: '{}' est un fichier.", chemin_cible);
            return;
        }
        Err("introuvable") => {
            println!("ls: impossible d'accéder à '{}': Aucun fichier ou dossier de ce type.", chemin_cible);
            return;
        }
        _ => {
            println!("ls: erreur lors de l'accès au répertoire.");
            return;
        }
    };

    if elements.is_empty() {
        println!("(Dossier vide)");
        return;
    }

    if mode_long {
        let total = elements.len();
        println!("total {}", total);

        for (nom, taille, est_dossier, h) in &elements {
            let (perms, liens) = if *est_dossier {
                ("drwxr-xr-x", 2)
            } else {
                ("-rw-r--r--", 1)
            };

            // Format standard UNIX : [droits] [liens] [user] [group] [taille] [HH:MM:SS] [nom]
            println!(
                "{} {:>2} qbx qbx {:>8} {:02}:{:02}:{:02} {}",
                perms,
                liens,
                taille,
                h.heure,
                h.minute,
                h.seconde,
                nom
            );
        }
    } else {
        // Affichage compact standard
        for (nom, _taille, est_dossier, _h) in &elements {
            if *est_dossier {
                println!("{}/", nom);
            } else {
                println!("{}", nom);
            }
        }
    }
}
