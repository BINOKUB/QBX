// QBX EDT Affichage - Révision 0.6
// Fichier : src/commandes/edt/affichage.rs
// Description : Rendu direct en mémoire VGA (0xb8000) et mise en page de l'interface

pub const LARGEUR: usize = 80;
pub const HAUTEUR_EDIT: usize = 23;
const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;

// --- [FONCTION 1 : ecrire_vga] ---
// Description : Écrit directement un octet et sa couleur à la position (x, y) de la grille VGA.
pub fn ecrire_vga(x: usize, y: usize, octet: u8, couleur: u8) {
    let offset = (y * LARGEUR + x) * 2;
    unsafe {
        *VGA_BUFFER.add(offset) = octet;
        *VGA_BUFFER.add(offset + 1) = couleur;
    }
}

// --- [FONCTION 2 : effacer_ecran_complet] ---
// Description : Balaye l'intégralité des 25 lignes de la mémoire vidéo pour supprimer tout résidu.
pub fn effacer_ecran_complet() {
    for y in 0..25 {
        for x in 0..LARGEUR {
            ecrire_vga(x, y, b' ', 0x07);
        }
    }
}

// --- [FONCTION 3 : dessiner_barre_statut] ---
// Description : Trace la ligne de séparation (ligne 23) et affiche le nom du fichier sur la barre d'état (ligne 24).
pub fn dessiner_barre_statut(nom_fichier: &str) {
    // Ligne de séparation
    for x in 0..LARGEUR {
        ecrire_vga(x, 23, b'-', 0x07);
    }

    // Barre d'état
    let mut col = 0;
    let entete = "=== QBX EDT | Fichier: ";
    for b in entete.bytes() { ecrire_vga(col, 24, b, 0x0f); col += 1; }
    for b in nom_fichier.bytes() { ecrire_vga(col, 24, b, 0x0e); col += 1; }
    let suite = " | [ESC] Quitter ===";
    for b in suite.bytes() {
        if col < LARGEUR { ecrire_vga(col, 24, b, 0x0f); col += 1; }
    }
    while col < LARGEUR {
        ecrire_vga(col, 24, b' ', 0x0f);
        col += 1;
    }
}
