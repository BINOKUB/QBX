// QBX EDT Engine - Révision 1.9
// Fichier : src/commandes/edt/moteur.rs
// Description : Moteur avec buffer anonyme, indicateur L/C, flash de sauvegarde et gestion complète

use alloc::vec::Vec;
use spin::Mutex;
use pc_keyboard::{DecodedKey, KeyCode};
use crate::commandes::edt::affichage;
use crate::fs;
use crate::vga_buffer;

pub struct Editeur {
    pub tampon: Vec<u8>,
    pub curseur_pos: usize,
    pub taille_texte: usize,
    pub ligne_debut: usize,
    pub option_l: bool,
    pub nom_fichier: [u8; 64],
    pub taille_nom: usize,
    pub actif: bool,
    pub presse_papier: Vec<u8>,
    pub selection_debut: Option<usize>,
    pub sauvegarde_flash: bool,
}

impl Editeur {
    pub const fn new() -> Self {
        Editeur {
            tampon: Vec::new(),
            curseur_pos: 0,
            taille_texte: 0,
            ligne_debut: 0,
            option_l: false,
            nom_fichier: [0; 64],
            taille_nom: 0,
            actif: false,
            presse_papier: Vec::new(),
            selection_debut: None,
            sauvegarde_flash: false,
        }
    }

    pub fn est_actif(&self) -> bool {
        self.actif
    }

    pub fn lancer(&mut self, option_l: bool, nom_fichier: &str) {
        self.option_l = option_l;
        self.curseur_pos = 0;
        self.ligne_debut = 0;
        self.actif = true;
        self.selection_debut = None;
        self.sauvegarde_flash = false;

        let nom_trim = nom_fichier.trim();
        if nom_trim.is_empty() {
            self.taille_nom = 0;
            self.nom_fichier = [0; 64];
            self.tampon = Vec::new();
            self.taille_texte = 0;
        } else {
            let bytes = nom_trim.as_bytes();
            self.taille_nom = bytes.len().min(64);
            self.nom_fichier = [0; 64];
            self.nom_fichier[..self.taille_nom].copy_from_slice(&bytes[..self.taille_nom]);

            if let Some(contenu) = fs::lire(nom_trim) {
                self.tampon = contenu.to_vec();
                self.taille_texte = self.tampon.len();
                self.curseur_pos = self.taille_texte;
            } else {
                self.tampon = Vec::new();
                self.taille_texte = 0;
            }
        }

        self.ajuster_defilement();
        affichage::dessiner_interface(self);
    }

    pub fn quitter(&mut self) {
        self.actif = false;
        vga_buffer::clear_screen();
        crate::print!("qbx> ");
    }

    pub fn sauvegarder(&mut self) {
        self.enregistrer();
    }

    pub fn enregistrer(&mut self) {
        if self.taille_nom == 0 {
            let defaut = "script.sh";
            let bytes = defaut.as_bytes();
            self.taille_nom = bytes.len().min(64);
            self.nom_fichier[..self.taille_nom].copy_from_slice(&bytes[..self.taille_nom]);
        }

        if self.taille_nom > 0 {
            if let Ok(nom) = core::str::from_utf8(&self.nom_fichier[..self.taille_nom]) {
                fs::ecrire(nom, &self.tampon);
                self.sauvegarde_flash = true;
            }
        }
    }

    pub fn inserer_caractere(&mut self, c: char) {
        if c == '\n' || (c >= ' ' && c <= '~') {
            if self.curseur_pos <= self.tampon.len() {
                self.tampon.insert(self.curseur_pos, c as u8);
                self.curseur_pos += 1;
                self.taille_texte += 1;
            }
        }
    }

    pub fn supprimer_caractere(&mut self) {
        if self.curseur_pos > 0 && !self.tampon.is_empty() {
            self.curseur_pos -= 1;
            self.tampon.remove(self.curseur_pos);
            self.taille_texte -= 1;
        }
    }

