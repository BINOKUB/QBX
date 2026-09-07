// QBX Commande - Révision 0.4
// Fichier : src/commandes/mnl.rs
// Description : Manuel du système QBX (Pages 'man' intégrées style UNIX classique)

use crate::println;

// --- [FONCTION 1 : executer] ---
// Description : Affiche la documentation d'une commande passée en argument au format classique.
pub fn executer(commande: &str) {
    let cmd = commande.trim();

    if cmd.is_empty() {
        println!("Usage : mnl <commande>");
        println!("Commandes disponibles : ntr, inf, tmps, edt, ls, cat, mnl, qtr");
        return;
    }

    match cmd {
        "ntr" => {
            println!("NTR(1)                      Manuel de référence QBX                      NTR(1)\n");
            println!("NOM");
            println!("     ntr - Nettoyeur d'écran\n");
            println!("SYNOPSIS");
            println!("     ntr\n");
            println!("DESCRIPTION");
            println!("     Nettoie intégralement l'écran du terminal et replace le curseur.");
            println!("     Efface le tampon d'affichage VGA actuel.\n");
            println!("QBX v0.1.0                      Révision 0.1                      NTR(1)");
        }
        "inf" => {
            println!("INF(1)                      Manuel de référence QBX                      INF(1)\n");
            println!("NOM");
            println!("     inf - Informations système\n");
            println!("SYNOPSIS");
            println!("     inf\n");
            println!("DESCRIPTION");
            println!("     Affiche les informations de version, d'architecture et de");
            println!("     configuration matérielle du noyau QBX.\n");
            println!("QBX v0.1.0                      Révision 0.1                      INF(1)");
        }
        "tmps" => {
            println!("TMPS(1)                     Manuel de référence QBX                     TMPS(1)\n");
            println!("NOM");
            println!("     tmps - Horloge système\n");
            println!("SYNOPSIS");
            println!("     tmps\n");
            println!("DESCRIPTION");
            println!("     Interroge le composant RTC (Real Time Clock) via les ports I/O");
            println!("     pour afficher l'heure actuelle du système.\n");
            println!("QBX v0.1.0                      Révision 0.1                     TMPS(1)");
        }
        "edt" => {
            println!("EDT(1)                      Manuel de référence QBX                      EDT(1)\n");
            println!("NOM");
            println!("     edt - Éditeur de texte plein écran\n");
            println!("SYNOPSIS");
            println!("     edt [-l] [fichier]\n");
            println!("DESCRIPTION");
            println!("     Ouvre un espace d'édition de texte 80x23 en mémoire RAM.");
            println!("     Appuyez sur la touche [F2] pour sauvegarder dans le RamDisk.");
            println!("     Appuyez sur la touche [ESC] pour quitter l'éditeur.\n");
            println!("QBX v0.1.0                      Révision 0.1                      EDT(1)");
        }
        "ls" => {
            println!("LS(1)                       Manuel de référence QBX                       LS(1)\n");
            println!("NOM");
            println!("     ls - Lister les fichiers en mémoire\n");
            println!("SYNOPSIS");
            println!("     ls\n");
            println!("DESCRIPTION");
            println!("     Parcourt et liste les fichiers enregistrés dans le système de");
            println!("     fichiers VFS (RAMDisk). Affiche le nom et la taille en octets.\n");
            println!("QBX v0.1.0                      Révision 0.1                      LS(1)");
        }
        "cat" => {
            println!("CAT(1)                      Manuel de référence QBX                      CAT(1)\n");
            println!("NOM");
            println!("     cat - Afficher le contenu d'un fichier\n");
            println!("SYNOPSIS");
            println!("     cat <nom_fichier>\n");
            println!("DESCRIPTION");
            println!("     Recherche un fichier texte dans la mémoire virtuelle (VFS)");
            println!("     et imprime son contenu brut sur la sortie standard.\n");
            println!("QBX v0.1.0                      Révision 0.1                      CAT(1)");
        }
        "mnl" => {
            println!("MNL(1)                      Manuel de référence QBX                      MNL(1)\n");
            println!("NOM");
            println!("     mnl - Manuel système\n");
            println!("SYNOPSIS");
            println!("     mnl <commande>\n");
            println!("DESCRIPTION");
            println!("     Affiche la page de manuel formatée d'une commande système.");
            println!("     Inspiré du format classique des pages MAN UNIX.\n");
            println!("QBX v0.1.0                      Révision 0.1                      MNL(1)");
        }
        "qtr" => {
            println!("QTR(1)                      Manuel de référence QBX                      QTR(1)\n");
            println!("NOM");
            println!("     qtr - Quitter le système\n");
            println!("SYNOPSIS");
            println!("     qtr\n");
            println!("DESCRIPTION");
            println!("     Arrête proprement le noyau QBX en envoyant un signal de");
            println!("     coupure d'alimentation sur les ports ACPI/APM.\n");
            println!("QBX v0.1.0                      Révision 0.1                      QTR(1)");
        }
        inconnu => {
            println!("Aucune page de manuel pour : {}", inconnu);
            println!("Tapez 'mnl' sans argument pour voir la liste des commandes.");
        }
    }
}
