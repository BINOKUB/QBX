// QBX Centurion - Test de Page Fault volontaire
// Fichier : src/commandes/tpf.rs

use crate::{println, session};

pub fn executer() {
    if !session::verifier_privilege(session::NiveauPrivilege::Architecte) {
        println!("tpf: opération réservée au niveau Architecte (EPERM)");
        crate::klog!("[SEC] Tentative non autorisée de déclenchement Page Fault");
        return;
    }

    println!("[QBX] Déclenchement volontaire d'un Page Fault sur 0xdeadbeef...");
    let ptr = 0xdead_beef as *mut u8;
    unsafe {
        core::ptr::write_volatile(ptr, 42);
    }
}
