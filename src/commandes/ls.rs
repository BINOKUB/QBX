// QBX Commande - Révision 0.3
// Fichier : src/commandes/ls.rs
// Description : Liste les fichiers et répertoires du dossier courant

use crate::{println, fs};

pub fn executer() {
    let elements = fs::lister();

    if elements.is_empty() {
        println!("(Dossier vide)");
        return;
    }

    println!("--- Contenu du répertoire ---");
    for (nom, taille) in elements {
        println!("  {}  ({} octets)", nom, taille);
    }
    println!("-----------------------------");
}
