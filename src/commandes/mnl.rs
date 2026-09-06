// QBX Commande - Révision 0.3
// Fichier : src/commandes/mnl.rs
// Description : Manuel du système QBX au format UNIX (Pages man)

use crate::println;

// --- [FONCTION 1 : executer] ---
// Description : Affiche la documentation au format UNIX traditionnel de la commande ciblée.
pub fn executer(cible: &str) {
    match cible {
        "ntr" => {
            println!("NTR(1)                   Manuel de référence QBX                  NTR(1)\n");
            println!("NOM");
            println!("    ntr - Nettoyer l'écran de la console\n");
            println!("SYNOPSIS");
            println!("    ntr\n");
            println!("DESCRIPTION");
            println!("    Efface l'intégralité du contenu de la grille texte VGA et replace");
            println!("    le curseur matériel au coin supérieur gauche.\n");
            println!("QBX v0.1.0                       Révision 0.1                       NTR(1)");
        }
        "qtr" => {
            println!("QTR(1)                   Manuel de référence QBX                  QTR(1)\n");
            println!("NOM");
            println!("    qtr - Quitter et éteindre le système\n");
            println!("SYNOPSIS");
            println!("    qtr\n");
            println!("DESCRIPTION");
            println!("    Déclenche l'extinction matérielle de la machine virtuelle via les");
            println!("    ports d'alimentation ACPI/APM.\n");
            println!("QBX v0.1.0                       Révision 0.2                       QTR(1)");
        }
        "inf" => {
            println!("INF(1)                   Manuel de référence QBX                  INF(1)\n");
            println!("NOM");
            println!("    inf - Informations système et noyau\n");
            println!("SYNOPSIS");
            println!("    inf\n");
            println!("DESCRIPTION");
            println!("    Affiche l'architecture du processeur, le mode d'affichage VGA,");
            println!("    la version courante du noyau et le statut de l'énergie.\n");
            println!("QBX v0.1.0                       Révision 0.1                       INF(1)");
        }
        "tmps" => {
            println!("TMPS(1)                  Manuel de référence QBX                 TMPS(1)\n");
            println!("NOM");
            println!("    tmps - Horloge temps réel (RTC)\n");
            println!("SYNOPSIS");
            println!("    tmps\n");
            println!("DESCRIPTION");
            println!("    Interroge les registres matériels CMOS (ports 0x70/0x71) pour");
            println!("    afficher l'heure système en direct.\n");
            println!("QBX v0.1.0                       Révision 0.1                      TMPS(1)");
        }
        "mnl" => {
            println!("MNL(1)                   Manuel de référence QBX                  MNL(1)\n");
            println!("NOM");
            println!("    mnl - Manuel du système\n");
            println!("SYNOPSIS");
            println!("    mnl <commande>\n");
            println!("DESCRIPTION");
            println!("    Affiche la page de manuel au format UNIX de la commande spécifiée.\n");
            println!("QBX v0.1.0                       Révision 0.3                       MNL(1)");
        }
        "edt" => {
            println!("EDT(1)                   Manuel de référence QBX                  EDT(1)\n");
            println!("NOM");
            println!("    edt - Éditeur de texte plein écran\n");
            println!("SYNOPSIS");
            println!("    edt\n");
            println!("DESCRIPTION");
            println!("    Ouvre un espace d'édition de texte 80x23 en mémoire RAM.");
            println!("    Appuyez sur la touche [ESC] pour quitter l'éditeur.\n");
            println!("QBX v0.1.0                       Révision 0.1                       EDT(1)");
        }
        "aide" => {
            println!("AIDE(1)                  Manuel de référence QBX                 AIDE(1)\n");
            println!("NOM");
            println!("    aide - Sommaire des commandes\n");
            println!("SYNOPSIS");
            println!("    aide\n");
            println!("DESCRIPTION");
            println!("    Affiche le sommaire rapide de toutes les commandes installées.\n");
            println!("QBX v0.1.0                       Révision 0.5                      AIDE(1)");
        }
        "" => {
            println!("Usage : mnl <commande> (Exemple : mnl inf)");
        }
        cmd => {
            println!("Pas de page de manuel pour '{}'", cmd);
        }
    }
}
