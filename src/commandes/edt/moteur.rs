// QBX EDT Engine - Révision 1.3
// Fichier : src/commandes/edt/moteur.rs
// Description : Moteur réorganisé, compatible VGA direct et file asynchrone (Corrections F2 et ESC)

use alloc::vec::Vec;
use spin::Mutex;
use pc_keyboard::{DecodedKey, KeyCode};
use crate::commandes::edt::affichage;
use crate::fs;
use crate::vga_buffer;

// --- [STRUCTURE 1 : Editeur] ---
// Description : Gère l'état complet de l'éditeur de texte.
pub struct Editeur {
    pub tampon: Vec<u8>,
    pub curseur_pos: usize,
    pub taille_texte: usize,
    pub ligne_debut: usize,
    pub option_l: bool,
    pub nom_fichier: [u8; 32],
    pub taille_nom: usize,
    pub actif: bool,
}

impl Editeur {
    // --- [FONCTION 1 : new] ---
    // Description : Instancie un nouvel objet Editeur réinitialisé.
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
        }
    }

    // --- [FONCTION 2 : est_actif] ---
    // Description : Renvoie un booléen indiquant si l'éditeur est en cours d'utilisation.
    pub fn est_actif(&self) -> bool {
        self.actif
    }

    // --- [FONCTION 3 : lancer] ---
    // Description : Initialise l'éditeur, force un nom par défaut si vide (corrige le bug F2), et charge le contenu.
    pub fn lancer(&mut self, option_l: bool, nom_fichier: &str) {
        self.option_l = option_l;
        self.curseur_pos = 0;
        self.ligne_debut = 0;
        self.actif = true;

        // Correction : Si aucun nom n'est fourni, on impose "sans_titre.txt"
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

        // On dessine l'interface et on retourne immédiatement.
        affichage::dessiner_interface(self);
    }

    // --- [FONCTION 4 : quitter] ---
    // Description : Désactive l'éditeur, efface proprement l'écran VGA et restitue le prompt.
    pub fn quitter(&mut self) {
        self.actif = false;
        // Correction : on utilise la fonction du module VGA pour synchroniser le curseur matériel
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
    // Description : Insère un caractère ou un saut de ligne dans le tampon à la position du curseur.
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
    // Description : Supprime le caractère situé juste avant le curseur (Backspace).
    pub fn supprimer_caractere(&mut self) {
        if self.curseur_pos > 0 && !self.tampon.is_empty() {
            self.curseur_pos -= 1;
            self.tampon.remove(self.curseur_pos);
            self.taille_texte -= 1;
        }
    }

    // --- [FONCTION 9 : deplacer_curseur] ---
    // Description : Ajuste la position du curseur en fonction des flèches directionnelles et met à jour l'écran.
    pub fn deplacer_curseur(&mut self, dx: isize, dy: isize) {
        if dy < 0 && self.ligne_debut > 0 {
            self.ligne_debut -= 1;
        } else if dy > 0 {
            self.ligne_debut += 1;
        }

        if dx < 0 && self.curseur_pos > 0 {
            self.curseur_pos -= 1;
        } else if dx > 0 && self.curseur_pos < self.taille_texte {
            self.curseur_pos += 1;
        }

        affichage::dessiner_interface(self);
    }

    // --- [FONCTION 10 : traiter_touche] ---
    // Description : Analyse les touches reçues (incluant l'échappement ASCII) et déclenche l'action associée.
    pub fn traiter_touche(&mut self, key: DecodedKey) -> bool {
        match key {
            // Correction : Capture du code brut ESC et du caractère ASCII 27 (Échap)
            DecodedKey::RawKey(KeyCode::Escape) | DecodedKey::Unicode('\x1b') => {
                self.quitter();
                return true;
            }
            DecodedKey::RawKey(KeyCode::F2) => {
                self.enregistrer();
                affichage::dessiner_interface(self);
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
// Description : Instance globale de l'éditeur protégée par Mutex.
pub static EDITEUR: Mutex<Editeur> = Mutex::new(Editeur::new());
