// QBX Centurion - Diagnostic et test du stockage persistant
// Fichier : src/commandes/dsk.rs

use crate::{print, println, storage::ahci};

pub fn executer() {
    println!("--- Test d'intégrité E/S Disque SATA (LBA 100) ---");

    let mut tampon_ecriture = [0u8; 512];
    let message = b"QBX CENTURION SECURE STORAGE PERSISTENCE LAYER OK";
    tampon_ecriture[..message.len()].copy_from_slice(message);

    print!("[DSK] Écriture sur LBA 100... ");
    match ahci::ecrire_secteur(100, &tampon_ecriture) {
        Ok(()) => println!("SUCCÈS"),
        Err(e) => {
            println!("ÉCHEC ({})", e);
            return;
        }
    }

    let mut tampon_lecture = [0u8; 512];
    print!("[DSK] Relecture LBA 100... ");
    match ahci::lire_secteur(100, &mut tampon_lecture) {
        Ok(()) => {
            println!("SUCCÈS");
            let lu = core::str::from_utf8(&tampon_lecture[..message.len()]).unwrap_or("<invalide>");
            println!("      Données relues : \"{}\"", lu);
            if lu == "QBX CENTURION SECURE STORAGE PERSISTENCE LAYER OK" {
                println!("[DSK] Persistance physique vérifiée à 100%.");
            } else {
                println!("[DSK] Erreur : les données lues ne correspondent pas.");
            }
        }
        Err(e) => println!("ÉCHEC ({})", e),
    }
    println!("--------------------------------------------------");
}
