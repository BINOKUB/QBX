// QBX - Commande mmr (Statistiques Mémoire et Contrôle Matériel)
// Fichier : src/commandes/mmr.rs
// Description : Affiche l'occupation du Heap (lecture seule libre).
//               Verrouille l'extension (-e), la saturation (-t) et la libération (-c)
//               au niveau exclusif 'Architecte' (su -arc).

use crate::println;
use crate::allocator::{self, HEAP_START};
use crate::session::{self, NiveauPrivilege};
use spin::Mutex;
use alloc::vec::Vec;

// Tampon statique pour maintenir la charge mémoire active sans passer par le VFS
static TAMPON_SATURATION: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());

pub fn executer(arguments: &str) {
    let args = arguments.trim();

    // 1. Barrière de sécurité Superutilisateur : réservé au rang Architecte
    if args == "-e" || args == "-t" || args == "-c" {
        if !session::verifier_privilege(NiveauPrivilege::Architecte) {
            println!("mmr: opération réservée au niveau Architecte (EPERM)");
            crate::klog!("[SEC] Tentative non autorisée d'altération mémoire avec '{}'", args);
            return;
        }
    }

    // 2. Traitement des commutateurs privilégiés
    if args == "-e" {
        println!("[MMR] Demande manuelle d'extension du Heap de 1 Mio (1024 Ko)...");
        match allocator::etendre_heap(1024 * 1024) {
            Ok(nouvelle_taille) => {
                println!("[MMR] Extension réussie. Nouvelle capacité : {} Ko", nouvelle_taille / 1024);
            }
            Err(e) => {
                println!("[MMR] Erreur lors de l'extension : {}", e);
                return;
            }
        }
    } else if args == "-t" {
        let mut tampon = TAMPON_SATURATION.lock();

        if !tampon.is_empty() {
            println!("[MMR] Test déjà actif. Tapez 'mmr -c' pour libérer la mémoire.");
        } else {
            println!("[MMR] Saturation progressive par blocs de 64 Ko...");
            let mut alloue = 0;

            loop {
                let (_, libre, _) = allocator::obtenir_statistiques();
                if libre <= 200 * 1024 {
                    break;
                }

                let mut bloc = Vec::new();
                bloc.resize(64 * 1024, 0xAA);
                tampon.push(bloc);
                alloue += 64 * 1024;
            }

            println!("[MMR] {} Ko alloués. Seuil critique atteint (< 256 Ko libres).", alloue / 1024);
            println!("[MMR] Exécutez la commande suivante pour déclencher l'auto-expansion.");
        }
    } else if args == "-c" {
        let mut tampon = TAMPON_SATURATION.lock();
        let nb_blocs = tampon.len();
        tampon.clear();
        println!("[MMR] Libération terminée : {} Ko restitués au tas.", nb_blocs * 64);
    }

    // 3. Affichage standard des métriques mémoire (accessible à tous les rôles)
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
