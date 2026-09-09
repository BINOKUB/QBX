// QBX Centurion - Extinction Matérielle
// Fichier : src/commandes/qtr.rs

use crate::{println, power, session};

pub fn executer() {
    if !session::verifier_privilege(session::NiveauPrivilege::Architecte) {
        println!("qtr: extinction réservée au niveau Architecte (EPERM)");
        crate::klog!("[SEC] Tentative d'extinction non autorisée");
        return;
    }

    println!("[QBX] Extinction du système...");
    power::eteindre();
}
