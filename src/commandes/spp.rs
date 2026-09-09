// QBX Commande : spp (Supprimer)
// Fichier : src/commandes/spp.rs
// Description : Supprime un fichier, ou un répertoire si l'option -r est présente avec contrôle RBAC

use alloc::vec::Vec;
use crate::println;
use crate::session::{verifier_privilege, NiveauPrivilege};

pub fn executer(arguments: &str) {
    let args: Vec<&str> = arguments.split_whitespace().collect();

    let (nom, dossier_attendu) = match args.len() {
        1 => {
            if args[0] == "-r" {
                println!("Erreur : Spécifiez le nom du répertoire après '-r'.");
                println!("Utilisation : spp [-r] <nom>");
                return;
            }
            (args[0], false)
        }
        2 => {
            if args[0] == "-r" {
                (args[1], true)
            } else if args[1] == "-r" {
                (args[0], true)
            } else {
                println!("Erreur : Paramètre inconnu '{}'.", args[0]);
                println!("Utilisation : spp [-r] <nom>");
                return;
            }
        }
        _ => {
            println!("Utilisation : spp <fichier> ou spp -r <repertoire>");
            return;
        }
    };

   // if nom.contains('/') {
   //     println!("Erreur : Le nom ne peut pas contenir le caractère '/'.");
    //    return;
  //  }

    // Contrôle RBAC : la suppression de répertoires (-r) exige le rang Administrateur (#) ou Architecte (!)
    if dossier_attendu && !verifier_privilege(NiveauPrivilege::Administrateur) {
        crate::klog!("[AUDIT] spp: tentative non autorisée de suppression du répertoire '{}' (EPERM)", nom);
        println!("spp: EPERM - Privilèges insuffisants pour supprimer un répertoire (Administrateur requis).");
        return;
    }

    match crate::fs::supprimer(nom, dossier_attendu) {
        Ok(()) => {
            if dossier_attendu {
                crate::klog!("[VFS] Répertoire '{}' supprimé", nom);
                println!("Répertoire '{}' supprimé avec succès.", nom);
            } else {
                crate::klog!("[VFS] Fichier '{}' supprimé", nom);
                println!("Fichier '{}' supprimé avec succès.", nom);
            }
        }
        Err("est_dossier") => {
            println!("Erreur : '{}' est un répertoire. Utilisez 'spp -r {}' pour le supprimer.", nom, nom);
        }
        Err("est_fichier") => {
            println!("Erreur : '{}' est un fichier. Utilisez 'spp {}' sans l'option '-r'.", nom, nom);
        }
        Err("introuvable") => {
            println!("Erreur : '{}' introuvable.", nom);
        }
        _ => {
            println!("Erreur : Impossible d'accéder au répertoire courant.");
        }
    }
}
