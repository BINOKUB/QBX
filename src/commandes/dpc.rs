// QBX - Commande dpc (Déplacer)
// Fichier : src/commandes/dpc.rs
// Description : Utilitaire de déplacement de fichiers et répertoires pour QBX VFS

use alloc::vec::Vec;
use crate::fs;
use crate::println;

/// Point d'entrée de la commande dpc
/// Syntaxe supportée :
///   dpc <source> <cible>
pub fn executer(args: &[&str]) {
    let mut cibles: Vec<&str> = Vec::new();

    for arg in args {
        if !arg.is_empty() {
            cibles.push(*arg);
        }
    }

    if cibles.len() < 2 {
        println!("Usage: dpc <source> <cible>");
        return;
    }

    let source = cibles[0];
    let destination = cibles[1];

    match fs::deplacer(source, destination) {
        Ok(()) => {},
        Err(err) => {
            println!("dpc: {}: {}", source, err.message());
        }
    }
}
