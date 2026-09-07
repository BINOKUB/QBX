// QBX Commande : ctr (Créer Répertoire)
// Fichier : src/commandes/ctr.rs
// Description : Commande shell pour instancier un nouveau sous-dossier dans le répertoire courant

use crate::println;

pub fn executer(nom_dossier: &str) {
    if nom_dossier.is_empty() {
        println!("Utilisation : ctr <nom_dossier>");
        return;
    }

    // Sécurité : interdire les slashes pour éviter de créer des noms composites corrompus
    if nom_dossier.contains('/') {
        println!("Erreur : Le nom du dossier ne peut pas contenir le caractère '/'.");
        return;
    }

    if crate::fs::SYSTEME_FICHIERS.lock().creer_repertoire(nom_dossier) {
        println!("Répertoire '{}' créé avec succès.", nom_dossier);
    } else {
        println!("Erreur : Impossible de créer le répertoire '{}' (existe déjà ou invalide).", nom_dossier);
    }
}
