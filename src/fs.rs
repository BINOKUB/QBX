// QBX VFS - Révision 0.5
// Fichier : src/fs.rs
// Description : Système de fichiers virtuel dynamique (Heap) sans limite de taille

use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

// --- [STRUCTURE 1 : Fichier] ---
// Description : Représente un fichier avec un nom et un contenu dynamiques.
pub struct Fichier {
    pub nom: String,
    pub contenu: Vec<u8>,
}

// --- [STRUCTURE 2 : RamDisk] ---
// Description : Conteneur dynamique de fichiers utilisant Vec.
pub struct RamDisk {
    pub fichiers: Vec<Fichier>,
}

impl RamDisk {
    // --- [FONCTION 2.1 : new] ---
    // Description : Initialise un RamDisk vide.
    pub const fn new() -> Self {
        RamDisk {
            fichiers: Vec::new(),
        }
    }

    // --- [FONCTION 2.2 : lire] ---
    // Description : Recherche un fichier et renvoie une référence vers son contenu (Slice).
    pub fn lire(&self, nom: &str) -> Option<&[u8]> {
        for f in self.fichiers.iter() {
            if f.nom == nom {
                return Some(&f.contenu);
            }
        }
        None
    }
}

// --- [STATIC 1 : SYSTEME_FICHIERS] ---
pub static SYSTEME_FICHIERS: Mutex<RamDisk> = Mutex::new(RamDisk::new());
pub static FS: &Mutex<RamDisk> = &SYSTEME_FICHIERS;

// --- [FONCTION 3 : ecrire] ---
// Description : Met à jour un fichier existant ou en crée un nouveau dynamiquement.
pub fn ecrire(nom: &str, contenu: &[u8]) -> bool {
    let mut fs = SYSTEME_FICHIERS.lock();

    // 1. Mise à jour d'un fichier existant
    for f in fs.fichiers.iter_mut() {
        if f.nom == nom {
            f.contenu = contenu.to_vec();
            return true;
        }
    }

    // 2. Création d'un nouveau fichier
    fs.fichiers.push(Fichier {
        nom: String::from(nom),
        contenu: contenu.to_vec(),
    });

    true
}

// --- [FONCTION 4 : lire] ---
// Description : Renvoie une copie du contenu (pour l'éditeur).
pub fn lire(nom: &str) -> Option<Vec<u8>> {
    let fs = SYSTEME_FICHIERS.lock();
    for f in fs.fichiers.iter() {
        if f.nom == nom {
            return Some(f.contenu.clone());
        }
    }
    None
}
