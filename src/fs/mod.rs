// QBX FileSystem Module - Révision 0.7
// Fichier : src/fs/mod.rs
// Description : Façade centrale du VFS persistée sur la partition /var (offset 65+)

pub mod node;
pub mod path;
pub mod ops;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use node::{Node, Horodatage};
pub use ops::VfsError;
use crate::storage::partitions;

const LBA_FS_SUPERBLOC: u64 = 65;
const LBA_FS_DATA_DEBUT: u64 = 66;
const FS_MAGIC: &[u8; 12] = b"QBX_FS_V1\0\0\0";

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
            self.synchroniser_sur_disque();
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
                self.synchroniser_sur_disque();
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
                        if !dossier_attendu { return Err("est_dossier"); }
                    }
                    Node::File { .. } => {
                        if dossier_attendu { return Err("est_fichier"); }
                    }
                }
                children.remove(nom);
                self.synchroniser_sur_disque();
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
                        if !dossier_attendu { return Err("est_dossier"); }
                    }
                    Node::File { .. } => {
                        if dossier_attendu { return Err("est_fichier"); }
                    }
                }
            } else {
                return Err("introuvable");
            }

            if let Some(noeud) = children.remove(ancien) {
                children.insert(String::from(nouveau), noeud);
                self.synchroniser_sur_disque();
                return Ok(());
            }
        }
        Err("erreur_fs")
    }

    pub fn copier(&mut self, source_str: &str, cible_str: &str, recursif: bool) -> Result<(), VfsError> {
        let chemin_src = self.resoudre_chemin(source_str);
        if chemin_src.is_empty() {
            return Err(VfsError::SourceIntrouvable);
        }

        let (noeud_a_copier, nom_source) = {
            let noeud = self.acceder_noeud(&chemin_src).ok_or(VfsError::SourceIntrouvable)?;
            if noeud.est_un_dossier() && !recursif {
                return Err(VfsError::EstUnRepertoire);
            }
            let nom = chemin_src.last().unwrap().clone();
            (noeud.clone(), nom)
        };

        let mut chemin_dst = self.resoudre_chemin(cible_str);
        if let Some(Node::Directory { .. }) = self.acceder_noeud(&chemin_dst) {
            chemin_dst.push(nom_source);
        }

        if noeud_a_copier.est_un_dossier() && ops::est_descendant(&chemin_src, &chemin_dst) {
            return Err(VfsError::CycleDetecte);
        }

        if self.acceder_noeud(&chemin_dst).is_some() {
            return Err(VfsError::CibleExisteDeja);
        }

        let (dossier_parent_dst, nom_final) = if chemin_dst.len() == 1 {
            (Vec::new(), chemin_dst[0].clone())
        } else if let Some((nom, parent)) = chemin_dst.split_last() {
            (parent.to_vec(), nom.clone())
        } else {
            return Err(VfsError::CheminInvalide);
        };

        if let Some(Node::Directory { children, .. }) = self.acceder_noeud_mut(&dossier_parent_dst) {
            children.insert(nom_final, noeud_a_copier);
            self.synchroniser_sur_disque();
            Ok(())
        } else {
            Err(VfsError::DestinationIntrouvable)
        }
    }

    pub fn deplacer(&mut self, source_str: &str, cible_str: &str) -> Result<(), VfsError> {
        let chemin_src = self.resoudre_chemin(source_str);
        if chemin_src.is_empty() {
            return Err(VfsError::SourceIntrouvable);
        }

        let est_dossier = {
            let noeud = self.acceder_noeud(&chemin_src).ok_or(VfsError::SourceIntrouvable)?;
            noeud.est_un_dossier()
        };

        let nom_source = chemin_src.last().unwrap().clone();
        let mut chemin_dst = self.resoudre_chemin(cible_str);

        if let Some(Node::Directory { .. }) = self.acceder_noeud(&chemin_dst) {
            chemin_dst.push(nom_source.clone());
        }

        if est_dossier && ops::est_descendant(&chemin_src, &chemin_dst) {
            return Err(VfsError::CycleDetecte);
        }

        if self.acceder_noeud(&chemin_dst).is_some() {
            return Err(VfsError::CibleExisteDeja);
        }

        let (dossier_parent_dst, nom_final) = if chemin_dst.len() == 1 {
            (Vec::new(), chemin_dst[0].clone())
        } else if let Some((nom, parent)) = chemin_dst.split_last() {
            (parent.to_vec(), nom.clone())
        } else {
            return Err(VfsError::CheminInvalide);
        };

        if self.acceder_noeud(&dossier_parent_dst).is_none() {
            return Err(VfsError::DestinationIntrouvable);
        }

        let (dossier_parent_src, nom_src_element) = if chemin_src.len() == 1 {
            (Vec::new(), chemin_src[0].clone())
        } else if let Some((nom, parent)) = chemin_src.split_last() {
            (parent.to_vec(), nom.clone())
        } else {
            return Err(VfsError::CheminInvalide);
        };

        let noeud_extrait = if let Some(Node::Directory { children, .. }) = self.acceder_noeud_mut(&dossier_parent_src) {
            children.remove(&nom_src_element).ok_or(VfsError::SourceIntrouvable)?
        } else {
            return Err(VfsError::SourceIntrouvable);
        };

        if let Some(Node::Directory { children, .. }) = self.acceder_noeud_mut(&dossier_parent_dst) {
            children.insert(nom_final, noeud_extrait);
            self.synchroniser_sur_disque();
            Ok(())
        } else {
            Err(VfsError::DestinationIntrouvable)
        }
    }

    // --- Persistance sur /var ---

    fn collecter_entrees(&self, noeud: &Node, prefixe: &str, sortie: &mut Vec<(String, bool, Vec<u8>, Horodatage)>) {
        if let Node::Directory { children, .. } = noeud {
            for (nom, enfant) in children {
                let mut chemin = String::from(prefixe);
                if !chemin.is_empty() {
                    chemin.push('/');
                }
                chemin.push_str(nom);

                let h = enfant.horodatage();
                match enfant {
                    Node::Directory { .. } => {
                        sortie.push((chemin.clone(), true, Vec::new(), h));
                        self.collecter_entrees(enfant, &chemin, sortie);
                    }
                    Node::File { data, .. } => {
                        sortie.push((chemin, false, data.clone(), h));
                    }
                }
            }
        }
    }

    fn inserer_noeud_absolu(&mut self, chemin: &str, est_dossier: bool, data: Vec<u8>, h: Horodatage) {
        let parties: Vec<&str> = chemin.trim_matches('/').split('/').filter(|p| !p.is_empty()).collect();
        if parties.is_empty() {
            return;
        }

        let mut courant = &mut self.root;
        let nb_parties = parties.len();

        for (idx, &partie) in parties.iter().enumerate() {
            let est_dernier = idx == nb_parties - 1;
            match courant {
                Node::Directory { children, .. } => {
                    if est_dernier {
                        if est_dossier {
                            if !children.contains_key(partie) {
                                children.insert(String::from(partie), Node::nouveau_dossier(h));
                            }
                        } else {
                            children.insert(String::from(partie), Node::nouveau_fichier(data, h));
                        }
                        return;
                    } else {
                        if !children.contains_key(partie) {
                            children.insert(String::from(partie), Node::nouveau_dossier(h));
                        }
                        courant = children.get_mut(partie).unwrap();
                    }
                }
                _ => return,
            }
        }
    }

    pub fn synchroniser_sur_disque(&self) {
        let mut entrees = Vec::new();
        self.collecter_entrees(&self.root, "", &mut entrees);

        let mut flux = Vec::new();
        for (chemin, est_dossier, data, horo) in &entrees {
            flux.push(if *est_dossier { 1u8 } else { 0u8 });
            flux.push(horo.heure);
            flux.push(horo.minute);
            flux.push(horo.seconde);
            let chemin_bytes = chemin.as_bytes();
            let p_len = (chemin_bytes.len() as u16).to_le_bytes();
            flux.extend_from_slice(&p_len);
            flux.extend_from_slice(chemin_bytes);

            if !est_dossier {
                let d_len = (data.len() as u32).to_le_bytes();
                flux.extend_from_slice(&d_len);
                flux.extend_from_slice(data);
            }
        }

        let taille_flux = flux.len() as u32;
        let nb_entrees = entrees.len() as u32;
        let nb_secteurs = if taille_flux == 0 {
            0u32
        } else {
            ((taille_flux + 511) / 512) as u32
        };

        if nb_secteurs > 0 {
            for (i, chunk) in flux.chunks(512).enumerate() {
                let mut secteur = [0u8; 512];
                secteur[..chunk.len()].copy_from_slice(chunk);
                let _ = partitions::ecrire_secteur_var(LBA_FS_DATA_DEBUT + (i as u64), &secteur);
            }
        }

        let mut sb = [0u8; 512];
        sb[0..12].copy_from_slice(FS_MAGIC);
        sb[12..16].copy_from_slice(&taille_flux.to_le_bytes());
        sb[16..20].copy_from_slice(&nb_entrees.to_le_bytes());
        sb[20..24].copy_from_slice(&nb_secteurs.to_le_bytes());
        let _ = partitions::ecrire_secteur_var(LBA_FS_SUPERBLOC, &sb);
    }

    pub fn charger_depuis_disque(&mut self) {
        let mut sb = [0u8; 512];
        if partitions::lire_secteur_var(LBA_FS_SUPERBLOC, &mut sb).is_err() {
            return;
        }

        if &sb[0..12] != FS_MAGIC {
            return;
        }

        let taille_flux = u32::from_le_bytes([sb[12], sb[13], sb[14], sb[15]]) as usize;
        let nb_entrees = u32::from_le_bytes([sb[16], sb[17], sb[18], sb[19]]) as usize;
        let nb_secteurs = u32::from_le_bytes([sb[20], sb[21], sb[22], sb[23]]) as usize;

        if nb_entrees == 0 || taille_flux == 0 {
            return;
        }

        if taille_flux > 16 * 1024 * 1024 || nb_secteurs != (taille_flux + 511) / 512 {
            return;
        }

        let mut flux = Vec::with_capacity(nb_secteurs * 512);
        for i in 0..nb_secteurs {
            let mut secteur = [0u8; 512];
            if partitions::lire_secteur_var(LBA_FS_DATA_DEBUT + (i as u64), &mut secteur).is_err() {
                return;
            }
            flux.extend_from_slice(&secteur);
        }
        flux.truncate(taille_flux);

        let mut offset = 0;
        let mut restaures = 0;

        while offset + 6 <= flux.len() {
            let est_dossier = flux[offset] == 1;
            let h = flux[offset + 1];
            let m = flux[offset + 2];
            let s = flux[offset + 3];
            let path_len = u16::from_le_bytes([flux[offset + 4], flux[offset + 5]]) as usize;
            offset += 6;

            if offset + path_len > flux.len() {
                break;
            }

            let chemin = match core::str::from_utf8(&flux[offset..offset + path_len]) {
                Ok(s_ref) => s_ref,
                Err(_) => break,
            };
            offset += path_len;

            let horo = Horodatage::new(h, m, s);

            if est_dossier {
                self.inserer_noeud_absolu(chemin, true, Vec::new(), horo);
                restaures += 1;
            } else {
                if offset + 4 > flux.len() {
                    break;
                }
                let data_len = u32::from_le_bytes([
                    flux[offset], flux[offset + 1], flux[offset + 2], flux[offset + 3],
                ]) as usize;
                offset += 4;

                if offset + data_len > flux.len() {
                    break;
                }
                let data = flux[offset..offset + data_len].to_vec();
                offset += data_len;

                self.inserer_noeud_absolu(chemin, false, data, horo);
                restaures += 1;
            }
        }

        if restaures > 0 {
            crate::println!("[VAR] VFS /var : {} élément(s) restauré(s) depuis le stockage.", restaures);
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

pub fn deplacer(source: &str, cible: &str) -> Result<(), VfsError> {
    SYSTEME_FICHIERS.lock().deplacer(source, cible)
}

pub fn charger_depuis_disque() {
    SYSTEME_FICHIERS.lock().charger_depuis_disque();
}

pub fn synchroniser() {
    SYSTEME_FICHIERS.lock().synchroniser_sur_disque();
}
