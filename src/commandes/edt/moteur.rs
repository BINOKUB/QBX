// QBX EDT Moteur - Révision 0.7
// Fichier : src/commandes/edt/moteur.rs
// Description : Moteur de données avec support du défilement vertical (scrolling 100 lignes)

use super::affichage::{self, LARGEUR, HAUTEUR_EDIT};

const HAUTEUR_MAX: usize = 100;
const TAMPON_TAILLE: usize = LARGEUR * HAUTEUR_MAX;

// --- [STRUCTURE 1 : Editeur] ---
// Description : État interne de l'éditeur avec gestion du décalage de vue (scrolling).
pub struct Editeur {
    tampon: [u8; TAMPON_TAILLE],
    curseur_x: usize,
    curseur_y: usize,
    decalage_y: usize,
    actif: bool,
    afficher_lignes: bool,
    nom_fichier: [u8; 32],
    nom_len: usize,
}

impl Editeur {
    // --- [FONCTION 1.1 : new] ---
    pub const fn new() -> Self {
        Editeur {
            tampon: [0; TAMPON_TAILLE],
            curseur_x: 0,
            curseur_y: 0,
            decalage_y: 0,
            actif: false,
            afficher_lignes: false,
            nom_fichier: [0; 32],
            nom_len: 0,
        }
    }

    // --- [FONCTION 1.2 : lancer] ---
    pub fn lancer(&mut self, option_lignes: bool, nom: &str) {
        self.actif = true;
        self.tampon = [0; TAMPON_TAILLE];
        self.curseur_x = 0;
        self.curseur_y = 0;
        self.decalage_y = 0;
        self.afficher_lignes = option_lignes;

        self.nom_len = 0;
        for b in nom.bytes() {
            if self.nom_len < 32 {
                self.nom_fichier[self.nom_len] = b;
                self.nom_len += 1;
            }
        }
        self.rafraichir_ecran();
    }

    // --- [FONCTION 1.3 : inserer_caractere] ---
    pub fn inserer_caractere(&mut self, c: char) {
        if !self.actif { return; }

        match c {
            '\x1b' => self.quitter(),
            '\x08' | '\x7f' => {
                if self.curseur_x > 0 {
                    self.curseur_x -= 1;
                } else if self.curseur_y > 0 {
                    self.curseur_y -= 1;
                    self.curseur_x = LARGEUR - 1;
                    self.ajuster_scrolling();
                }
                let idx = self.curseur_y * LARGEUR + self.curseur_x;
                self.tampon[idx] = 0;
                self.rafraichir_ecran();
            }
            '\n' | '\r' => {
                if self.curseur_y < HAUTEUR_MAX - 1 {
                    self.curseur_y += 1;
                    self.curseur_x = 0;
                    self.ajuster_scrolling();
                    self.rafraichir_ecran();
                }
            }
            caractere => {
                let idx = self.curseur_y * LARGEUR + self.curseur_x;
                if idx < TAMPON_TAILLE {
                    self.tampon[idx] = caractere as u8;
                    if self.curseur_x < LARGEUR - 1 {
                        self.curseur_x += 1;
                    } else if self.curseur_y < HAUTEUR_MAX - 1 {
                        self.curseur_x = 0;
                        self.curseur_y += 1;
                        self.ajuster_scrolling();
                    }
                    self.rafraichir_ecran();
                }
            }
        }
    }

    // --- [FONCTION 1.4 : deplacer_curseur] ---
    pub fn deplacer_curseur(&mut self, dx: isize, dy: isize) {
        if !self.actif { return; }

        if dx < 0 && self.curseur_x > 0 { self.curseur_x -= 1; }
        else if dx > 0 && self.curseur_x < LARGEUR - 1 { self.curseur_x += 1; }

        if dy < 0 && self.curseur_y > 0 {
            self.curseur_y -= 1;
        } else if dy > 0 && self.curseur_y < HAUTEUR_MAX - 1 {
            self.curseur_y += 1;
        }

        self.ajuster_scrolling();
        self.rafraichir_ecran();
    }

    // --- [FONCTION 1.5 : ajuster_scrolling] ---
    // Description : Recadre la fenêtre d'affichage (decalage_y) en fonction de la position du curseur.
    fn ajuster_scrolling(&mut self) {
        if self.curseur_y < self.decalage_y {
            self.decalage_y = self.curseur_y;
        } else if self.curseur_y >= self.decalage_y + HAUTEUR_EDIT {
            self.decalage_y = self.curseur_y - HAUTEUR_EDIT + 1;
        }
    }

   // --- [FONCTION 1.6 : rafraichir_ecran] ---
    fn rafraichir_ecran(&self) {
        affichage::effacer_ecran_complet();
        let marge_x = if self.afficher_lignes { 4 } else { 0 };

        for y_ecran in 0..HAUTEUR_EDIT {
            let y_virtuel = self.decalage_y + y_ecran;
            if y_virtuel >= HAUTEUR_MAX { break; }

            if self.afficher_lignes {
                let num = y_virtuel + 1;
                let cent = ((num / 100) % 10) as u8 + b'0';
                let diz = ((num / 10) % 10) as u8 + b'0';
                let uni = (num % 10) as u8 + b'0';
                
                // Alignement propre : espaces pour les zéros non significatifs
                affichage::ecrire_vga(0, y_ecran, if num >= 100 { cent } else { b' ' }, 0x08);
                affichage::ecrire_vga(1, y_ecran, if num >= 10 { diz } else { b' ' }, 0x08);
                affichage::ecrire_vga(2, y_ecran, uni, 0x08);
                affichage::ecrire_vga(3, y_ecran, b'|', 0x08);
            }

            for x in 0..(LARGEUR - marge_x) {
                let idx = y_virtuel * LARGEUR + x;
                let octet = self.tampon[idx];
                let caractere = if octet != 0 { octet } else { b' ' };
                let couleur = if x == self.curseur_x && y_virtuel == self.curseur_y {
                    0x2f
                } else {
                    0x02
                };
                affichage::ecrire_vga(x + marge_x, y_ecran, caractere, couleur);
            }
        }

        let nom_str = core::str::from_utf8(&self.nom_fichier[..self.nom_len]).unwrap_or("sans_titre");
        affichage::dessiner_barre_statut(nom_str);
    }

    // --- [FONCTION 1.7 : quitter] ---
    pub fn quitter(&mut self) {
        self.actif = false;
        crate::commandes::ntr::executer();
        crate::println!("[EDT] Fermeture de l'éditeur.");
    }

    // --- [FONCTION 1.8 : est_actif] ---
    pub fn est_actif(&self) -> bool {
        self.actif
    }
}

// --- [STATIC 1 : EDITEUR] ---
pub static EDITEUR: spin::Mutex<Editeur> = spin::Mutex::new(Editeur::new());
