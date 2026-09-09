// QBX Centurion - Sous-système de stockage
// Fichier : src/storage/mod.rs

pub mod ahci;
pub mod partitions;

pub fn initialiser() {
    ahci::initialiser();
}
