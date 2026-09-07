// QBX EDT Engine - Révision 1.4
// Fichier : src/commandes/edt/moteur.rs
// Description : Moteur avec navigation 2D absolue et presse-papier (Copier/Couper/Coller via F3/F4/F5)

use alloc::vec::Vec;
use spin::Mutex;
use pc_keyboard::{DecodedKey, KeyCode};
use crate::commandes::edt::affichage;
use crate::fs;
use crate::vga_buffer;

// --- [STRUCTURE 1 : Editeur] ---
// Description : Gère l'état complet de l'éditeur de texte et du presse-papier.
pub struct Editeur {
    pub tampon: Vec<u8>,
    pub curseur_pos: usize,
    pub taille_texte: usize,
    pub ligne_debut: usize,
    pub option_l: bool,
    pub nom_fichier: [u8; 32],
    pub taille_nom: usize,
    pub actif: bool,
    pub presse_papier: Vec<u8>,
}

impl Editeur {
    // --- [FONCTION 1 : new] ---
    // Description : Instancie un nouvel objet Editeur réinitialisé avec presse-papier.
    pub const fn new() -> Self {
        Editeur {
            tampon: Vec::new(),
            curseur_pos: 0,
            taille_texte: 0,
            ligne_debut: 0,
            option_l: false,
            nom_fichier: [0; 32],
            taille_nom: 0,
            actif: false,
            presse_papier: Vec::new(),
        }
    }

    // --- [FONCTION 2 : est_actif] ---
    // Description : Renvoie un booléen indiquant si l'éditeur est en cours d'utilisation.
    pub fn est_actif(&self) -> bool {
        self.actif
    }

    // --- [FONCTION 3 : lancer] ---
    // Description : Initialise l'éditeur et charge le fichier.
    pub fn lancer(&mut self, option_l: bool, nom_fichier: &str) {
        self.option_l = option_l;
        self.curseur_pos = 0;
        self.ligne_debut = 0;
        self.actif = true;

        let nom_final = if nom_fichier.trim().is_empty() { "sans_titre.txt" } else { nom_fichier };
        let bytes = nom_final.as_bytes();
        self.taille_nom = bytes.len().min(32);
        self.nom_fichier[..self.taille_nom].copy_from_slice(&bytes[..self.taille_nom]);

        if let Some(contenu) = fs::lire(nom_final) {
            self.tampon = contenu.to_vec();
            self.taille_texte = self.tampon.len();
            self.curseur_pos = self.taille_texte;
        } else {
            self.tampon = Vec::new();
            self.taille_texte = 0;
        }

        affichage::dessiner_interface(self);
    }

    // --- [FONCTION 4 : quitter] ---
    // Description : Désactive l'éditeur, efface proprement l'écran VGA et restitue le prompt.
    pub fn quitter(&mut self) {
        self.actif = false;
        vga_buffer::clear_screen();
        crate::print!("qbx> ");
    }

    // --- [FONCTION 5 : sauvegarder] ---
    // Description : Alias pour enregistrer le fichier.
    pub fn sauvegarder(&self) {
        self.enregistrer();
    }

    // --- [FONCTION 6 : enregistrer] ---
    // Description : Écrit le contenu du tampon dynamique dans le système de fichiers (VFS).
    pub fn enregistrer(&self) {
        if self.taille_nom > 0 {
            if let Ok(nom) = core::str::from_utf8(&self.nom_fichier[..self.taille_nom]) {
                fs::ecrire(nom, &self.tampon);
            }
        }
    }

    // --- [FONCTION 7 : inserer_caractere] ---
    // Description : Insère un caractère ou un saut de ligne dans le tampon.
    pub fn inserer_caractere(&mut self, c: char) {
        if c == '\n' || (c >= ' ' && c <= '~') {
            if self.curseur_pos <= self.tampon.len() {
                self.tampon.insert(self.curseur_pos, c as u8);
                self.curseur_pos += 1;
                self.taille_texte += 1;
            }
        }
    }

    // --- [FONCTION 8 : supprimer_caractere] ---
    // Description : Supprime le caractère situé juste avant le curseur.
    pub fn supprimer_caractere(&mut self) {
        if self.curseur_pos > 0 && !self.tampon.is_empty() {
            self.curseur_pos -= 1;
            self.tampon.remove(self.curseur_pos);
            self.taille_texte -= 1;
        }
    }

