// QBX Commande : ls (Lister)
// Fichier : src/commandes/ls.rs
// Description : Liste le contenu d'un répertoire avec support de -l, -h et cibles arborescentes

use crate::println;
use alloc::string::String;
use alloc::vec::Vec;

fn formater_taille(octets: usize, human_readable: bool) -> String {
    if !human_readable {
        let mut s = String::new();
        use core::fmt::Write;
        let _ = write!(s, "{:>8}", octets);
        return s;
    }

    let mut s = String::new();
    use core::fmt::Write;
    if octets < 1024 {
        let _ = write!(s, "{:>7}B", octets);
    } else if octets < 1024 * 1024 {
        let ko = octets / 1024;
        let frac = (octets % 1024) * 10 / 1024;
        if ko < 10 {
            let _ = write!(s, "{:>5}.{}K", ko, frac);
        } else {
            let _ = write!(s, "{:>6}K", ko);
        }
    } else {
        let mo = octets / (1024 * 1024);
        let frac = ((octets % (1024 * 1024)) * 10) / (1024 * 1024);
        if mo < 10 {
            let _ = write!(s, "{:>5}.{}M", mo, frac);
        } else {
            let _ = write!(s, "{:>6}M", mo);
        }
    }
    s
}

pub fn executer(arguments: &str) {
    let tokens: Vec<&str> = arguments.split_whitespace().collect();
    let mut mode_long = false;
    let mut human_readable = false;
    let mut chemin_cible = "";

    for token in tokens {
        if token.starts_with('-') && token.len() > 1 {
            for c in token[1..].chars() {
                match c {
                    'l' => mode_long = true,
                    'h' => human_readable = true,
                    'a' => {} // Prévu pour les fichiers cachés
                    _ => {}
                }
            }
        } else if chemin_cible.is_empty() {
            chemin_cible = token;
        }
    }

    let elements = match crate::fs::lister_cible(chemin_cible) {
        Ok(el) => el,
        Err("est_fichier") => {
            println!("ls: '{}' est un fichier.", chemin_cible);
            return;
        }
        Err("introuvable") => {
            println!("ls: impossible d'accéder à '{}': Aucun fichier ou dossier de ce type.", chemin_cible);
            return;
        }
        _ => {
            println!("ls: erreur lors de l'accès au répertoire.");
            return;
        }
    };

    if elements.is_empty() {
        println!("(Dossier vide)");
        return;
    }

    if mode_long {
        let total = elements.len();
        println!("total {}", total);

        for (nom, taille, est_dossier, h) in &elements {
            let (perms, liens) = if *est_dossier {
                ("drwxr-xr-x", 2)
            } else {
                ("-rw-r--r--", 1)
            };

            let str_taille = formater_taille(*taille, human_readable);

            println!(
                "{} {:>2} qbx qbx {} {:02}:{:02}:{:02} {}",
                perms,
                liens,
                str_taille,
                h.heure,
                h.minute,
                h.seconde,
                nom
            );
        }
    } else {
        for (nom, _taille, est_dossier, _h) in &elements {
            if *est_dossier {
                println!("{}/", nom);
            } else {
                println!("{}", nom);
            }
        }
    }
}
