// QBX Centurion - Commande echo
// Fichier : src/commandes/echo.rs
// Description : Affiche une ligne de texte (supporte ou ignore les guillemets)

use crate::println;

pub fn executer(texte: &str) {
    let net = texte.trim();
    let contenu = if (net.starts_with('"') && net.ends_with('"'))
        || (net.starts_with('\'') && net.ends_with('\''))
    {
        if net.len() >= 2 {
            &net[1..net.len() - 1]
        } else {
            ""
        }
    } else {
        net
    };

    println!("{}", contenu);
}
