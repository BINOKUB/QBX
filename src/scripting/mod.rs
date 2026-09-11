// QBX Scripting Engine - Interpréteur de scripts corrigé
// Fichier : src/scripting/mod.rs
// Description : Exécute les lignes de script sans provoquer de deadlock sur le Mutex du shell

use crate::fs;
use crate::println;

pub fn executer(args: &str) {
    let nom_script = args.trim();
    
    if nom_script.is_empty() {
        println!("Erreur : Aucun script spécifié. Utilisation : strt <nom_script>");
        return;
    }

    let contenu_octets = match fs::lire(nom_script) {
        Some(octets) => octets,
        None => {
            println!("Erreur : Impossible de trouver le fichier '{}'", nom_script);
            return;
        }
    };

    let contenu_str = match core::str::from_utf8(&contenu_octets) {
        Ok(s) => s,
        Err(_) => {
            println!("Erreur : Le fichier '{}' n'est pas un texte UTF-8 valide", nom_script);
            return;
        }
    };

    println!("--- [ Début du script : {} ] ---", nom_script);

    for ligne in contenu_str.lines() {
        let ligne_propre = ligne.trim();

        if ligne_propre.is_empty() || ligne_propre.starts_with('#') {
            continue;
        }

        println!("$ {}", ligne_propre);

        // Correction du deadlock : Utilisation d'une exécution découplée du verrou SHELL
        executer_ligne_script(ligne_propre);
    }

    println!("--- [ Fin du script ] ---");
}

// Fonction auxiliaire pour router les commandes de script sans verrouiller le Shell interactif
fn executer_ligne_script(entree: &str) {
    let parties: alloc::vec::Vec<&str> = entree.split_whitespace().collect();
    if parties.is_empty() {
        return;
    }

    let commande = parties[0];
    let argument = parties.get(1).copied().unwrap_or("");
    let args = &parties[1..];

    match commande {
        "echo" => {
            let reste_args = if entree.len() > 4 { entree[4..].trim() } else { "" };
            crate::commandes::echo::executer(reste_args);
        }
        "ls" => {
            let reste_args = if entree.len() > 2 { entree[2..].trim() } else { "" };
            crate::commandes::ls::executer(reste_args);
        }
        "cat" => {
            let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
            crate::commandes::cat::executer(reste_args);
        }
        "df" => {
            crate::commandes::df::executer();
        }
        "dsk" => {
            crate::commandes::dsk::executer();
        }
        _ => {
            println!("script: commande non supportée en mode script : '{}'", commande);
        }
    }
}
