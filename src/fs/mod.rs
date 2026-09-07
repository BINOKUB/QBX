// QBX FileSystem Module - Révision 0.5
// Fichier : src/fs/mod.rs
// Description : Façade centrale du VFS avec gestion dynamique du CWD, suppression, renommage et horodatage RTC

pub mod node;
pub mod path;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use node::{Node, Horodatage};

pub struct FileSystem {
    root: Node,
    cwd: Vec<String>,
}

impl FileSystem {
    pub const fn new() -> Self {
        FileSystem {
            root: Node::Directory {
                children: BTreeMap::new(),
                horodatage: Horodatage { heure: 0, minute: 0, seconde: 0 },
            },
            cwd: Vec::new(),
        }
    }

    fn date_actuelle() -> Horodatage {
        let (h, m, s) = crate::commandes::tmps::obtenir_heure_actuelle();
        Horodatage::new(h, m, s)
    }

    pub fn ecrire(&mut self, nom: &str, donnees: Vec<u8>) -> bool {
        let cwd_clone = self.cwd.clone();
        let h = Self::date_actuelle();
        if let Some(Node::Directory { children, .. }) = self.acceder_noeud_mut(&cwd_clone) {
            children.insert(String::from(nom), Node::nouveau_fichier(donnees, h));
            return true;
        }
        false
    }

    pub fn lire(&self, nom: &str) -> Option<Vec<u8>> {
        if let Some(Node::Directory { children, .. }) = self.acceder_noeud(&self.cwd) {
            if let Some(Node::File { data, .. }) = children.get(nom) {
                return Some(data.clone());
            }
        }
        None
    }

    // Retourne : (nom, taille_octets, est_dossier, horodatage)
    pub fn lister_courant(&self) -> Vec<(String, usize, bool, Horodatage)> {
        let mut resultats = Vec::new();
        if let Some(Node::Directory { children, .. }) = self.acceder_noeud(&self.cwd) {
            for (nom, node) in children {
                let (taille, est_dossier) = match node {
                    Node::File { data, .. } => (data.len(), false),
                    Node::Directory { children, .. } => (children.len(), true),
                };
                resultats.push((nom.clone(), taille, est_dossier, node.horodatage()));
            }
        }
        resultats
    }

    pub fn creer_repertoire(&mut self, nom: &str) -> bool {
        if nom.is_empty() {
            return false;
        }
        let cwd_clone = self.cwd.clone();
        let h = Self::date_actuelle();
        if let Some(Node::Directory { children, .. }) = self.acceder_noeud_mut(&cwd_clone) {
            if !children.contains_key(nom) {
                children.insert(String::from(nom), Node::nouveau_dossier(h));
                return true;
            }
        }
        false
    }

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

    fn acceder_noeud(&self, chemin: &[String]) -> Option<&Node> {
        let mut courant = &self.root;
        for dossier in chemin {
            if let Node::Directory { children, .. } = courant {
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

    fn acceder_noeud_mut(&mut self, chemin: &[String]) -> Option<&mut Node> {
        let mut courant = &mut self.root;
        for dossier in chemin {
            if let Node::Directory { children, .. } = courant {
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

    pub fn supprimer(&mut self, nom: &str, dossier_attendu: bool) -> Result<(), &'static str> {
        let cwd_clone = self.cwd.clone();
        if let Some(Node::Directory { children, .. }) = self.acceder_noeud_mut(&cwd_clone) {
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

    pub fn renommer(&mut self, ancien: &str, nouveau: &str, dossier_attendu: bool) -> Result<(), &'static str> {
        let cwd_clone = self.cwd.clone();
        if let Some(Node::Directory { children, .. }) = self.acceder_noeud_mut(&cwd_clone) {
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

pub static SYSTEME_FICHIERS: Mutex<FileSystem> = Mutex::new(FileSystem::new());

pub fn lire(nom: &str) -> Option<Vec<u8>> {
    SYSTEME_FICHIERS.lock().lire(nom)
}

pub fn ecrire(nom: &str, donnees: &[u8]) {
    SYSTEME_FICHIERS.lock().ecrire(nom, donnees.to_vec());
}

pub fn lister() -> Vec<(String, usize, bool, Horodatage)> {
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
