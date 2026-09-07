// QBX Commande - Révision 0.1
// Fichier : src/commandes/cdr.rs
// Description : Commande de changement de répertoire (cdr - Changer de répertoire)

use crate::{println, fs};

pub fn executer(args: &str) {
    let chemin = args.trim();

    if chemin.is_empty() {
        println!("Usage : cdr <chemin>");
        return;
    }

    let mut fs_guard = fs::SYSTEME_FICHIERS.lock();
    if fs_guard.changer_repertoire(chemin) {
        println!("Déplacement réussi vers '{}'.", chemin);
    } else {
        println!("Erreur : Répertoire introuvable ou invalide.");
    }
}
