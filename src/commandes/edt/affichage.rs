// QBX EDT Affichage - Révision 0.7
// Fichier : src/commandes/edt/affichage.rs
// Description : Rendu direct en mémoire VGA (0xb8000) intégré avec Vec<u8>

use crate::commandes::edt::moteur::Editeur;

pub const LARGEUR: usize = 80;
pub const HAUTEUR_EDIT: usize = 23;
const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

pub fn ecrire_vga(x: usize, y: usize, octet: u8, couleur: u8) {
    let offset = (y * LARGEUR + x) * 2;
    unsafe {
        *VGA_BUFFER.add(offset) = octet;
        *VGA_BUFFER.add(offset + 1) = couleur;
    }
}

pub fn effacer_ecran_complet() {
    for y in 0..25 {
        for x in 0..LARGEUR {
            ecrire_vga(x, y, b' ', 0x07);
        }
    }
}

pub fn dessiner_barre_statut(nom_fichier: &str) {
    for x in 0..LARGEUR {
        ecrire_vga(x, 23, b'-', 0x07);
    }

    let mut col = 0;
    let entete = "=== QBX EDT | Fichier: ";
    for b in entete.bytes() { ecrire_vga(col, 24, b, 0x0f); col += 1; }
    for b in nom_fichier.bytes() { ecrire_vga(col, 24, b, 0x0e); col += 1; }
    let suite = " | [F2] Sauvegarder | [ESC] Quitter ===";
    for b in suite.bytes() {
        if col < LARGEUR { ecrire_vga(col, 24, b, 0x0f); col += 1; }
    }
    while col < LARGEUR {
        ecrire_vga(col, 24, b' ', 0x0f);
        col += 1;
    }
}

pub fn dessiner_interface(editeur: &Editeur) {
    effacer_ecran_complet();

    let nom = core::str::from_utf8(&editeur.nom_fichier[..editeur.taille_nom]).unwrap_or("inconnu");
    dessiner_barre_statut(nom);

    // Affichage du tampon texte
    let mut x = 0;
    let mut y = 0;
    
    for &byte in &editeur.tampon {
        if byte == b'\n' {
            x = 0;
            y += 1;
        } else {
            ecrire_vga(x, y, byte, 0x07);
            x += 1;
            if x >= LARGEUR {
                x = 0;
                y += 1;
            }
        }
        if y >= HAUTEUR_EDIT {
            break; // Limite visuelle de l'écran
        }
    }
    
    // Curseur visuel de base
    ecrire_vga(x, y, b'_', 0x0f);
}
