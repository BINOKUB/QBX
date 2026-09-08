// QBX - Commande afn (Afficher Informations Noyau)
// Fichier : src/commandes/afn.rs
// Description : Affiche le tampon des messages système enregistrés par le noyau

use crate::println;
use crate::journal::JOURNAL;

pub fn executer() {
    let entrees = JOURNAL.lock().lire_tous();

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
