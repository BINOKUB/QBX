// QBX Kernel Log Module
// Fichier : src/journal.rs
// Description : Tampon circulaire de messages noyau avec découplage propre du verrou lors de la persistance sur /var

use spin::Mutex;
use alloc::string::String;
use alloc::vec::Vec;
use crate::storage::partitions;

const CAPACITE_MAX: usize = 64;
const MAGIC_JOURNAL: &[u8; 12] = b"QBX_JRNL_V1\0";

pub struct JournalNoyau {
    messages: [Option<String>; CAPACITE_MAX],
    debut: usize,
    taille: usize,
}

impl JournalNoyau {
    pub const fn new() -> Self {
        const INIT: Option<String> = None;
        JournalNoyau {
            messages: [INIT; CAPACITE_MAX],
            debut: 0,
            taille: 0,
        }
    }

    // Ajoute le message en RAM uniquement et retourne les index calculés
    pub fn inserer_en_memoire(&mut self, message: String) -> (usize, usize) {
        let index = (self.debut + self.taille) % CAPACITE_MAX;
        self.messages[index] = Some(message);

        if self.taille < CAPACITE_MAX {
            self.taille += 1;
        } else {
            self.debut = (self.debut + 1) % CAPACITE_MAX;
        }
        (index, self.taille)
    }

    pub fn charger_depuis_disque(&mut self) {
        let mut sb = [0u8; 512];
        if partitions::lire_secteur_var(0, &mut sb).is_err() {
            return;
        }

        if &sb[0..12] != MAGIC_JOURNAL {
            return;
        }

        let count = u32::from_le_bytes([sb[12], sb[13], sb[14], sb[15]]) as usize;
        let head = u32::from_le_bytes([sb[16], sb[17], sb[18], sb[19]]) as usize;

        if count == 0 || count > CAPACITE_MAX || head >= CAPACITE_MAX {
            return;
        }

        let mut entrees_chargees = 0;
        let start = (head + CAPACITE_MAX - count) % CAPACITE_MAX;

        for i in 0..count {
            let slot = (start + i) % CAPACITE_MAX;
            let mut secteur = [0u8; 512];
            if partitions::lire_secteur_var(1 + (slot as u64), &mut secteur).is_ok() {
                let len = u16::from_le_bytes([secteur[0], secteur[1]]) as usize;
                if len > 0 && len <= 508 {
                    if let Ok(s) = core::str::from_utf8(&secteur[2..2 + len]) {
                        self.messages[i] = Some(String::from(s));
                        entrees_chargees += 1;
                    }
                }
            }
        }

        if entrees_chargees > 0 {
            self.debut = 0;
            self.taille = entrees_chargees;
            crate::println!("[VAR] Journal /var : {} entrée(s) d'audit restaurée(s).", entrees_chargees);
        }
    }

    pub fn lire_tous(&self) -> Vec<String> {
        let mut sortie = Vec::with_capacity(self.taille);
        for i in 0..self.taille {
            let index = (self.debut + i) % CAPACITE_MAX;
            if let Some(ref msg) = self.messages[index] {
                sortie.push(msg.clone());
            }
        }
        sortie
    }
}

pub static JOURNAL: Mutex<JournalNoyau> = Mutex::new(JournalNoyau::new());

pub fn charger_depuis_disque() {
    JOURNAL.lock().charger_depuis_disque();
}

pub fn enregistrer(message: &str) {
    let (h, m, s) = crate::commandes::tmps::obtenir_heure_actuelle();
    let ligne = alloc::format!("[{:02}:{:02}:{:02}] {}", h, m, s, message);

    // 1. Mise à jour RAM sous verrou éphémère
    let (index, taille_actuelle) = {
        let mut j = JOURNAL.lock();
        j.inserer_en_memoire(ligne.clone())
    }; // Le verrou JOURNAL est libéré ici

    // 2. Persistance disque en dehors de tout verrouillage
    persister_sur_disque(index, &ligne, taille_actuelle);
}

fn persister_sur_disque(index: usize, message: &str, taille_totale: usize) {
    let mut secteur = [0u8; 512];
    let octets = message.as_bytes();
    let len = octets.len().min(508);
    secteur[0..2].copy_from_slice(&(len as u16).to_le_bytes());
    secteur[2..2 + len].copy_from_slice(&octets[..len]);

    if partitions::ecrire_secteur_var(1 + (index as u64), &secteur).is_err() {
        return;
    }

    let mut sb = [0u8; 512];
    sb[0..12].copy_from_slice(MAGIC_JOURNAL);
    let head = ((index + 1) % CAPACITE_MAX) as u32;
    let count = taille_totale.min(CAPACITE_MAX) as u32;
    sb[12..16].copy_from_slice(&count.to_le_bytes());
    sb[16..20].copy_from_slice(&head.to_le_bytes());
    let _ = partitions::ecrire_secteur_var(0, &sb);
}

#[macro_export]
macro_rules! klog {
    ($($arg:tt)*) => {
        $crate::journal::enregistrer(&alloc::format!($($arg)*));
    };
}