    pub fn deplacer_curseur(&mut self, dx: isize, dy: isize) {
        if dx == -1 && self.curseur_pos > 0 {
            self.curseur_pos -= 1;
        } else if dx == 1 && self.curseur_pos < self.taille_texte {
            self.curseur_pos += 1;
        }

        if dy != 0 {
            let mut x_actuel = 0;
            let mut debut_ligne = 0;
            
            for i in (0..self.curseur_pos).rev() {
                if self.tampon[i] == b'\n' {
                    debut_ligne = i + 1;
                    x_actuel = self.curseur_pos - debut_ligne;
                    break;
                }
                if i == 0 { x_actuel = self.curseur_pos; }
            }

            if dy == -1 {
                if debut_ligne > 0 {
                    let fin_ligne_prec = debut_ligne - 1;
                    let mut debut_ligne_prec = 0;
                    for i in (0..fin_ligne_prec).rev() {
                        if self.tampon[i] == b'\n' {
                            debut_ligne_prec = i + 1;
                            break;
                        }
                    }
                    let taille_ligne_prec = fin_ligne_prec - debut_ligne_prec;
                    let cible_x = x_actuel.min(taille_ligne_prec);
                    self.curseur_pos = debut_ligne_prec + cible_x;
                }
            } else if dy == 1 {
                let mut fin_ligne = self.taille_texte;
                for i in self.curseur_pos..self.taille_texte {
                    if self.tampon[i] == b'\n' {
                        fin_ligne = i;
                        break;
                    }
                }
                
                if fin_ligne < self.taille_texte {
                    let debut_ligne_suiv = fin_ligne + 1;
                    let mut fin_ligne_suiv = self.taille_texte;
                    for i in debut_ligne_suiv..self.taille_texte {
                        if self.tampon[i] == b'\n' {
                            fin_ligne_suiv = i;
                            break;
                        }
                    }
                    let taille_ligne_suiv = fin_ligne_suiv - debut_ligne_suiv;
                    let cible_x = x_actuel.min(taille_ligne_suiv);
                    self.curseur_pos = debut_ligne_suiv + cible_x;
                }
            }
        }
        self.ajuster_defilement();
        affichage::dessiner_interface(self);
    }

    fn obtenir_limites_ligne(&self) -> (usize, usize) {
        let mut debut = 0;
        for i in (0..self.curseur_pos).rev() {
            if self.tampon[i] == b'\n' {
                debut = i + 1;
                break;
            }
        }
        let mut fin = self.taille_texte;
        for i in self.curseur_pos..self.taille_texte {
            if self.tampon[i] == b'\n' {
                fin = i;
                break;
            }
        }
        (debut, fin)
    }

    pub fn basculer_selection(&mut self) {
        if self.selection_debut.is_some() {
            self.selection_debut = None;
        } else {
            self.selection_debut = Some(self.curseur_pos);
        }
        affichage::dessiner_interface(self);
    }

    pub fn copier_selection_ou_ligne(&mut self) {
        self.presse_papier.clear();
        
        if let Some(debut) = self.selection_debut {
            let min = debut.min(self.curseur_pos);
            let max = debut.max(self.curseur_pos);
            self.presse_papier.extend_from_slice(&self.tampon[min..max]);
            self.selection_debut = None;
        } else {
            let (debut, fin) = self.obtenir_limites_ligne();
            let fin_copie = if fin < self.taille_texte { fin + 1 } else { fin };
            self.presse_papier.extend_from_slice(&self.tampon[debut..fin_copie]);
        }
    }

