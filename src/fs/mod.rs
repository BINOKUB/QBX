// QBX FileSystem Module - Révision 0.4
// Fichier : src/fs/mod.rs
// Description : Façade centrale du système de fichiers hiérarchique avec support dynamique du CWD, suppression et renommage stricts[cite: 5]

pub mod node;
pub mod path;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use node::Node;

// --- [STRUCTURE 1 : FileSystem] ---
// Description : Représente l'état global du système de fichiers hiérarchique en mémoire (RAMDisk).
pub struct FileSystem {
    root: Node,
    cwd: Vec<String>, // Répertoire courant (Current Working Directory)
}

impl FileSystem {
    // --- [FONCTION 1.1 : new] ---
    // Description : Initialise un nouveau système de fichiers avec un répertoire racine vide.
    pub const fn new() -> Self {
        FileSystem {
            root: Node::Directory {
                children: BTreeMap::new(),
            },
            cwd: Vec::new(),
        }
    }

    // --- [FONCTION 1.2 : ecrire] ---
    // Description : Crée ou met à jour un fichier dans le répertoire courant (CWD).
    pub fn ecrire(&mut self, nom: &str, donnees: Vec<u8>) -> bool {
        let cwd_clone = self.cwd.clone();
        if let Some(Node::Directory { children }) = self.acceder_noeud_mut(&cwd_clone) {
            children.insert(String::from(nom), Node::nouveau_fichier(donnees));
            return true;
        }
        false
    }

    // --- [FONCTION 1.3 : lire] ---
    // Description : Lit un fichier depuis le répertoire courant (CWD).
    pub fn lire(&self, nom: &str) -> Option<Vec<u8>> {
        if let Some(Node::Directory { children }) = self.acceder_noeud(&self.cwd) {
            if let Some(Node::File { data }) = children.get(nom) {
                return Some(data.clone());
            }
        }
        None
    }

    // --- [FONCTION 1.4 : lister_courant] ---
    // Description : Retourne la liste des éléments présents dans le répertoire courant (CWD).
    pub fn lister_courant(&self) -> Vec<(String, usize)> {
        let mut resultats = Vec::new();
        if let Some(Node::Directory { children }) = self.acceder_noeud(&self.cwd) {
            for (nom, node) in children {
                let taille = match node {
                    Node::File { data } => data.len(),
                    Node::Directory { children } => children.len(),
                };
                resultats.push((nom.clone(), taille));
            }
        }
        resultats
    }

    // --- [FONCTION 1.5 : creer_repertoire] ---
    // Description : Crée un nouveau dossier dans le répertoire courant (CWD).
    pub fn creer_repertoire(&mut self, nom: &str) -> bool {
        if nom.is_empty() {
            return false;
        }
        let cwd_clone = self.cwd.clone();
        if let Some(Node::Directory { children }) = self.acceder_noeud_mut(&cwd_clone) {
            if !children.contains_key(nom) {
                children.insert(String::from(nom), Node::nouveau_dossier());
                return true;
            }
        }
        false
    }

    // --- [FONCTION 1.6 : changer_repertoire] ---
    // Description : Modifie le répertoire courant (CWD) en fonction d'un chemin relatif ou absolu.
    pub fn changer_repertoire(&mut self, chemin: &str) -> bool {
        let elements = path::nettoyer_chemin(chemin);
        
        let mut cible = if path::est_absolu(chemin) {
            Vec::new()
        } else {
            self.cwd.clone()
        };

        for elem in elements {
            if elem == ".." {
                cible.pop();
            } else {
                cible.push(elem);
            }
        }

        if self.acceder_noeud(&cible).is_some() {
            self.cwd = cible;
            return true;
        }

        false
    }

