// QBX - Commande afn (Afficher Informations Noyau)
// Fichier : src/commandes/afn.rs

use alloc::string::String;
use crate::println;
use crate::journal;
use crate::commandes::pager;
use crate::session::{verifier_privilege, NiveauPrivilege};

pub fn executer(args: &str) {
    let option = args.trim();

    if option == "-c" {
        // Seul l'Administrateur ou l'Architecte peut purger les journaux
        if !verifier_privilege(NiveauPrivilege::Administrateur) {
            println!("afn: EPERM - Privilèges insuffisants pour purger le journal.");
            return;
        }

        journal::purger();
        crate::klog!("[AUDIT] Journal système réinitialisé");
        println!("afn: Journal d'audit noyau purgé avec succès.");
        return;
    }

    let entrees = journal::JOURNAL.lock().lire_tous();

    if entrees.is_empty() {
        println!("afn: Aucun message dans le journal du noyau.");
        return;
    }

    // Si le journal est volumineux, concaténer et passer au téléavertisseur (pager)
    if entrees.len() > 20 {
        let mut sortie = String::new();
        sortie.push_str("--- Journal des messages du noyau QBX ---\n");
        for ligne in &entrees {
            sortie.push_str(ligne);
            sortie.push('\n');
        }
        sortie.push_str("-----------------------------------------");
        pager::afficher_avec_pagination(&sortie);
    } else {
        println!("--- Journal des messages du noyau QBX ---");
        for ligne in entrees {
            println!("{}", ligne);
        }
        println!("-----------------------------------------");
    }
}
