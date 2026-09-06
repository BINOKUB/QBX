// QBX Commande - Révision 0.1
// Fichier : src/commandes/inf.rs
// Description : Affiche les informations système et l'architecture du noyau QBX

use crate::println;

// --- [FONCTION 1 : executer] ---
// Description : Affiche les caractéristiques matérielles et logicielles du système.
pub fn executer() {
    println!("=== INFORMATIONS SYSTÈME QBX ===");
    println!("Système d'exploitation : QBX (Québec UNIX)");
    println!("Version                 : 0.1.0-baremetal");
    println!("Architecture            : x86_64 (64-bit)");
    println!("Mode d'affichage        : Texte VGA 80x25 (CP437)");
    println!("Gestionnaire d'energie  : ACPI / APM actif");
    println!("=================================");
}