    // --- [FONCTION 1.7 : acceder_noeud (interne immutable)] ---
    fn acceder_noeud(&self, chemin: &[String]) -> Option<&Node> {
        let mut courant = &self.root;
        for dossier in chemin {
            if let Node::Directory { children } = courant {
                if let Some(enfant) = children.get(dossier) {
                    courant = enfant;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        Some(courant)
    }

    // --- [FONCTION 1.8 : acceder_noeud_mut (interne mutable)] ---
    fn acceder_noeud_mut(&mut self, chemin: &[String]) -> Option<&mut Node> {
        let mut courant = &mut self.root;
        for dossier in chemin {
            if let Node::Directory { children } = courant {
                if let Some(enfant) = children.get_mut(dossier) {
                    courant = enfant;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        Some(courant)
    }

    // --- [FONCTION 1.9 : obtenir_chemin_actuel] ---
    // Description : Retourne le chemin textuel complet du répertoire courant (ex: /monrep).
    pub fn obtenir_chemin_actuel(&self) -> String {
        if self.cwd.is_empty() {
            String::from("/")
        } else {
            let mut s = String::new();
            for dossier in &self.cwd {
                s.push('/');
                s.push_str(dossier);
            }
            s
        }
    }

    // --- [FONCTION 1.10 : supprimer] ---
    // Description : Supprime un fichier ou un dossier selon le type explicitement attendu.
    pub fn supprimer(&mut self, nom: &str, dossier_attendu: bool) -> Result<(), &'static str> {
        let cwd_clone = self.cwd.clone();
        if let Some(Node::Directory { children }) = self.acceder_noeud_mut(&cwd_clone) {
            if let Some(noeud) = children.get(nom) {
                match noeud {
                    Node::Directory { .. } => {
                        if !dossier_attendu {
                            return Err("est_dossier");
                        }
                    }
                    Node::File { .. } => {
                        if dossier_attendu {
                            return Err("est_fichier");
                        }
                    }
                }
                children.remove(nom);
                return Ok(());
            } else {
                return Err("introuvable");
            }
        }
        Err("erreur_fs")
    }

    // --- [FONCTION 1.11 : renommer] ---
    // Description : Renomme un fichier ou un dossier dans le répertoire courant selon le type attendu.
    pub fn renommer(&mut self, ancien: &str, nouveau: &str, dossier_attendu: bool) -> Result<(), &'static str> {
        let cwd_clone = self.cwd.clone();
        if let Some(Node::Directory { children }) = self.acceder_noeud_mut(&cwd_clone) {
            if children.contains_key(nouveau) {
                return Err("existe_deja");
            }

            if let Some(noeud) = children.get(ancien) {
                match noeud {
                    Node::Directory { .. } => {
                        if !dossier_attendu {
                            return Err("est_dossier");
                        }
                    }
                    Node::File { .. } => {
                        if dossier_attendu {
                            return Err("est_fichier");
                        }
                    }
                }
            } else {
                return Err("introuvable");
            }

            if let Some(noeud) = children.remove(ancien) {
                children.insert(String::from(nouveau), noeud);
                return Ok(());
            }
        }
        Err("erreur_fs")
    }
}

// --- [STATIC 1 : SYSTEME_FICHIERS] ---
pub static SYSTEME_FICHIERS: Mutex<FileSystem> = Mutex::new(FileSystem::new());

// --- [FONCTIONS GLOBALES DE COMPATIBILITÉ] ---
pub fn lire(nom: &str) -> Option<Vec<u8>> {
    SYSTEME_FICHIERS.lock().lire(nom)
}

pub fn ecrire(nom: &str, donnees: &[u8]) {
    SYSTEME_FICHIERS.lock().ecrire(nom, donnees.to_vec());
}

pub fn lister() -> Vec<(String, usize)> {
    SYSTEME_FICHIERS.lock().lister_courant()
}

pub fn chemin_actuel() -> String {
    SYSTEME_FICHIERS.lock().obtenir_chemin_actuel()
}

pub fn supprimer(nom: &str, dossier_attendu: bool) -> Result<(), &'static str> {
    SYSTEME_FICHIERS.lock().supprimer(nom, dossier_attendu)
}

pub fn renommer(ancien: &str, nouveau: &str, dossier_attendu: bool) -> Result<(), &'static str> {
    SYSTEME_FICHIERS.lock().renommer(ancien, nouveau, dossier_attendu)
}
