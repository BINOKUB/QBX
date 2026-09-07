// QBX VFS Node - Révision 0.1
// Fichier : src/fs/node.rs
// Description : Définition des structures de données pour l'arbre VFS (Fichiers et Dossiers hiérarchiques)

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

// --- [ÉNUMÉRATION 1 : Node] ---
// Description : Représente un élément du système de fichiers (soit un fichier de données, soit un dossier conteneur).
pub enum Node {
    File {
        data: Vec<u8>,
    },
    Directory {
        children: BTreeMap<String, Node>,
    },
}

impl Node {
    // --- [FONCTION 1.1 : nouveau_fichier] ---
    // Description : Instancie et retourne un nouveau nœud de type Fichier contenant un tableau d'octets.
    pub fn nouveau_fichier(data: Vec<u8>) -> Self {
        Node::File { data }
    }

    // --- [FONCTION 1.2 : nouveau_dossier] ---
    // Description : Instancie et retourne un nouveau nœud de type Dossier avec une table de correspondance vide.
    pub fn nouveau_dossier() -> Self {
        Node::Directory {
            children: BTreeMap::new(),
        }
    }

    // --- [FONCTION 1.3 : est_un_dossier] ---
    // Description : Retourne 'true' si le nœud actuel est un dossier, 'false' sinon.
    pub fn est_un_dossier(&self) -> bool {
        matches!(self, Node::Directory { .. })
    }

    // --- [FONCTION 1.4 : est_un_fichier] ---
    // Description : Retourne 'true' si le nœud actuel est un fichier, 'false' sinon.
    pub fn est_un_fichier(&self) -> bool {
        matches!(self, Node::File { .. })
    }
}
