// QBX Commande - Révision 0.5
// Fichier : src/commandes/cat.rs
// Description : Affiche le contenu d'un fichier avec pagination automatique (> 22 lignes écran) ou forcée (-ppp)

use crate::{println, fs};
use crate::commandes::pager;

/// Calcule le nombre réel de lignes VGA (80 colonnes) occupées par le texte
fn calculer_lignes_ecran(texte: &str) -> usize {
    let mut total = 0;
    for ligne in texte.lines() {
        let len = ligne.chars().count();
        total += if len == 0 { 1 } else { (len + 79) / 80 };
    }
    total
}

// --- [FONCTION 1 : executer] ---
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

        if let Ok(contenu_str) = core::str::from_utf8(&contenu) {
            let lignes = calculer_lignes_ecran(contenu_str);

            // Déclenchement si l'option -ppp est demandée OU si le contenu déborde de l'écran (> 20 lignes)
            if option_ppp || lignes > 20 {
                drop(fs_guard); // Libérer le verrou VFS avant la pause interactive
                pager::afficher_avec_pagination(contenu_str);
            } else {
                for &octet in &contenu {
                    crate::print!("{}", octet as char);
                }
                if !contenu_str.ends_with('\n') {
                    println!();
                }
            }
        } else {
            println!("Erreur : le fichier ne contient pas de texte UTF-8 valide.");
        }
    } else {
        println!("Erreur : Fichier '{}' introuvable.", nom_fichier);
    }
}
