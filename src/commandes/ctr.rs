// QBX Commande - Révision 0.1
// Fichier : src/commandes/ctr.rs
// Description : Commande de création de répertoire (ctr - Créer répertoire)

use crate::{println, fs};

// --- [FONCTION 1 : executer] ---
// Description : Analyse l'argument fourni et demande au VFS la création du répertoire.
pub fn executer(args: &str) {
    let nom_repertoire = args.trim();

    if nom_repertoire.is_empty() {
        println!("Usage : ctr <nom_repertoire>");
        return;
    }

    let mut fs_guard = fs::SYSTEME_FICHIERS.lock();
    if fs_guard.creer_repertoire(nom_repertoire) {
        println!("Répertoire '{}' créé avec succès.", nom_repertoire);
    } else {
        println!("Erreur : Impossible de créer '{}' (déjà existant ou nom invalide).", nom_repertoire);
    }
}
