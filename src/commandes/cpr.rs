// QBX - Commande cpr (Copier)
// Fichier : src/commandes/cpr.rs
// Description : Utilitaire de copie inspiré de FreeBSD cp(1) pour QBX VFS

use alloc::vec::Vec;
use crate::fs;
use crate::println;

/// Point d'entrée de la commande cpr
/// Syntaxe supportée :
///   cpr [-r] <source> <cible>
pub fn executer(args: &[&str]) {
    let mut recursif = false;
    let mut cibles: Vec<&str> = Vec::new();

    // Analyse des drapeaux et arguments
    for arg in args {
        if *arg == "-r" || *arg == "-R" {
            recursif = true;
        } else if !arg.is_empty() {
            cibles.push(*arg);
        }
    }

    if cibles.len() < 2 {
        println!("Usage: cpr [-r] <source> <cible>");
        return;
    }

    let source = cibles[0];
    let destination = cibles[1];

    match fs::copier(source, destination, recursif) {
        Ok(()) => {},
        Err(err) => {
            println!("cpr: {}: {}", source, err.message());
        }
    }
}
