// QBX - Commande ver 
// Fichier : src/commandes/ver.rs
// Description : Restitue les informations d'identification, de version et d'architecture

use crate::println;

const SYSNAME: &str = "QBX";
const NODENAME: &str = "qbx-box";
const RELEASE: &str = "0.1-EXP";
const VERSION: &str = "Microkernel Bare-Metal";
const MACHINE: &str = "x86_64";

pub fn executer(arguments: &str) {
    let args = arguments.trim();

    // Comportement par défaut sans arguments : affichage synthétique
    if args.is_empty() {
        println!("{} {} ({})", SYSNAME, RELEASE, MACHINE);
        return;
    }

    let mut flag_s = false;
    let mut flag_n = false;
    let mut flag_r = false;
    let mut flag_v = false;
    let mut flag_m = false;

    // Analyse des drapeaux style getopt BSD
    for mot in args.split_whitespace() {
        if mot.starts_with('-') {
            for c in mot[1..].chars() {
                match c {
                    'a' => {
                        flag_s = true;
                        flag_n = true;
                        flag_r = true;
                        flag_v = true;
                        flag_m = true;
                    }
                    's' => flag_s = true,
                    'n' => flag_n = true,
                    'r' => flag_r = true,
                    'v' => flag_v = true,
                    'm' | 'p' => flag_m = true,
                    _ => {
                        println!("ver: option invalide -- '{}'", c);
                        println!("usage: ver [-asnmrv]");
                        return;
                    }
                }
            }
        }
    }

    // Affichage des champs sélectionnés avec espacement
    let mut premier = true;
    let mut afficher_champ = |valeur: &str| {
        if !premier {
            crate::print!(" ");
        }
        crate::print!("{}", valeur);
        premier = false;
    };

    if flag_s { afficher_champ(SYSNAME); }
    if flag_n { afficher_champ(NODENAME); }
    if flag_r { afficher_champ(RELEASE); }
    if flag_v { afficher_champ(VERSION); }
    if flag_m { afficher_champ(MACHINE); }

    crate::println!();
}
