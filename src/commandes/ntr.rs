// QBX Commande - Révision 0.1
// Fichier : src/commandes/ntr.rs
// Description : Commande "nettoyer" (clear) pour effacer l'écran VGA

use crate::vga_buffer;

pub fn executer() {
    vga_buffer::clear_screen();
}
