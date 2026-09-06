// QBX System - Révision 0.1
// Fichier : src/fs.rs
// Description : Système de fichiers virtuel en mémoire RAM (VFS RAMDisk)

use spin::Mutex;

const MAX_FICHIERS: usize = 10;
const TAILLE_FICHIER_MAX: usize = 8192; // 8 Ko par fichier

// --- [STRUCTURE 1 : FichierVirtuel] ---
// Description : Représentation d'un fichier binaire/texte en mémoire vive.
#[derive(Clone, Copy)]
pub struct FichierVirtuel {
    pub nom: [u8; 32],
    pub nom_len: usize,
    pub contenu: [u8; TAILLE_FICHIER_MAX],
    pub taille: usize,
    pub utilise: bool,
}

impl FichierVirtuel {
    // --- [FONCTION 1.1 : vide] ---
    pub const fn vide() -> Self {
        FichierVirtuel {
            nom: [0; 32],
            nom_len: 0,
            contenu: [0; TAILLE_FICHIER_MAX],
            taille: 0,
            utilise: false,
        }
    }
}

// --- [STRUCTURE 2 : RamDisk] ---
// Description : Table d'allocation mémoire regroupant les fichiers du système.
pub struct RamDisk {
    pub fichiers: [FichierVirtuel; MAX_FICHIERS],
}

impl RamDisk {
    // --- [FONCTION 2.1 : new] ---
    pub const fn new() -> Self {
        RamDisk {
            fichiers: [FichierVirtuel::vide(); MAX_FICHIERS],
        }
    }

    // --- [FONCTION 2.2 : ecrire] ---
    // Description : Crée ou écrase un fichier du RamDisk par son nom.
    pub fn ecrire(&mut self, nom: &str, donnees: &[u8]) -> bool {
        let nom_bytes = nom.as_bytes();
        let len_nom = nom_bytes.len().min(32);

        // 1. Recherche d'un fichier existant avec le même nom pour écrasement
        for f in self.fichiers.iter_mut() {
            if f.utilise && f.nom_len == len_nom && &f.nom[..len_nom] == &nom_bytes[..len_nom] {
                let copié = donnees.len().min(TAILLE_FICHIER_MAX);
                f.contenu[..copié].copy_from_slice(&donnees[..copié]);
                f.taille = copié;
                return true;
            }
        }

        // 2. Création dans un nouvel emplacement disponible
        for f in self.fichiers.iter_mut() {
            if !f.utilise {
                f.nom = [0; 32];
                f.nom[..len_nom].copy_from_slice(&nom_bytes[..len_nom]);
                f.nom_len = len_nom;
                
                let copié = donnees.len().min(TAILLE_FICHIER_MAX);
                f.contenu[..copié].copy_from_slice(&donnees[..copié]);
                f.taille = copié;
                f.utilise = true;
                return true;
            }
        }

        false // Plus de place dans la table
    }

    // --- [FONCTION 2.3 : lire] ---
    // Description : Extrait le contenu d'un fichier par son nom.
    pub fn lire(&self, nom: &str) -> Option<(&[u8], usize)> {
        let nom_bytes = nom.as_bytes();
        let len_nom = nom_bytes.len().min(32);

        for f in self.fichiers.iter() {
            if f.utilise && f.nom_len == len_nom && &f.nom[..len_nom] == &nom_bytes[..len_nom] {
                return Some((&f.contenu[..f.taille], f.taille));
            }
        }
        None
    }
}

// --- [STATIC 1 : SYSTEME_FICHIERS] ---
pub static SYSTEME_FICHIERS: Mutex<RamDisk> = Mutex::new(RamDisk::new());
