// QBX Commande - Révision 0.2
// Fichier : src/commandes/ls.rs
// Description : Liste les fichiers présents dans le système de fichiers dynamique

use crate::{println, fs};

// --- [FONCTION 1 : executer] ---
// Description : Parcourt le vecteur du RamDisk et affiche le nom et la taille.
pub fn executer() {
    let fs_guard = fs::SYSTEME_FICHIERS.lock();

    println!("Nom du fichier                   Taille");
    println!("---------------------------------------");

    if fs_guard.fichiers.is_empty() {
        println!("(Aucun fichier dans le système de fichiers)");
        return;
    }

    for f in fs_guard.fichiers.iter() {
        println!("{:<32} {} octets", f.nom, f.contenu.len());
    }
}
