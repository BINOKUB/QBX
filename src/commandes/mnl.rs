// QBX Commande - Révision 0.7
// Fichier : src/commandes/mnl.rs
// Description : Manuel du système QBX avec support du pager modulaire pour les pages longues

use crate::println;
use crate::commandes::pager;

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
            let texte_manuel = "EDT(1)                      Manuel de référence QBX                      EDT(1)\n\
\n\
NOM\n\
     edt - Éditeur de texte plein écran\n\
\n\
SYNOPSIS\n\
     edt [-l] [fichier]\n\
\n\
DESCRIPTION\n\
     Edt est un éditeur de texte orienté mémoire (RAMDisk) conçu pour QBX.\n\
     Il est utilisé pour créer, afficher, modifier et manipuler des fichiers\n\
     textuels en mode plein écran (80x23).\n\
\n\
     Si aucun fichier n'est spécifié au démarrage, l'éditeur s'ouvre sur\n\
     un espace de travail vierge et anonyme ([Sans nom]) sans impacter le disque.\n\
     Les modifications s'effectuent dans un tampon dynamique et ne sont enregistrées\n\
     qu'à la demande explicite de l'utilisateur.\n\
\n\
OPTIONS\n\
     Les options suivantes sont disponibles :\n\
\n\
     -l      Affiche les numéros de ligne dynamiques le long de la marge.\n\
     fichier Spécifie le nom d'un fichier à charger depuis le système VFS.\n\
\n\
TOUCHES DE CONTRÔLE\n\
     [F2]    Sauvegarde le tampon courant dans le système de fichiers.\n\
     [F3]    Copie le bloc sélectionné (ou la ligne courante) dans le presse-papier.\n\
     [F4]    Coupe le bloc sélectionné (ou la ligne courante) vers le presse-papier.\n\
     [F5]    Colle le contenu du presse-papier à la position actuelle du curseur.\n\
     [F6]    Pose ou efface un point d'ancrage pour délimiter un bloc de texte.\n\
     [ESC]   Quitte l'éditeur de texte.\n\
\n\
QBX v0.1.0                      Révision 0.2                      EDT(1)";

            pager::afficher_avec_pagination(texte_manuel);
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
            println!("CAT(1)                      Manuel de référence QBX                      CAT(1)");
            println!("NOM");
            println!("     cat - Afficher le contenu d'un fichier");
            println!("SYNOPSIS");
            println!("     cat <nom_fichier> [-ppp]");
            println!("DESCRIPTION");
            println!("     Recherche un fichier texte dans la mémoire virtuelle (VFS)");
            println!("     et imprime son contenu brut sur la sortie standard.");
            println!("OPTIONS");
            println!("     -ppp    Active la pagination page par page pour les fichiers longs.");
            println!("QBX v0.1.0                      Révision 0.2                      CAT(1)");
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
        "ctr" => {
            println!("CTR(1)                      Manuel de référence QBX                      CTR(1)");
            println!("NOM");
            println!("     ctr - Créer un répertoire");
            println!("SYNOPSIS");
            println!("     ctr <nom_repertoire>");
            println!("DESCRIPTION");
            println!("     Crée un nouveau répertoire vide dans le système de fichiers VFS.");
            println!("QBX v0.1.0                      Révision 0.1                      CTR(1)");
        }
            "cdr" => {
    println!("CDR(1)                      Manuel de référence QBX                      CDR(1)");
    println!("NOM");
    println!("     cdr - Changer de répertoire courant");
    println!("SYNOPSIS");
    println!("     cdr <chemin>");
    println!("DESCRIPTION");
    println!("     Permet de naviguer dans l'arborescence des dossiers du VFS.");
    println!("QBX v0.1.0                      Révision 0.1                      CDR(1)");
}
        inconnu => {
            println!("Aucune page de manuel pour : {}", inconnu);
            println!("Tapez 'mnl' sans argument pour voir la liste des commandes.");
        }
    }
}
