// QBX - Commande afn (Afficher Informations Noyau)
// Fichier : src/commandes/afn.rs

use crate::println;
use crate::journal;
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

    println!("--- Journal des messages du noyau QBX ---");
    for ligne in entrees {
        println!("{}", ligne);
    }
    println!("-----------------------------------------");
}
