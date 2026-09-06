// QBX Commande - Révision 0.1
// Fichier : src/commandes/mnl.rs
// Description : Manuel du système (Page de manuel / man)

use crate::println;

pub fn executer(cible: &str) {
    match cible {
        "ntr" => {
            println!("NOM");
            println!("    ntr - Nettoyer l'écran");
            println!("\nDESCRIPTION");
            println!("    Efface le contenu de la console VGA et replace le curseur.");
        }
        "qtr" => {
            println!("NOM");
            println!("    qtr - Quitter le système");
            println!("\nDESCRIPTION");
            println!("    Déclenche l'extinction matérielle de la machine via le bus d'alimentation.");
        }
        "mnl" => {
            println!("NOM");
            println!("    mnl - Manuel du système");
            println!("\nDESCRIPTION");
            println!("    Affiche la documentation d'une commande. Usage: mnl <commande>");
        }
        "aide" => {
            println!("NOM");
            println!("    aide - Sommaire des commandes");
            println!("\nDESCRIPTION");
            println!("    Affiche le lexique court des commandes disponibles dans QBX.");
        }
        "" => {
            println!("Usage : mnl <commande> (Exemple : mnl ntr)");
        }
        cmd => {
            println!("Pas de page de manuel pour '{}'", cmd);
        }
    }
}
