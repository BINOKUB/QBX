// QBX - Commande mmr (Statistiques Mémoire)
// Fichier : src/commandes/mmr.rs
// Description : Affiche l'occupation du Heap, supporte l'extension (-e) et la plage d'adresses virtuelles

use crate::println;
use crate::allocator::{self, HEAP_START};

pub fn executer(arguments: &str) {
    let args = arguments.trim();

    if args == "-e" {
        println!("[MMR] Demande d'extension du Heap de 1 Mio (1024 Ko)...");
        match allocator::etendre_heap(1024 * 1024) {
            Ok(nouvelle_taille) => {
                println!("[MMR] Extension réussie. Nouvelle capacité : {} Ko", nouvelle_taille / 1024);
            }
            Err(e) => {
                println!("[MMR] Erreur lors de l'extension : {}", e);
                return;
            }
        }
    }

    let (utilise, libre, total) = allocator::obtenir_statistiques();

    let fin_virtuelle = HEAP_START + total;
    let pourcentage = if total > 0 { (utilise * 100) / total } else { 0 };

    println!("--- Statistiques mémoire QBX (Heap) ---");
    println!("  Plage virtuelle : 0x{:x} - 0x{:x}", HEAP_START, fin_virtuelle);
    println!("  Capacité totale : {} Ko ({} octets)", total / 1024, total);
    println!("  Mémoire allouée : {} Ko ({} octets) [{}%]", utilise / 1024, utilise, pourcentage);
    println!("  Mémoire libre   : {} Ko ({} octets)", libre / 1024, libre);
    println!("---------------------------------------");
}
