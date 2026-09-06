// QBX Commande - Révision 0.1
// Fichier : src/commandes/cat.rs
// Description : Affiche le contenu d'un fichier texte enregistré dans le VFS

use crate::{println, fs};

// --- [FONCTION 1 : executer] ---
// Description : Recherche un fichier dans le RamDisk et imprime son contenu à l'écran.
pub fn executer(nom_fichier: &str) {
    if nom_fichier.is_empty() {
        println!("Usage : cat <nom_fichier>");
        return;
    }

    let fs_guard = fs::SYSTEME_FICHIERS.lock();
    if let Some((contenu, taille)) = fs_guard.lire(nom_fichier) {
        for &octet in &contenu[..taille] {
            if octet != 0 {
                crate::print!("{}", octet as char);
            }
        }
        println!();
    } else {
        println!("Erreur : Fichier '{}' introuvable.", nom_fichier);
    }
}
