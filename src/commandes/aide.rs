// QBX Centurion - Sommaire d'aide interactif
// Fichier : src/commandes/aide.rs

use crate::commandes::pager;

pub fn executer() {
    let texte_aide = "\
=== QBX Centurion : Sommaire des Commandes ===
[Système & Matériel]
  inf      : Informations système, architecture et CPU
  tmps     : Horloge et temps d'activité (uptime)
  tsk      : Lister les processus actifs
  mmr      : Consommation du tas mémoire (-e/-t/-c)
  ntr      : Nettoyer l'affichage console (clear)
  qtr      : Extinction matérielle du système (!)
[Système de Fichiers VFS]
  ls       : Lister le contenu du répertoire
  cat <f>  : Afficher le contenu d'un fichier
  ctr <f>  : Créer un fichier vide
  cdr <d>  : Changer de répertoire
  cpr <s> <d>: Copier un fichier
  dpc <s> <d>: Déplacer un fichier
  rnm <a> <n>: Renommer un fichier
  spp <f>  : Supprimer un fichier
  edt <f>  : Éditeur de texte interactif
[Réseau & Reconnaissance]
  pci      : Auditer les périphériques du bus matériel
  net      : Statut interface réseau e1000 et adresse MAC
  snf      : Interception de trames (mode Promiscuous)
  probe    : Sonde ARP furtive et balayage de segment
[Sécurité & Privilèges]
  su       : Élévation de privilèges (-adm, -arc, -d)
  exit     : Rétrogradation au mode Opérateur
  ver      : Verrouiller la session
  afn      : Journal d'audit et messages du noyau
  mnl <c>  : Manuel technique d'une commande
  initadm  : Définir la clé secrète Administrateur (#)
  initarch : Sceller la clé secrète Architecte (!)";

    pager::afficher_avec_pagination(texte_aide);
}
