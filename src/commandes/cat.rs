// QBX Commande - Révision 0.2
// Fichier : src/commandes/cat.rs
// Description : Affiche le contenu d'un fichier texte enregistré dans le VFS dynamique

use crate::{println, fs};

// --- [FONCTION 1 : executer] ---
// Description : Recherche un fichier dans le RamDisk et imprime son contenu à l'écran.
pub fn executer(nom_fichier: &str) {
    if nom_fichier.is_empty() {
        println!("Usage : cat <nom_fichier>");
        return;
    }

    let fs_guard = fs::SYSTEME_FICHIERS.lock();
    if let Some(contenu) = fs_guard.lire(nom_fichier) {
        if contenu.is_empty() {
            println!("(Fichier vide)");
            return;
        }
        
        for &octet in contenu {
            crate::print!("{}", octet as char);
        }
        println!();
    } else {
        println!("Erreur : Fichier '{}' introuvable.", nom_fichier);
    }
}
