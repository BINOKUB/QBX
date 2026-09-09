// QBX Centurion - Découpage et isolation des partitions LBA
// Fichier : src/storage/partitions.rs

use crate::storage::ahci;

// Découpage LBA standard QBX Centurion
pub const LBA_BOOT_DEBUT: u64 = 0;
pub const LBA_BOOT_TAILLE: u64 = 2048;      // 1 Mo réservé bootloader/noyau

pub const LBA_SEC_DEBUT: u64  = 2048;
pub const LBA_SEC_TAILLE: u64 = 18432;     // 9 Mo réservés pour le coffre /sec

pub const LBA_VAR_DEBUT: u64  = 20480;     // Espace journaux et captures /var

/// Lit un secteur relatif à la partition sécurisée /sec
pub fn lire_secteur_sec(offset: u64, tampon: &mut [u8; 512]) -> Result<(), &'static str> {
    if offset >= LBA_SEC_TAILLE {
        return Err("/sec: dépassement de la limite de partition");
    }
    ahci::lire_secteur(LBA_SEC_DEBUT + offset, tampon)
}

/// Écrit un secteur relatif à la partition sécurisée /sec
pub fn ecrire_secteur_sec(offset: u64, donnees: &[u8; 512]) -> Result<(), &'static str> {
    if offset >= LBA_SEC_TAILLE {
        return Err("/sec: dépassement de la limite de partition");
    }
    ahci::ecrire_secteur(LBA_SEC_DEBUT + offset, donnees)
}

/// Lit un secteur relatif à la partition de données /var
pub fn lire_secteur_var(offset: u64, tampon: &mut [u8; 512]) -> Result<(), &'static str> {
    ahci::lire_secteur(LBA_VAR_DEBUT + offset, tampon)
}

/// Écrit un secteur relatif à la partition de données /var
pub fn ecrire_secteur_var(offset: u64, donnees: &[u8; 512]) -> Result<(), &'static str> {
    ahci::ecrire_secteur(LBA_VAR_DEBUT + offset, donnees)
}
