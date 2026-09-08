//! # Module des opérations et erreurs du système de fichiers (QBX VFS)
//!
//! Définit les codes d'erreur normalisés et la catégorisation d'opération
//! inspirée de l'architecture BSD cp(1).

use alloc::string::String;
// ON ALLOUE DYNAMIQUEMENT A LA PLACE DU STATIC
// use alloc::vec::Vec;

/// Codes d'erreurs standardisés pour les opérations VFS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsError {
    SourceIntrouvable,
    DestinationIntrouvable,
    CibleExisteDeja,
    PasUnRepertoire,
    EstUnRepertoire,
    CycleDetecte,
    CheminInvalide,
}

impl VfsError {
    pub fn message(&self) -> &'static str {
        match self {
            VfsError::SourceIntrouvable => "Fichier ou repertoire source introuvable.",
            VfsError::DestinationIntrouvable => "Repertoire de destination introuvable.",
            VfsError::CibleExisteDeja => "La cible existe deja.",
            VfsError::PasUnRepertoire => "Le chemin de destination n'est pas un repertoire.",
            VfsError::EstUnRepertoire => "La source est un repertoire (utiliser -r).",
            VfsError::CycleDetecte => "Impossible de copier un repertoire a l'interieur de lui-meme.",
            VfsError::CheminInvalide => "Chemin specifie invalide.",
        }
    }
}

/// Catégorisation de l'opération de copie inspirée de l'enum `op` de FreeBSD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeOperationCopie {
    FichierVersFichier,
    VersRepertoire,
}

/// Vérifie si `chemin_cible` est un sous-chemin ou le même chemin que `chemin_source`.
///
/// Prévient la récursion infinie (ex: `cpr -r /a /a/b`).
pub fn est_descendant(chemin_source: &[String], chemin_cible: &[String]) -> bool {
    if chemin_cible.len() < chemin_source.len() {
        return false;
    }
    chemin_cible.starts_with(chemin_source)
}
