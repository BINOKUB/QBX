// QBX Commande : rnm (Renommer)
// Fichier : src/commandes/rnm.rs
// Description : Renomme un fichier, ou un répertoire si l'option -r est présente avec contrôle RBAC

use alloc::vec::Vec;
use crate::println;
use crate::session::{verifier_privilege, NiveauPrivilege};

pub fn executer(arguments: &str) {
    let args: Vec<&str> = arguments.split_whitespace().collect();

    let (ancien, nouveau, dossier_attendu) = match args.len() {
        2 => {
            if args[0] == "-r" || args[1] == "-r" {
                println!("Utilisation : rnm -r <ancien_rep> <nouveau_rep>");
                return;
            }
            (args[0], args[1], false)
        }
        3 => {
            if args[0] == "-r" {
                (args[1], args[2], true)
            } else {
                println!("Paramètre invalide : '{}'.", args[0]);
                println!("Utilisation : rnm <ancien_fichier> <nouveau_fichier> ou rnm -r <ancien_rep> <nouveau_rep>");
                return;
            }
        }
        _ => {
            println!("Utilisation :");
            println!("  rnm <ancien_fichier> <nouveau_fichier>");
            println!("  rnm -r <ancien_rep> <nouveau_rep>");
            return;
        }
    };

    if ancien.contains('/') || nouveau.contains('/') {
        println!("Erreur : Les noms ne peuvent pas contenir le caractère '/'.");
        return;
    }

    // Contrôle RBAC : le renommage de répertoires (-r) exige le rang Administrateur (#) ou Architecte (!)
    if dossier_attendu && !verifier_privilege(NiveauPrivilege::Administrateur) {
        crate::klog!("[AUDIT] rnm: tentative non autorisée de renommage du répertoire '{}' (EPERM)", ancien);
        println!("rnm: EPERM - Privilèges insuffisants pour renommer un répertoire (Administrateur requis).");
        return;
    }

    match crate::fs::renommer(ancien, nouveau, dossier_attendu) {
        Ok(()) => {
            if dossier_attendu {
                crate::klog!("[VFS] Répertoire '{}' renommé en '{}'", ancien, nouveau);
                println!("Répertoire '{}' renommé en '{}'.", ancien, nouveau);
            } else {
                crate::klog!("[VFS] Fichier '{}' renommé en '{}'", ancien, nouveau);
                println!("Fichier '{}' renommé en '{}'.", ancien, nouveau);
            }
        }
        Err("est_dossier") => {
            println!("Erreur : '{}' est un répertoire. Utilisez 'rnm -r {} {}' pour le renommer.", ancien, ancien, nouveau);
        }
        Err("est_fichier") => {
            println!("Erreur : '{}' est un fichier. Utilisez 'rnm {} {}' sans l'option '-r'.", ancien, ancien, nouveau);
        }
        Err("existe_deja") => {
            println!("Erreur : '{}' existe déjà.", nouveau);
        }
        Err("introuvable") => {
            println!("Erreur : '{}' introuvable.", ancien);
        }
        _ => {
            println!("Erreur : Impossible d'accéder au répertoire courant.");
        }
    }
}
