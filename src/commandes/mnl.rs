// QBX Commande - Révision 1.0
// Fichier : src/commandes/mnl.rs
// Description : Manuel du système QBX avec support du pager et couverture complète des commandes

use crate::println;
use crate::commandes::pager;

pub fn executer(commande: &str) {
    let cmd = commande.trim();

    if cmd.is_empty() {
        println!("Usage : mnl <commande>");
        println!("Commandes disponibles :");
        println!("  ntr, inf, tmps, edt, ls, cat, echo, ctr, cdr, spp,");
        println!("  rnm, cpr, dpc, df, dsk, afn, mmr, ver, tpf, tsk,");
        println!("  su, pci, snf, mnl, qtr");
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
        "tsk" => {
            println!("TSK(1)                      Manuel de référence QBX                      TSK(1)\n");
            println!("NOM");
            println!("     tsk - Lister les tâches et processus du noyau\n");
            println!("SYNOPSIS");
            println!("     tsk\n");
            println!("DESCRIPTION");
            println!("     Interroge l'ordonnanceur coopératif pour afficher la table");
            println!("     des tâches enregistrées, incluant l'identifiant (TID),");
            println!("     l'état d'exécution (ACTIF, PRÊT, TERMINÉ) et le nom assigné.\n");
            println!("QBX v0.1.0                      Révision 0.1                      TSK(1)");
        }
        "su" => {
            println!("SU(1)                       Manuel de référence QBX                       SU(1)\n");
            println!("NOM");
            println!("     su - Gestion des niveaux de privilèges et d'autorité\n");
            println!("SYNOPSIS");
            println!("     su [-adm | -arc | -d]\n");
            println!("DESCRIPTION");
            println!("     Bascule le niveau d'autorité de la session courante après");
            println!("     vérification du mot de passe associé (saisie masquée par *).");
            println!("     Le symbole de l'invite de commande reflète le rang actif :\n");
            println!("       '>' Opérateur      Mode restreint de surveillance et d'exploitation.");
            println!("       '#' Administrateur Gestion avancée et opérations de défense (-adm).");
            println!("       '!' Architecte     Contrôle matériel, mémoire et noyau (-arc).\n");
            println!("OPTIONS");
            println!("     -adm    Élévation au palier Administrateur.");
            println!("     -arc    Élévation au palier Architecte.");
            println!("     -d      Rétrogradation immédiate d'un palier (identique à exit).\n");
            println!("QBX v0.1.0                      Révision 0.1                       SU(1)");
        }
        "pci" => {
            println!("PCI(1)                      Manuel de référence QBX                      PCI(1)\n");
            println!("NOM");
            println!("     pci - Inspection des périphériques connectés au bus matériel\n");
            println!("SYNOPSIS");
            println!("     pci\n");
            println!("DESCRIPTION");
            println!("     Interroge le bus PCI via les registres d'E/S 0xCF8 et 0xCFC.");
            println!("     Affiche la topologie matérielle (Bus, Périphérique, Fonction),");
            println!("     l'identifiant constructeur (Vendor ID), le produit (Device ID),");
            println!("     l'adresse de base mémoire (BAR0) et cible les interfaces réseau.\n");
            println!("QBX v0.1.0                      Révision 0.1                       PCI(1)");
        }
        "snf" => {
            println!("SNF(1)                      Manuel de référence QBX                      SNF(1)\n");
            println!("NOM");
            println!("     snf - Sonde de surveillance et d'interception réseau\n");
            println!("SYNOPSIS");
            println!("     snf\n");
            println!("DESCRIPTION");
            println!("     Active l'écoute des descripteurs DMA matériels de l'e1000.");
            println!("     Grâce au mode Promiscuous (UPE/MPE), capture et décode l'ensemble");
            println!("     des trames Ethernet (ARP, IPv4, IPv6) transitant sur le segment.\n");
            println!("QBX v0.1.0                      Révision 0.1                       SNF(1)");
        }
        "edt" => {
            let texte_manuel = r#"EDT(1)                      Manuel de référence QBX                      EDT(1)

NOM
     edt - Éditeur de texte plein écran

SYNOPSIS
     edt [-l] [fichier]

DESCRIPTION
     Edt est un éditeur de texte interactif conçu pour QBX Centurion.
     Il permet de créer, afficher et modifier des fichiers texte en mode
     plein écran (80x23) avec persistance directe sur le disque SATA (/var).

     Si aucun fichier n'est spécifié au démarrage, l'éditeur s'ouvre sur
     un espace de travail vierge et anonyme ([Sans nom]). Les modifications
     sont enregistrées sur le stockage persistant à la demande de l'utilisateur.

OPTIONS
     -l        Affiche les numéros de ligne dynamiques le long de la marge.
     fichier   Nom du fichier à charger ou créer depuis le système VFS.

TOUCHES DE CONTRÔLE
     [F2]      Sauvegarde le tampon sur le disque dur (/var).
     [F3]      Copie la ligne ou le bloc sélectionné dans le presse-papier.
     [F4]      Coupe la ligne ou le bloc sélectionné vers le presse-papier.
     [F5]      Colle le contenu du presse-papier à la position du curseur.
     [F6]      Pose ou efface un point d'ancrage pour délimiter un bloc.
     [ESC]     Quitte l'éditeur et revient au shell.

QBX v0.1.0                      Révision 0.2                      EDT(1)"#;

            pager::afficher_avec_pagination(texte_manuel);
        }
        "ls" => {
            println!("LS(1)                       Manuel de référence QBX                       LS(1)\n");
            println!("NOM");
            println!("     ls - Lister les fichiers et répertoires\n");
            println!("SYNOPSIS");
            println!("     ls\n");
            println!("DESCRIPTION");
            println!("     Parcourt et liste les éléments enregistrés dans le système de");
            println!("     fichiers persistant (VFS sur /var). Affiche le type, l'heure et la taille.\n");
            println!("QBX v0.1.0                      Révision 0.2                      LS(1)");
        }
        "cat" => {
            println!("CAT(1)                      Manuel de référence QBX                      CAT(1)\n");
            println!("NOM");
            println!("     cat - Afficher le contenu d'un fichier\n");
            println!("SYNOPSIS");
            println!("     cat <nom_fichier> [-ppp]\n");
            println!("DESCRIPTION");
            println!("     Recherche un fichier texte dans le système de fichiers (VFS)");
            println!("     et imprime son contenu brut sur la sortie standard.\n");
            println!("OPTIONS");
            println!("     -ppp    Active la pagination page par page pour les fichiers longs.\n");
            println!("QBX v0.1.0                      Révision 0.2                      CAT(1)");
        }
        "echo" => {
            println!("ECHO(1)                     Manuel de référence QBX                     ECHO(1)\n");
            println!("NOM");
            println!("     echo - Afficher du texte ou rediriger vers un fichier\n");
            println!("SYNOPSIS");
            println!("     echo [texte] [> fichier | >> fichier]\n");
            println!("DESCRIPTION");
            println!("     Affiche la chaîne passée en argument sur la sortie standard.");
            println!("     Supporte la redirection '>' (écrasement) et '>>' (ajout) vers le VFS.\n");
            println!("QBX v0.1.0                      Révision 0.1                     ECHO(1)");
        }
        "ctr" => {
            println!("CTR(1)                      Manuel de référence QBX                      CTR(1)\n");
            println!("NOM");
            println!("     ctr - Créer un répertoire\n");
            println!("SYNOPSIS");
            println!("     ctr <nom_repertoire>\n");
            println!("DESCRIPTION");
            println!("     Crée un nouveau répertoire vide dans le système de fichiers VFS.\n");
            println!("QBX v0.1.0                      Révision 0.1                      CTR(1)");
        }
        "cdr" => {
            println!("CDR(1)                      Manuel de référence QBX                      CDR(1)\n");
            println!("NOM");
            println!("     cdr - Changer de répertoire courant\n");
            println!("SYNOPSIS");
            println!("     cdr <chemin>\n");
            println!("DESCRIPTION");
            println!("     Permet de naviguer dans l'arborescence des dossiers du VFS.\n");
            println!("QBX v0.1.0                      Révision 0.1                      CDR(1)");
        }
        "spp" => {
            println!("SPP(1)                      Manuel de référence QBX                      SPP(1)\n");
            println!("NOM");
            println!("     spp - Supprimer un élément du système de fichiers\n");
            println!("SYNOPSIS");
            println!("     spp <nom_fichier>");
            println!("     spp -r <nom_repertoire>\n");
            println!("DESCRIPTION");
            println!("     Supprime définitivement un fichier ou un dossier du stockage persistant.");
            println!("     L'option '-r' est obligatoire pour détruire un répertoire.\n");
            println!("QBX v0.1.0                      Révision 0.2                      SPP(1)");
        }
        "rnm" => {
            println!("RNM(1)                      Manuel de référence QBX                      RNM(1)\n");
            println!("NOM");
            println!("     rnm - Renommer un élément\n");
            println!("SYNOPSIS");
            println!("     rnm <ancien_fichier> <nouveau_fichier>");
            println!("     rnm -r <ancien_rep> <nouveau_rep>\n");
            println!("DESCRIPTION");
            println!("     Renomme un fichier ou un répertoire dans le dossier courant.");
            println!("     L'option '-r' est requise pour renommer un répertoire.\n");
            println!("QBX v0.1.0                      Révision 0.2                      RNM(1)");
        }
        "cpr" => {
            println!("CPR(1)                      Manuel de référence QBX                      CPR(1)\n");
            println!("NOM");
            println!("     cpr - Copier un fichier ou un répertoire\n");
            println!("SYNOPSIS");
            println!("     cpr <source> <destination>");
            println!("     cpr -r <rep_source> <rep_destination>\n");
            println!("DESCRIPTION");
            println!("     Duplique un fichier ou une arborescence complète dans le VFS.");
            println!("     L'option '-r' est requise pour copier récursivement un dossier.\n");
            println!("QBX v0.1.0                      Révision 0.1                      CPR(1)");
        }
        "dpc" => {
            println!("DPC(1)                      Manuel de référence QBX                      DPC(1)\n");
            println!("NOM");
            println!("     dpc - Déplacer un élément\n");
            println!("SYNOPSIS");
            println!("     dpc <source> <destination>\n");
            println!("DESCRIPTION");
            println!("     Déplace un fichier ou un dossier vers un nouvel emplacement du VFS.\n");
            println!("QBX v0.1.0                      Révision 0.1                      DPC(1)");
        }
        "df" => {
            println!("DF(1)                       Manuel de référence QBX                       DF(1)\n");
            println!("NOM");
            println!("     df - Table des partitions LBA et points de montage\n");
            println!("SYNOPSIS");
            println!("     df\n");
            println!("DESCRIPTION");
            println!("     Affiche la table des partitions matérielles du disque SATA (/sec, /var),");
            println!("     les plages de blocs LBA allouées ainsi que l'état d'occupation.\n");
            println!("QBX v0.1.0                      Révision 0.1                       DF(1)");
        }
        "dsk" => {
            println!("DSK(1)                      Manuel de référence QBX                      DSK(1)\n");
            println!("NOM");
            println!("     dsk - Diagnostic et statut du contrôleur AHCI/SATA\n");
            println!("SYNOPSIS");
            println!("     dsk\n");
            println!("DESCRIPTION");
            println!("     Interroge le contrôleur AHCI sur le bus PCI et affiche les paramètres");
            println!("     du disque dur (modèle, numéro de série, secteurs LBA, capacité).\n");
            println!("QBX v0.1.0                      Révision 0.1                      DSK(1)");
        }
        "afn" => {
            println!("AFN(1)                      Manuel de référence QBX                      AFN(1)\n");
            println!("NOM");
            println!("     afn - Journal des messages noyau et d'audit\n");
            println!("SYNOPSIS");
            println!("     afn [-c]\n");
            println!("DESCRIPTION");
            println!("     Affiche l'ensemble des événements et messages consignés dans le");
            println!("     journal d'audit persistant (/var).\n");
            println!("OPTIONS");
            println!("     -c      Purge le journal d'audit physique (réservé Administrateur / Architecte).\n");
            println!("QBX v0.1.0                      Révision 0.2                      AFN(1)");
        }
        "mmr" => {
            println!("MMR(1)                      Manuel de référence QBX                      MMR(1)\n");
            println!("NOM");
            println!("     mmr - Statistiques et gestion de la mémoire vive (Heap)\n");
            println!("SYNOPSIS");
            println!("     mmr [-e | -t | -c]\n");
            println!("DESCRIPTION");
            println!("     Interroge l'allocateur global du noyau pour afficher l'état");
            println!("     en temps réel du tas (Heap) de QBX :");
            println!("       - Plage d'adresses virtuelles allouée");
            println!("       - Capacité totale configurée");
            println!("       - Espace actuellement consommé et pourcentage");
            println!("       - Espace mémoire libre disponible\n");
            println!("OPTIONS");
            println!("     -e      Alloue et mappe manuellement 1 Mio (1024 Ko) de pages");
            println!("             physiques supplémentaires pour agrandir le tas.");
            println!("     -t      Épreuve de saturation progressive par blocs de 64 Ko");
            println!("             pour contraindre la mémoire sous le seuil critique (< 256 Ko).");
            println!("     -c      Libère intégralement les blocs de l'épreuve de saturation");
            println!("             (-t) et restitue l'espace au tas.\n");
            println!("QBX v0.1.0                      Révision 0.3                      MMR(1)");
        }
        "ver" => {
            println!("VER(1)                      Manuel de référence QBX                      VER(1)\n");
            println!("NOM");
            println!("     ver - Identification, version et architecture système\n");
            println!("SYNOPSIS");
            println!("     ver [-asnmrv]\n");
            println!("DESCRIPTION");
            println!("     Restitue les informations d'identification du noyau QBX.");
            println!("     Sans argument, affiche un résumé synthétique.\n");
            println!("OPTIONS");
            println!("     -a      Affiche l'ensemble des champs d'identification.");
            println!("     -s      Affiche le nom du système (QBX).");
            println!("     -n      Affiche le nom d'hôte de la machine (qbx-box).");
            println!("     -r      Affiche le niveau de révision ou release.");
            println!("     -v      Affiche la description interne du noyau.");
            println!("     -m, -p  Affiche l'architecture matérielle cible (x86_64).\n");
            println!("QBX v0.1.0                      Révision 0.1                      VER(1)");
        }
        "tpf" => {
            println!("TPF(1)                      Manuel de référence QBX                      TPF(1)\n");
            println!("NOM");
            println!("     tpf - Test de défaut de page (Page Fault #PF)\n");
            println!("SYNOPSIS");
            println!("     tpf\n");
            println!("DESCRIPTION");
            println!("     Déclenche délibérément une violation d'accès mémoire par écriture");
            println!("     volatile sur une adresse virtuelle non mappée (0xdeadbeef).");
            println!("     Permet de valider l'interception matérielle par l'interruption 14 (#PF),");
            println!("     la lecture exacte du registre CR2 et la prévention du triple fault.\n");
            println!("QBX v0.1.0                      Révision 0.1                      TPF(1)");
        }
        "mnl" => {
            println!("MNL(1)                      Manuel de référence QBX                      MNL(1)\n");
            println!("NOM");
            println!("     mnl - Manuel de référence système\n");
            println!("SYNOPSIS");
            println!("     mnl <commande>\n");
            println!("DESCRIPTION");
            println!("     Affiche la page de manuel formatée d'une commande système.\n");
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