    pub fn couper_selection_ou_ligne(&mut self) {
        if let Some(debut) = self.selection_debut {
            let min = debut.min(self.curseur_pos);
            let max = debut.max(self.curseur_pos);
            
            self.presse_papier.clear();
            self.presse_papier.extend_from_slice(&self.tampon[min..max]);
            
            self.tampon.drain(min..max);
            self.taille_texte = self.tampon.len();
            self.curseur_pos = min;
            self.selection_debut = None;
            self.ajuster_defilement();
            affichage::dessiner_interface(self);
        } else {
            self.copier_selection_ou_ligne();
            let (debut, fin) = self.obtenir_limites_ligne();
            let fin_coupe = if fin < self.taille_texte { fin + 1 } else { fin };
            
            self.tampon.drain(debut..fin_coupe);
            self.taille_texte = self.tampon.len();
            self.curseur_pos = debut.min(self.taille_texte);
            self.ajuster_defilement();
            affichage::dessiner_interface(self);
        }
    }

    pub fn coller(&mut self) {
        if self.presse_papier.is_empty() { return; }
        
        let mut i = self.curseur_pos;
        for &octet in &self.presse_papier {
            self.tampon.insert(i, octet);
            i += 1;
        }
        self.taille_texte = self.tampon.len();
        self.curseur_pos = i;
        self.ajuster_defilement();
        affichage::dessiner_interface(self);
    }

    pub fn supprimer_caractere_droit(&mut self) {
        if self.curseur_pos < self.taille_texte {
            self.tampon.remove(self.curseur_pos);
            self.taille_texte -= 1;
            self.ajuster_defilement();
            affichage::dessiner_interface(self);
        }
    }

    pub fn ajuster_defilement(&mut self) {
        let mut ligne_curseur = 0;
        let limit = self.curseur_pos.min(self.tampon.len());
        for i in 0..limit {
            if self.tampon[i] == b'\n' {
                ligne_curseur += 1;
            }
        }

        let hauteur_max = affichage::HAUTEUR_EDIT;
        if ligne_curseur < self.ligne_debut {
            self.ligne_debut = ligne_curseur;
        } else if ligne_curseur >= self.ligne_debut + hauteur_max {
            self.ligne_debut = ligne_curseur - hauteur_max + 1;
        }
    }

    pub fn traiter_touche(&mut self, key: DecodedKey) -> bool {
        // Efface le flash de sauvegarde dès qu'une action/frappe survient
        self.sauvegarde_flash = false;

        match key {
            DecodedKey::RawKey(KeyCode::Escape) | DecodedKey::Unicode('\x1b') => {
                self.quitter();
                return true;
            }
            DecodedKey::RawKey(KeyCode::F2) => {
                self.enregistrer();
                affichage::dessiner_interface(self);
                return false;
            }
            DecodedKey::RawKey(KeyCode::F3) => {
                self.copier_selection_ou_ligne(); 
            }
            DecodedKey::RawKey(KeyCode::F4) => {
                self.couper_selection_ou_ligne();
            }
            DecodedKey::RawKey(KeyCode::F5) => {
                self.coller();
            }
            DecodedKey::RawKey(KeyCode::F6) => {
                self.basculer_selection();
            }
            DecodedKey::RawKey(KeyCode::Delete) | DecodedKey::Unicode('\x7f') => {
                self.supprimer_caractere_droit();
            }
            DecodedKey::RawKey(KeyCode::ArrowUp) => self.deplacer_curseur(0, -1),
            DecodedKey::RawKey(KeyCode::ArrowDown) => self.deplacer_curseur(0, 1),
            DecodedKey::RawKey(KeyCode::ArrowLeft) => self.deplacer_curseur(-1, 0),
            DecodedKey::RawKey(KeyCode::ArrowRight) => self.deplacer_curseur(1, 0),
            DecodedKey::Unicode('\x08') => {
                self.supprimer_caractere();
                self.ajuster_defilement();
                affichage::dessiner_interface(self);
            }
            DecodedKey::Unicode(c) => {
                self.inserer_caractere(c);
                self.ajuster_defilement();
                affichage::dessiner_interface(self);
            }
            _ => {}
        }
        self.ajuster_defilement();
        false
    }
}

pub static EDITEUR: Mutex<Editeur> = Mutex::new(Editeur::new());
