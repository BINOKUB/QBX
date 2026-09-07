// QBX Commande : ls (Lister)
// Fichier : src/commandes/ls.rs
// Description : Liste les fichiers (taille en octets) et répertoires (décompte d'entrées) avec horodatage RTC

use crate::println;

pub fn executer() {
    let elements = crate::fs::lister();

    if elements.is_empty() {
        println!("(Dossier vide)");
        return;
    }

    println!("--- Contenu du répertoire ---");
    for (nom, taille, est_dossier, h) in elements {
        if est_dossier {
            let unite = if taille > 1 { "entrées" } else { "entrée " };
            println!(
                "[REP] {:02}:{:02}:{:02}  {:>3} {}  {}",
                h.heure, h.minute, h.seconde, taille, unite, nom
            );
        } else {
            println!(
                "[FIC] {:02}:{:02}:{:02}  {:>4} octets   {}",
                h.heure, h.minute, h.seconde, taille, nom
            );
        }
    }
    println!("-----------------------------");
}
