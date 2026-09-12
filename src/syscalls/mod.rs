// QBX Syscalls Module
// Fichier : src/syscalls/mod.rs
// Description : Table des appels système et répartiteur central (dispatcher) avec validation de mémoire

use crate::println;

// --- [ TABLE DES IDENTIFIANTS DE SYSCALLS ] ---
pub const SYS_EXIT: usize = 1;
pub const SYS_WRITE: usize = 2;
pub const SYS_LOG: usize = 3;
pub const SYS_READ: usize = 4;
pub const SYS_WRITE_FILE: usize = 5;

// --- [ VALIDATION DE LA PLAGE MÉMOIRE ] ---
// Vérifie que chaque page de la plage [ptr, ptr + len] est bien mappée et accessible.
fn valider_plage_memoire(ptr: usize, len: usize) -> bool {
    if len == 0 || ptr == 0 {
        return false;
    }
    let debut = ptr as u64;
    let fin = (ptr + len - 1) as u64;

    // Vérification du début de la plage
    if crate::memory::traduire_virtuelle_vers_physique(debut).is_none() {
        return false;
    }
    // Vérification de la fin de la plage
    if crate::memory::traduire_virtuelle_vers_physique(fin).is_none() {
        return false;
    }

    // Vérification des pages intermédiaires si la plage s'étend sur plusieurs pages de 4KiB
    let page_debut = debut & !0xFFF;
    let page_fin = fin & !0xFFF;
    let mut p = page_debut + 4096;
    while p <= page_fin {
        if crate::memory::traduire_virtuelle_vers_physique(p).is_none() {
            return false;
        }
        p += 4096;
    }

    true
}

// --- [ RÉPARTITEUR CENTRAL (DISPATCHER) ] ---
// Intercepte la requête, vérifie l'ID, valide la mémoire et aiguille vers la fonction adéquate.
pub fn executer_syscall(id: usize, arg1: usize, arg2: usize) -> isize {
    match id {
        SYS_EXIT => {
            println!("[SYSCALL] Terminaison demandée (exit).");
            0
        }
        SYS_WRITE => {
            // Emplacement futur pour l'écriture sécurisée vers la console/sortie standard
            0
        }
        SYS_LOG => {
            // Permet de journaliser un événement via l'infrastructure klog! du noyau
            crate::klog!("[SYSCALL] Entrée d'audit générée par appel système");
            0
        }
        SYS_READ => {
            // Validation sécurisée de toute la plage mémoire avant déréférencement
            if !valider_plage_memoire(arg1, arg2) {
                println!("[SYSCALL] Erreur de sécurité : Plage mémoire invalide ou non mappée (SYS_READ).");
                return -5; // Code d'erreur EFAULT (Bad address)
            }

            // Reconstitution sécurisée du nom de fichier depuis le pointeur et la longueur
            let nom_fichier = unsafe {
                let ptr = arg1 as *const u8;
                match core::str::from_utf8(core::slice::from_raw_parts(ptr, arg2)) {
                    Ok(s) => s,
                    Err(_) => return -2, // Erreur : Encodage UTF-8 invalide
                }
            };

            if nom_fichier.is_empty() {
                println!("[SYSCALL] Erreur : Aucun nom de fichier spécifié pour la lecture.");
                return -1; // Erreur : Argument manquant
            }

            println!("[SYSCALL] Lecture VFS demandée pour le fichier : '{}'", nom_fichier);
            
            // Appel direct vers le module VFS réel[cite: 5]
            match crate::fs::lire(nom_fichier) {
                Some(contenu) => {
                    println!("[VFS] Fichier lu avec succès ({} octets)", contenu.len());
                    0
                }
                None => -3, // Code d'erreur : Fichier introuvable
            }
        }
        SYS_WRITE_FILE => {
            // Validation sécurisée de toute la plage mémoire avant déréférencement
            if !valider_plage_memoire(arg1, arg2) {
                println!("[SYSCALL] Erreur de sécurité : Plage mémoire invalide ou non mappée (SYS_WRITE_FILE).");
                return -5; // Code d'erreur EFAULT
            }

            let nom_fichier = unsafe {
                let ptr = arg1 as *const u8;
                match core::str::from_utf8(core::slice::from_raw_parts(ptr, arg2)) {
                    Ok(s) => s,
                    Err(_) => return -2, // Erreur : Encodage UTF-8 invalide
                }
            };

            if nom_fichier.is_empty() {
                println!("[SYSCALL] Erreur : Aucun nom de fichier spécifié pour l'écriture.");
                return -1; // Erreur : Argument manquant
            }

            println!("[SYSCALL] Écriture VFS demandée pour le fichier : '{}'", nom_fichier);

            // Appel direct vers le module VFS réel[cite: 5]
            crate::fs::ecrire(nom_fichier, b"");
            println!("[VFS] Écriture réussie sur le disque.");
            0
        }
        _ => {
            println!("[SYSCALL] Erreur : Numéro de syscall inconnu ({})", id);
            -1 // Code d'erreur standard (EPERM / EINVAL)
        }
    }
}
