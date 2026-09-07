// QBX VFS Path - Révision 0.2
// Fichier : src/fs/path.rs
// Description : Analyse, normalisation et découpage des chemins du système de fichiers hiérarchique

use alloc::string::String;
use alloc::vec::Vec;

// --- [FONCTION 1.1 : nettoyer_chemin] ---
// Description : Nettoie un chemin brut en préservant les remontées '..' pour les appliquer au parcours.
pub fn nettoyer_chemin(chemin: &str) -> Vec<String> {
    let mut elements = Vec::new();

    for partie in chemin.split('/') {
        if partie.is_empty() || partie == "." {
            continue; // Ignore les slashes multiples et le dossier courant '.'
        } else {
            elements.push(String::from(partie)); // Conserve '..' et les noms de dossiers
        }
    }

    elements
}

// --- [FONCTION 1.2 : est_absolu] ---
// Description : Vérifie si un chemin textuel commence par une racine absolue ('/').
pub fn est_absolu(chemin: &str) -> bool {
    chemin.starts_with('/')
}
