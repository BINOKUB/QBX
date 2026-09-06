// QBX Commande - Révision 0.1
// Fichier : src/commandes/ntr.rs
// Description : Nettoyage de l'écran console

use crate::vga_buffer;

// --- [FONCTION 1 : executer] ---
// Description : Déclenche l'effacement complet de la grille texte VGA.
pub fn executer() {
    vga_buffer::clear_screen();
}
