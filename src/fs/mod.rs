// QBX FileSystem Module - Révision 0.6
// Fichier : src/fs/mod.rs
// Description : Façade centrale du VFS avec gestion dynamique du CWD, copie, suppression, renommage et RTC

pub mod node;
pub mod path;
pub mod ops;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use node::{Node, Horodatage};
pub use ops::VfsError;

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

    /// Résout un chemin relatif ou absolu en une liste canonique d'éléments.
    pub fn resoudre_chemin(&self, chemin: &str) -> Vec<String> {
        let elements = path::nettoyer_chemin(chemin);
        let mut resultat = if path::est_absolu(chemin) {
            Vec::new()
        } else {
            self.cwd.clone()
        };

        for elem in elements {
            if elem == ".." {
                resultat.pop();
            } else {
                resultat.push(elem);
            }
        }
        resultat
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
        let cible = self.resoudre_chemin(chemin);

        if let Some(Node::Directory { .. }) = self.acceder_noeud(&cible) {
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

    /// Copie un fichier ou un répertoire avec gestion des options récursives et détection de cycle.
    pub fn copier(&mut self, source_str: &str, cible_str: &str, recursif: bool) -> Result<(), VfsError> {
        let chemin_src = self.resoudre_chemin(source_str);
        if chemin_src.is_empty() {
            return Err(VfsError::SourceIntrouvable);
        }

        // 1. Cloner le nœud source
        let (noeud_a_copier, nom_source) = {
            let noeud = self.acceder_noeud(&chemin_src).ok_or(VfsError::SourceIntrouvable)?;
            if noeud.est_un_dossier() && !recursif {
                return Err(VfsError::EstUnRepertoire);
            }
            let nom = chemin_src.last().unwrap().clone();
            (noeud.clone(), nom)
        };

        // 2. Déterminer le chemin cible absolu
        let mut chemin_dst = self.resoudre_chemin(cible_str);

        // Si la destination existe et est un répertoire : on copie dedans
        if let Some(Node::Directory { .. }) = self.acceder_noeud(&chemin_dst) {
            chemin_dst.push(nom_source);
        }

        // 3. Empêcher la récursion cyclique (ex: copier /a dans /a/b)
        if noeud_a_copier.est_un_dossier() && ops::est_descendant(&chemin_src, &chemin_dst) {
            return Err(VfsError::CycleDetecte);
        }

        // 4. Vérifier que la destination n'existe pas déjà
        if self.acceder_noeud(&chemin_dst).is_some() {
            return Err(VfsError::CibleExisteDeja);
        }

        // 5. Insérer le nœud dans le dossier parent de la cible
        let (dossier_parent_dst, nom_final) = if chemin_dst.len() == 1 {
            (Vec::new(), chemin_dst[0].clone())
        } else if let Some((nom, parent)) = chemin_dst.split_last() {
            (parent.to_vec(), nom.clone())
        } else {
            return Err(VfsError::CheminInvalide);
        };

        if let Some(Node::Directory { children, .. }) = self.acceder_noeud_mut(&dossier_parent_dst) {
            children.insert(nom_final, noeud_a_copier);
            Ok(())
        } else {
            Err(VfsError::DestinationIntrouvable)
        }
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

pub fn copier(source: &str, cible: &str, recursif: bool) -> Result<(), VfsError> {
    SYSTEME_FICHIERS.lock().copier(source, cible, recursif)
}
