// QBX VFS - Révision 0.4
// Fichier : src/fs.rs
// Description : VFS RamDisk aligné sur les attentes de ls.rs, cat.rs et edt

use spin::Mutex;

// --- [STRUCTURE 1 : Fichier] ---
pub struct Fichier {
    pub nom: [u8; 32],
    pub nom_len: usize,
    pub contenu: [u8; 1024],
    pub taille: usize,
    pub utilise: bool,
}

// --- [STRUCTURE 2 : RamDisk] ---
pub struct RamDisk {
    pub fichiers: [Fichier; 10],
}

impl RamDisk {
    pub const fn new() -> Self {
        const FICHIER_VIDE: Fichier = Fichier {
            nom: [0; 32],
            nom_len: 0,
            contenu: [0; 1024],
            taille: 0,
            utilise: false,
        };
        RamDisk {
            fichiers: [FICHIER_VIDE; 10],
        }
    }

    // Méthode appelée directement par cat.rs : fs_guard.lire(nom_fichier)
    pub fn lire(&self, nom: &str) -> Option<(&[u8], usize)> {
        let nom_bytes = nom.as_bytes();
        for f in self.fichiers.iter() {
            if f.utilise && &f.nom[..f.nom_len] == nom_bytes {
                return Some((&f.contenu[..f.taille], f.taille));
            }
        }
        None
    }
}

pub static SYSTEME_FICHIERS: Mutex<RamDisk> = Mutex::new(RamDisk::new());
pub static FS: &Mutex<RamDisk> = &SYSTEME_FICHIERS;

// --- [FONCTIONS PUBLIQUES] ---

pub fn ecrire(nom: &str, contenu: &[u8]) -> bool {
    let mut fs = SYSTEME_FICHIERS.lock();
    let nom_bytes = nom.as_bytes();

    // 1. Mise à jour d'un fichier existant
    for f in fs.fichiers.iter_mut() {
        if f.utilise && &f.nom[..f.nom_len] == nom_bytes {
            let len = contenu.len().min(1024);
            f.contenu[..len].copy_from_slice(&contenu[..len]);
            f.taille = len;
            return true;
        }
    }

    // 2. Création d'un nouveau fichier
    for f in fs.fichiers.iter_mut() {
        if !f.utilise {
            let nlen = nom_bytes.len().min(32);
            f.nom[..nlen].copy_from_slice(&nom_bytes[..nlen]);
            f.nom_len = nlen;

            let clen = contenu.len().min(1024);
            f.contenu[..clen].copy_from_slice(&contenu[..clen]);
            f.taille = clen;
            f.utilise = true;
            return true;
        }
    }

    false
}

pub fn lire(nom: &str) -> Option<[u8; 1024]> {
    let fs = SYSTEME_FICHIERS.lock();
    let nom_bytes = nom.as_bytes();

    for f in fs.fichiers.iter() {
        if f.utilise && &f.nom[..f.nom_len] == nom_bytes {
            return Some(f.contenu);
        }
    }
    None
}
