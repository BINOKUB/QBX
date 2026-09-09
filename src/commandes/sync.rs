// QBX Centurion - Commande sync
// Fichier : src/commandes/sync.rs
// Description : Forcer le vidage des tampons mémoires et la persistance immédiate sur /var

use crate::println;
use crate::fs;

pub fn executer() {
    fs::synchroniser();
    crate::klog!("[SYNC] Synchronisation forcée des tampons VFS vers SATA (/var)");
    println!("sync: Synchronisation terminée. Tampons VFS vidés vers le contrôleur SATA (/var).");
}
