# QBX (Québec UNIX) — EXP

Système d'exploitation expérimental 64-bit (`x86_64`) bare-metal écrit en Rust, conçu avec une philosophie UNIX québécoise : minimaliste, direct, strictement typé et autonome.

--
## Vue d'ensemble

QBX est développé directement sur le silicium sans dépendance à la bibliothèque standard (`#![no_std]`). Il intègre son propre allocateur de mémoire dynamique, un pilote VGA texte, une gestion clavier avec historique circulaire, un système de fichiers virtuel (VFS) hiérarchique en RAM et l'accès direct au matériel RTC CMOS.

```text
=== QBX - EXP (Québec UNIX) v0.1 ===
Initialisation du système...
Système prêt.

qbx:/> ls
--- Contenu du répertoire ---
[REP] 22:55:29     1 entrée   rep1
[FIC] 22:55:56    37 octets   dd.txt
-----------------------------
qbx:/> _

Caractéristiques actuelles
Architecture Bare-Metal : Cible x86_64-unknown-none, boot via bootloader et exécution sous QEMU.

Gestion mémoire : Allocateur dynamique de tas (Heap) de 2 Mio.

Système de fichiers virtuel (VFS) :

Structure arborescente en mémoire basée sur BTreeMap et nœuds typés (File vs Directory).

Support complet du répertoire de travail courant (CWD) avec chemins relatifs et absolus.

Horodatage matériel temps réel (RTC CMOS) intégré à chaque nœud (HH:MM:SS).

Typage strict et sécuritaire : les commandes de manipulation interdisent la confusion entre fichiers et dossiers (-r obligatoire pour les répertoires).

Shell interactif & Pilotes :

Historique de saisie navigable via les touches Flèche Haut / Flèche Bas.

Éditeur de texte interactif pleine page (edt).

Manuel du système embarqué (mnl).

## Lexique des commandes

```text
  ls              Lister le contenu du dossier (type, heure RTC, taille)
  ctr <nom>       Créer un répertoire
  cdr <chemin>    Changer de répertoire (ex: cdr rep1, cdr .., cdr /)
  spp <nom>       Supprimer un fichier (ou spp -r pour un dossier)
  rnm <a1> <a2>   Renommer un fichier (ou rnm -r pour un dossier)
  cat <fichier>   Afficher le contenu d'un fichier
  edt <fichier>   Éditeur de texte intégré
  tmps            Afficher l'heure matérielle (RTC CMOS)
  inf             Informations et état du système
  ntr             Nettoyer l'écran (clear)
  mnl [cmd]       Manuel système interactif
  qtr             Éteindre la machine




Feuille de route (Roadmap)
[x] VFS hiérarchique dynamique avec horodatage RTC

[x] Commandes de base avec typage strict (spp, rnm, ctr, cdr)

[ ] Commande de copie : cpr (cpr <source> <cible>, cpr -r <rep_src> <rep_dest>)

[ ] Commande de déplacement : dpc (dpc <source> <cible>)

[ ] Redirection des flux de sortie du shell vers un fichier (>, >>)

[ ] Pilote disque ATA PIO pour la persistance réelle sur bloc de stockage

[ ] Déploiement sur support bootable (clé USB) et installation sur machine dédiée

Compilation et exécution
Prérequis
Chaîne de compilation Rust Nightly

Composants : rust-src, llvm-tools-preview

cargo-bootimage

QEMU pour l'émulation locale

cargo install bootimage
rustup component add rust-src llvm-tools-preview

cargo bootimage
qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-qbx.bin

Licence
Distribué sous licence GPL-3.0. Consultez le fichier LICENSE pour plus de détails.
