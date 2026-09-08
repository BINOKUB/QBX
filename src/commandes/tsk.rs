// QBX - Commande tsk (Gestion des tâches et processus)
// Fichier : src/commandes/tsk.rs
// Description : Affiche la table des tâches actives et leur état dans l'ordonnanceur

use crate::println;
use crate::task::{self, EtatTache};

pub fn executer() {
    let taches = task::lister_taches();

    println!("--- Tâches et processus QBX ---");
    println!("  TID   ÉTAT        NOM");

    for t in &taches {
        let etat_str = match t.etat {
            EtatTache::EnCours => "ACTIF   ",
            EtatTache::Prete => "PRÊT    ",
            EtatTache::Terminee => "TERMINÉ ",
        };
        println!("  {:<5} {:<11} {}", t.id, etat_str, t.nom);
    }

    println!("-------------------------------");
    println!("  Total : {} tâche(s)", taches.len());
}
