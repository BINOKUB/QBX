// QBX EDT Affichage - Révision 0.8
// Fichier : src/commandes/edt/affichage.rs
// Description : Rendu direct en mémoire VGA (0xb8000) intégré avec Vec<u8>

use crate::commandes::edt::moteur::Editeur;

pub const LARGEUR: usize = 80;
pub const HAUTEUR_EDIT: usize = 23;
const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

// --- [FONCTION 1 : ecrire_vga] ---
// Description : Écrit directement un octet et sa couleur aux coordonnées (x, y) en mémoire vidéo.
pub fn ecrire_vga(x: usize, y: usize, octet: u8, couleur: u8) {
    let offset = (y * LARGEUR + x) * 2;
    unsafe {
        *VGA_BUFFER.add(offset) = octet;
        *VGA_BUFFER.add(offset + 1) = couleur;
    }
}

// --- [FONCTION 2 : effacer_ecran_complet] ---
// Description : Balaye l'intégralité des 25 lignes de l'écran pour le remplir d'espaces.
pub fn effacer_ecran_complet() {
    for y in 0..25 {
        for x in 0..LARGEUR {
            ecrire_vga(x, y, b' ', 0x07);
        }
    }
}

// --- [FONCTION 3 : dessiner_barre_statut] ---
// Description : Trace la ligne de séparation et affiche le nom du fichier ainsi que les raccourcis.
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

// --- [FONCTION 4 : dessiner_interface] ---
// Description : Efface l'écran, dessine l'interface et restitue le texte en liant le curseur visuel à curseur_pos.
pub fn dessiner_interface(editeur: &Editeur) {
    effacer_ecran_complet();

    let nom = core::str::from_utf8(&editeur.nom_fichier[..editeur.taille_nom]).unwrap_or("inconnu");
    dessiner_barre_statut(nom);

    let mut x = 0;
    let mut y = 0;
    
    for (i, &byte) in editeur.tampon.iter().enumerate() {
        // Applique un fond gris (0x70) si l'index correspond à la position du curseur
        let est_curseur = i == editeur.curseur_pos;
        let couleur = if est_curseur { 0x70 } else { 0x07 };
        
        if byte == b'\n' {
            if est_curseur {
                ecrire_vga(x, y, b' ', 0x70);
            }
            x = 0;
            y += 1;
        } else {
            ecrire_vga(x, y, byte, couleur);
            x += 1;
            if x >= LARGEUR {
                x = 0;
                y += 1;
            }
        }
        
        if y >= HAUTEUR_EDIT {
            break;
        }
    }
    
    // Si le curseur est placé à la toute fin du fichier (après le dernier caractère)
    if editeur.curseur_pos >= editeur.tampon.len() && y < HAUTEUR_EDIT {
        ecrire_vga(x, y, b'_', 0x0f);
    }
}
