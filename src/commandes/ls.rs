// QBX Commande : ls (Lister)
// Fichier : src/commandes/ls.rs
// Description : Liste le contenu du répertoire courant (mode standard ou format long -l)

use crate::println;
use alloc::vec::Vec;

pub fn executer(arguments: &str) {
    let args: Vec<&str> = arguments.split_whitespace().collect();
    let mode_long = args.contains(&"-l") || args.contains(&"-la") || args.contains(&"-al");

    let elements = crate::fs::lister();

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
        // Affichage standard : dossiers marqués d'un slash '/'
        for (nom, _taille, est_dossier, _h) in &elements {
            if *est_dossier {
                println!("{}/", nom);
            } else {
                println!("{}", nom);
            }
        }
    }
}
