// QBX Commande - Révision 0.3
// Fichier : src/commandes/mnl.rs
// Description : Manuel du système QBX (Pages 'man' intégrées)

use crate::println;

// --- [FONCTION 1 : executer] ---
// Description : Affiche la documentation d'une commande passée en argument.
pub fn executer(commande: &str) {
    let cmd = commande.trim();

    if cmd.is_empty() {
        println!("Usage : mnl <commande>");
        println!("Commandes disponibles : ntr, inf, tmps, edt, ls, cat, mnl, qtr");
        return;
    }

    match cmd {
        "ntr" => {
            println!("=== MANUEL : ntr ===");
            println!("Description : Nettoie l'écran du terminal.");
            println!("Usage       : ntr");
        }
        "inf" => {
            println!("=== MANUEL : inf ===");
            println!("Description : Affiche les informations de version et du noyau QBX.");
            println!("Usage       : inf");
        }
        "tmps" => {
            println!("=== MANUEL : tmps ===");
            println!("Description : Affiche l'heure actuelle du système.");
            println!("Usage       : tmps");
        }
        "edt" => {
            println!("=== MANUEL : edt ===");
            println!("Description : Éditeur de texte plein écran avec défilement.");
            println!("Usage       : edt [-l] [fichier]");
            println!("Options     : -l (Affiche les numéros de ligne)");
            println!("Raccourcis  : [F2] Sauvegarder dans le RamDisk | [ESC] Quitter");
        }
        "ls" => {
            println!("=== MANUEL : ls ===");
            println!("Description : Liste les fichiers enregistrés dans le système VFS (RAMDisk).");
            println!("Usage       : ls");
            println!("Affiche     : Nom du fichier et sa taille en octets.");
        }
        "cat" => {
            println!("=== MANUEL : cat ===");
            println!("Description : Affiche le contenu d'un fichier texte en mémoire.");
            println!("Usage       : cat <nom_fichier>");
            println!("Exemple     : cat notes.txt");
        }
        "mnl" => {
            println!("=== MANUEL : mnl ===");
            println!("Description : Affiche la page de manuel d'une commande donnée.");
            println!("Usage       : mnl <commande>");
        }
        "qtr" => {
            println!("=== MANUEL : qtr ===");
            println!("Description : Arrête proprement le noyau QBX (Shutdown ACPI).");
            println!("Usage       : qtr");
        }
        inconnu => {
            println!("Aucune page de manuel pour : {}", inconnu);
            println!("Tapez 'mnl' sans argument pour voir la liste des commandes.");
        }
    }
}
