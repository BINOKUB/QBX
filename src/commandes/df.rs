// QBX Centurion - Commande df (Affichage des partitions et systèmes de fichiers)
// Fichier : src/commandes/df.rs

use crate::println;
use crate::storage::partitions;

pub fn executer() {
    println!("Volume    Point de montage  Plage LBA           Taille     Rôle / État");
    println!("─────────────────────────────────────────────────────────────────────────────");
    println!(
        "hd0s0     (boot)            {:>7}..{:<7}   1 Mo       Amorçage noyau [RO]",
        partitions::LBA_BOOT_DEBUT,
        partitions::LBA_BOOT_DEBUT + partitions::LBA_BOOT_TAILLE - 1
    );
    println!(
        "hd0s1     /sec              {:>7}..{:<7}   9 Mo       Coffre RBAC / Clés [RW]",
        partitions::LBA_SEC_DEBUT,
        partitions::LBA_SEC_DEBUT + partitions::LBA_SEC_TAILLE - 1
    );
    println!(
        "hd0s2     /var              {:>7}..{:<7}   1014 Mo    Audit & Fichiers [RW]",
        partitions::LBA_VAR_DEBUT,
        2_097_151
    );
    println!("─────────────────────────────────────────────────────────────────────────────");
    println!("Disque physique : SATA Port 0 (1024 Mo / 2097152 secteurs LBA de 512 octets)");
}