    // --- [FONCTION 9 : deplacer_curseur] ---
    // Description : Convertit un mouvement 2D en index absolu dans le vecteur texte.
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
        affichage::dessiner_interface(self);
    }

    // --- [FONCTION 10 : obtenir_limites_ligne] ---
    // Description : Trouve l'index de début et de fin de la ligne où se trouve le curseur.
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

    // --- [FONCTION 11 : copier_ligne] ---
    // Description : Touche F3 - Copie la ligne courante dans le presse-papier.
    pub fn copier_ligne(&mut self) {
        let (debut, fin) = self.obtenir_limites_ligne();
        self.presse_papier.clear();
        let fin_copie = if fin < self.taille_texte { fin + 1 } else { fin }; // Inclus le \n
        self.presse_papier.extend_from_slice(&self.tampon[debut..fin_copie]);
    }

    // --- [FONCTION 12 : couper_ligne] ---
    // Description : Touche F4 - Copie puis supprime la ligne courante.
    pub fn couper_ligne(&mut self) {
        self.copier_ligne();
        let (debut, fin) = self.obtenir_limites_ligne();
        let fin_coupe = if fin < self.taille_texte { fin + 1 } else { fin };
        
        self.tampon.drain(debut..fin_coupe);
        self.taille_texte = self.tampon.len();
        self.curseur_pos = debut.min(self.taille_texte);
        affichage::dessiner_interface(self);
    }

    // --- [FONCTION 13 : coller] ---
    // Description : Touche F5 - Insère le contenu du presse-papier à la position du curseur.
    pub fn coller(&mut self) {
        if self.presse_papier.is_empty() { return; }
        
        let mut i = self.curseur_pos;
        for &octet in &self.presse_papier {
            self.tampon.insert(i, octet);
            i += 1;
        }
        self.taille_texte = self.tampon.len();
        self.curseur_pos = i;
        affichage::dessiner_interface(self);
    }

    // --- [FONCTION 14 : supprimer_caractere_droit] ---
    // Description : Touche DEL (Suppr) - Supprime le caractère situé exactement sous le curseur.
    pub fn supprimer_caractere_droit(&mut self) {
        if self.curseur_pos < self.taille_texte {
            self.tampon.remove(self.curseur_pos);
            self.taille_texte -= 1;
            affichage::dessiner_interface(self);
        }
    }

   // --- [FONCTION 15 : traiter_touche] ---
    // Description : Analyse les touches reçues, intégrant un filet de sécurité (catch-all) pour la touche DEL.
    pub fn traiter_touche(&mut self, key: DecodedKey) -> bool {
        match key {
            DecodedKey::RawKey(KeyCode::Escape) | DecodedKey::Unicode('\x1b') => {
                self.quitter();
                return true;
            }
            DecodedKey::RawKey(KeyCode::F2) => {
                self.enregistrer();
                affichage::dessiner_interface(self);
            }
            DecodedKey::RawKey(KeyCode::F3) => {
                self.copier_ligne(); 
            }
            DecodedKey::RawKey(KeyCode::F4) => {
                self.couper_ligne();
            }
            DecodedKey::RawKey(KeyCode::F5) => {
                self.coller();
            }
            // --- CORRECTION : Capture du Delete standard et de l'ASCII 127 (Émulateur) ---
            DecodedKey::RawKey(KeyCode::Delete) | 
            DecodedKey::Unicode('\x7f') => {
                self.supprimer_caractere_droit();
            }
            DecodedKey::RawKey(KeyCode::ArrowUp) => self.deplacer_curseur(0, -1),
            DecodedKey::RawKey(KeyCode::ArrowDown) => self.deplacer_curseur(0, 1),
            DecodedKey::RawKey(KeyCode::ArrowLeft) => self.deplacer_curseur(-1, 0),
            DecodedKey::RawKey(KeyCode::ArrowRight) => self.deplacer_curseur(1, 0),
            DecodedKey::Unicode('\x08') => {
                self.supprimer_caractere();
                affichage::dessiner_interface(self);
            }
            DecodedKey::Unicode(c) => {
                self.inserer_caractere(c);
                affichage::dessiner_interface(self);
            }
            _ => {}
        }
        false
    }
}

// --- [STATIC 1 : EDITEUR] ---
pub static EDITEUR: Mutex<Editeur> = Mutex::new(Editeur::new());
