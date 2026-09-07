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
// Description : Efface l'écran, dessine l'interface et restitue le texte avec gestion du curseur, de l'option -l et de la sélection visuelle (F6).
pub fn dessiner_interface(editeur: &Editeur) {
    effacer_ecran_complet();

    let nom = core::str::from_utf8(&editeur.nom_fichier[..editeur.taille_nom]).unwrap_or("inconnu");
    dessiner_barre_statut(nom);

    let mut x = 0;
    let mut y = 0;
    let mut ligne_courante = 1;

    // Calcul des bornes de la sélection si F6 est actif
    let (sel_min, sel_max) = if let Some(debut) = editeur.selection_debut {
        (debut.min(editeur.curseur_pos), debut.max(editeur.curseur_pos))
    } else {
        (0, 0)
    };
    let en_selection = editeur.selection_debut.is_some();

    // Si l'option -l est activée, affichage du premier numéro de ligne
    if editeur.option_l && y < HAUTEUR_EDIT {
        let prefixe = b"1  ";
        for &b in prefixe {
            ecrire_vga(x, y, b, 0x08);
            x += 1;
        }
    }
    
    for (i, &byte) in editeur.tampon.iter().enumerate() {
        let est_curseur = i == editeur.curseur_pos;
        let dans_bloc = en_selection && i >= sel_min && i < sel_max;

        // Détermination des couleurs VGA :
        // 0x70 = Curseur (Fond gris, texte noir)
        // 0x17 = Sélection de bloc (Fond bleu, texte gris clair)
        // 0x07 = Normal
        let couleur = if est_curseur {
            0x70
        } else if dans_bloc {
            0x17
        } else {
            0x07
        };
        
        if byte == b'\n' {
            if est_curseur {
                ecrire_vga(x, y, b' ', 0x70);
            }
            x = 0;
            y += 1;
            ligne_courante += 1;

            if editeur.option_l && y < HAUTEUR_EDIT {
                let num_str = if ligne_courante < 10 {
                    alloc::format!("{}  ", ligne_courante)
                } else if ligne_courante < 100 {
                    alloc::format!("{} ", ligne_courante)
                } else {
                    alloc::format!("{}", ligne_courante)
                };

                for &b in num_str.as_bytes() {
                    ecrire_vga(x, y, b, 0x08);
                    x += 1;
                }
            }
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
    
    if editeur.curseur_pos >= editeur.tampon.len() && y < HAUTEUR_EDIT {
        ecrire_vga(x, y, b'_', 0x0f);
    }
}
