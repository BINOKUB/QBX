// QBX Kernel Log Module
// Fichier : src/journal.rs
// Description : Tampon circulaire de messages noyau en mémoire

use spin::Mutex;
use alloc::string::String;
use alloc::vec::Vec;

const CAPACITE_MAX: usize = 64;

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

    pub fn ajouter(&mut self, message: String) {
        let index = (self.debut + self.taille) % CAPACITE_MAX;
        self.messages[index] = Some(message);

        if self.taille < CAPACITE_MAX {
            self.taille += 1;
        } else {
            self.debut = (self.debut + 1) % CAPACITE_MAX;
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

/// Enregistre une ligne dans le journal du noyau
pub fn enregistrer(message: &str) {
    let (h, m, s) = crate::commandes::tmps::obtenir_heure_actuelle();
    let ligne = alloc::format!("[{:02}:{:02}:{:02}] {}", h, m, s, message);
    JOURNAL.lock().ajouter(ligne);
}

/// Macro pratique pour enregistrer un message dans le noyau
#[macro_export]
macro_rules! klog {
    ($($arg:tt)*) => {
        $crate::journal::enregistrer(&alloc::format!($($arg)*));
    };
}
