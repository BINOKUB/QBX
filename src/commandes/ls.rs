// QBX Commande - Révision 0.1
// Fichier : src/commandes/ls.rs
// Description : Liste les fichiers présents dans le système de fichiers RAM (VFS)

use crate::{println, fs};

// --- [FONCTION 1 : executer] ---
// Description : Parcours la table du RamDisk et affiche le nom et la taille de chaque fichier.
pub fn executer() {
    let fs_guard = fs::SYSTEME_FICHIERS.lock();
    let mut trouve = false;

    println!("Nom du fichier                   Taille");
    println!("---------------------------------------");

    for f in fs_guard.fichiers.iter() {
        if f.utilise {
            trouve = true;
            if let Ok(nom) = core::str::from_utf8(&f.nom[..f.nom_len]) {
                println!("{:<32} {} octets", nom, f.taille);
            }
        }
    }

    if !trouve {
        println!("(Aucun fichier dans le système de fichiers)");
    }
}
