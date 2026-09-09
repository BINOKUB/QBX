// QBX Centurion - Commande aide
// Fichier : src/commandes/aide.rs
// Description : Sommaire complet des commandes avec pagination

// N'OUBLIE PS DE REMETTRE LE PAGER ET texte_aide!!!!'
use crate::commandes::pager;

pub fn executer() {
    let texte_aide = r#"=== QBX Centurion : Commandes disponibles ===

[Système & Matériel]
  inf           : Informations système, architecture et CPU
  tmps          : Horloge matérielle RTC
  tsk           : Liste des tâches et fils d'exécution
  mmr           : Consommation et allocation de la mémoire Heap
  ver           : Version et révision du noyau
  ntr           : Nettoyer l'affichage de l'écran
  qtr           : Arrêt sécurisé et extinction de la machine

[Stockage & Partitions]
  df            : Table des partitions LBA et points de montage (/sec, /var)
  dsk           : Diagnostic d'intégrité et géométrie du contrôleur SATA/AHCI

[Système de Fichiers]
  ls            : Lister le contenu du dossier actif
  cat <f>       : Afficher le contenu textuel d'un fichier
  echo <texte>  : Écrire du texte (supporte les redirections > et >>)
  ctr <d>       : Créer un sous-répertoire
  cdr <d>       : Changer de répertoire (ex: 'cdr ..' ou 'cdr /')
  cpr <s> <d>   : Copier un fichier ou dossier (-r pour récursif)
  dpc <s> <d>   : Déplacer un fichier ou dossier
  rnm <a> <n>   : Renommer un élément
  spp <f>       : Supprimer un fichier ou un dossier
  edt <f>       : Éditeur de texte plein écran interactif

[Sécurité, Audit & Privilèges]
  afn [-c]      : Consulter les logs d'audit noyau (-c pour purger [# / !])
  su <opt>      : Élévation de privilèges (-arc, -adm, ou -d pour déchoir)
  initarch      : Sceller la clé d'accès Architecte (!)
  initadm       : Configurer la clé Administrateur (#)
  exit          : Rétrograder la session en mode Opérateur (>)

[Réseau & Diagnostics]
  pci           : Lister les périphériques détectés sur le bus PCI
  net           : État et configuration de l'interface réseau
  snf           : Analyseur de trames réseau
  probe         : Émission de sondes réseau
  tpf           : Test et vérification de la pile système
  mnl <cmd>     : Manuel d'instructions détaillé d'une commande

Redirections de flux supportées : commande > fichier.txt (ou >>)"#;

    pager::afficher_avec_pagination(texte_aide);
}
