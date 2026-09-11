// QBX Log Command
// Fichier : src/commandes/log.rs
// Description : Affiche le contenu du journal du noyau (tampon circulaire et persistance /var)

use crate::journal::JOURNAL;
use crate::println;

pub fn executer() {
    let journal = JOURNAL.lock();
    let messages = journal.lire_tous();

    if messages.is_empty() {
        println!("Journal du noyau : Aucun événement enregistré.");
        return;
    }

    println!("--- [ Journal du noyau QBX (Audit & Système) ] ---");
    for msg in messages {
        println!("{}", msg);
    }
    println!("--------------------------------------------------");
}
