// QBX Commande - Révision 0.3
// Fichier : src/commandes/cat.rs
// Description : Affiche le contenu d'un fichier texte du VFS avec support optionnel du pager (-ppp)

use crate::{println, fs};
use crate::commandes::pager;

// --- [FONCTION 1 : executer] ---
// Description : Analyse les arguments, recherche le fichier dans le VFS et l'imprime (avec ou sans pagination).
pub fn executer(args: &str) {
    let mut option_ppp = false;
    let mut nom_fichier = "";

    let mut parties = args.split_whitespace();
    while let Some(arg) = parties.next() {
        if arg == "-ppp" {
            option_ppp = true;
        } else if nom_fichier.is_empty() {
            nom_fichier = arg;
        }
    }

    if nom_fichier.is_empty() {
        println!("Usage : cat <nom_fichier> [-ppp]");
        return;
    }

    let fs_guard = fs::SYSTEME_FICHIERS.lock();
    if let Some(contenu) = fs_guard.lire(nom_fichier) {
        if contenu.is_empty() {
            println!("(Fichier vide)");
            return;
        }
        
        if let Ok(contenu_str) = core::str::from_utf8(contenu) {
            if option_ppp {
                pager::afficher_avec_pagination(contenu_str);
            } else {
                for &octet in contenu {
                    crate::print!("{}", octet as char);
                }
                println!();
            }
        } else {
            println!("Erreur : le fichier ne contient pas de texte UTF-8 valide.");
        }
    } else {
        println!("Erreur : Fichier '{}' introuvable.", nom_fichier);
    }
}
