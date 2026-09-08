// QBX VFS Node - Révision 0.2
// Fichier : src/fs/node.rs
// Description : Définition des structures de données pour l'arbre VFS avec horodatage

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub struct Horodatage {
    pub heure: u8,
    pub minute: u8,
    pub seconde: u8,
}

impl Horodatage {
    pub fn new(heure: u8, minute: u8, seconde: u8) -> Self {
        Horodatage { heure, minute, seconde }
    }

    pub fn zero() -> Self {
        Horodatage { heure: 0, minute: 0, seconde: 0 }
    }
}


#[derive(Clone)]


pub enum Node {
    File {
        data: Vec<u8>,
        horodatage: Horodatage,
    },
    Directory {
        children: BTreeMap<String, Node>,
        horodatage: Horodatage,
    },
}

impl Node {
    pub fn nouveau_fichier(data: Vec<u8>, horodatage: Horodatage) -> Self {
        Node::File { data, horodatage }
    }

    pub fn nouveau_dossier(horodatage: Horodatage) -> Self {
        Node::Directory {
            children: BTreeMap::new(),
            horodatage,
        }
    }

    pub fn est_un_dossier(&self) -> bool {
        matches!(self, Node::Directory { .. })
    }

    pub fn est_un_fichier(&self) -> bool {
        matches!(self, Node::File { .. })
    }

    pub fn horodatage(&self) -> Horodatage {
        match self {
            Node::File { horodatage, .. } => *horodatage,
            Node::Directory { horodatage, .. } => *horodatage,
        }
    }
}
