// QBX Network Subsystem - Révision 0.1
// Fichier : src/net/mod.rs

pub mod e1000;

pub fn initialiser() {
    e1000::initialiser();
}
