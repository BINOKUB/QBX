// QBX Shell Module - Révision 0.9
// Fichier : src/shell.rs
// Description : Gestionnaire de ligne de commande avec tampon d'entrée, routage edt/inf/tmps et historique de commandes via les flèches

use crate::{commandes, print, println, power, vga_buffer};
use spin::Mutex;
use core::iter::Iterator;

const BUFFER_SIZE: usize = 256;
const HISTORIQUE_TAILLE: usize = 10;

// --- [STRUCTURE 1 : Shell] ---
// Description : État de la ligne de commande avec tampon circulaire et système d'historique de navigation.
pub struct Shell {
    buffer: [u8; BUFFER_SIZE],
    cursor: usize,
    historique: [[u8; BUFFER_SIZE]; HISTORIQUE_TAILLE],
    historique_lens: [usize; HISTORIQUE_TAILLE],
    historique_count: usize,
    historique_index: usize,
}

impl Shell {
    // --- [FONCTION 1.1 : new] ---
    // Description : Instancie un nouvel objet Shell réinitialisé avec un historique vide.
    pub const fn new() -> Self {
        Shell {
            buffer: [0; BUFFER_SIZE],
            cursor: 0,
            historique: [[0; BUFFER_SIZE]; HISTORIQUE_TAILLE],
            historique_lens: [0; HISTORIQUE_TAILLE],
            historique_count: 0,
            historique_index: 0,
        }
    }

   // --- [FONCTION 1.2 : introduire_caractere] ---
    // Description : Traite la saisie des caractères, gère Entrée et Backspace pour le Shell.
    pub fn introduire_caractere(&mut self, c: char) {
        match c {
            '\n' | '\r' => {
                println!();
                self.enregistrer_dans_historique();
                self.executer_commande();
                self.reinitialiser();
                let chemin = crate::fs::chemin_actuel();
                print!("qbx:{}> ", chemin);
            }
            '\x08' | '\x7f' => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.buffer[self.cursor] = 0;
                    vga_buffer::ECRIVAIN.lock().effacer_dernier_caractere();
                }
            }
            caractere => {
                if self.cursor < BUFFER_SIZE - 1 {
                    self.buffer[self.cursor] = caractere as u8;
                    self.cursor += 1;
                    print!("{}", caractere);
                }
            }
        }
    }

    // --- [FONCTION 1.3 : historique_precedent] ---
    // Description : Rappelle la commande précédente dans l'historique lors d'un appui sur Flèche Haut.
    pub fn historique_precedent(&mut self) {
        if self.historique_count == 0 || self.historique_index == 0 {
            return;
        }

        self.historique_index -= 1;
        self.remplacer_ligne_saisie(self.historique_index);
    }

    // --- [FONCTION 1.4 : historique_suivant] ---
    // Description : Navigue vers la commande suivante (ou une ligne vide) lors d'un appui sur Flèche Bas.
    pub fn historique_suivant(&mut self) {
        if self.historique_index >= self.historique_count {
            return;
        }

        self.historique_index += 1;
        if self.historique_index == self.historique_count {
            self.effacer_ligne_courante();
            self.cursor = 0;
            self.buffer = [0; BUFFER_SIZE];
        } else {
            self.remplacer_ligne_saisie(self.historique_index);
        }
    }

    // --- [FONCTION 1.5 : remplacer_ligne_saisie] ---
    // Description : Efface la saisie en cours à l'écran et affiche la commande sélectionnée dans l'historique.
    fn remplacer_ligne_saisie(&mut self, idx: usize) {
        self.effacer_ligne_courante();
        let len = self.historique_lens[idx];
        self.buffer[..len].copy_from_slice(&self.historique[idx][..len]);
        self.cursor = len;

        if let Ok(cmd_str) = core::str::from_utf8(&self.buffer[..self.cursor]) {
            print!("{}", cmd_str);
        }
    }

    // --- [FONCTION 1.6 : effacer_ligne_courante] ---
    // Description : Efface visuellement tous les caractères saisis après le prompt 'qbx> '.
    fn effacer_ligne_courante(&mut self) {
        while self.cursor > 0 {
            vga_buffer::ECRIVAIN.lock().effacer_dernier_caractere();
            self.cursor -= 1;
        }
    }

    // --- [FONCTION 1.7 : enregistrer_dans_historique] ---
    // Description : Empile la commande courante dans le tampon d'historique s'il n'est pas vide.
    fn enregistrer_dans_historique(&mut self) {
        if self.cursor == 0 {
            return;
        }

        if self.historique_count < HISTORIQUE_TAILLE {
            let slot = self.historique_count;
            self.historique[slot][..self.cursor].copy_from_slice(&self.buffer[..self.cursor]);
            self.historique_lens[slot] = self.cursor;
            self.historique_count += 1;
        } else {
            for i in 0..(HISTORIQUE_TAILLE - 1) {
                self.historique[i] = self.historique[i + 1];
                self.historique_lens[i] = self.historique_lens[i + 1];
            }
            let slot = HISTORIQUE_TAILLE - 1;
            self.historique[slot][..self.cursor].copy_from_slice(&self.buffer[..self.cursor]);
            self.historique_lens[slot] = self.cursor;
        }
        self.historique_index = self.historique_count;
    }

    // --- [FONCTION 1.8 : executer_commande] ---
    // Description : Achemine la commande saisie vers son module d'exécution.
    fn executer_commande(&mut self) {
        let entree = core::str::from_utf8(&self.buffer[..self.cursor])
            .unwrap_or("")
            .trim();

        if entree.is_empty() {
            return;
        }

        let mut parties = entree.split_whitespace();
        let commande = parties.next().unwrap_or("");
        let argument = parties.next().unwrap_or("");

        match commande {
            "qtr" => {
                println!("[QBX] Extinction du système...");
                power::eteindre();
            }
            "ntr" => {
                commandes::ntr::executer();
            }
            "mnl" => {
                commandes::mnl::executer(argument);
            }
            "inf" => {
                commandes::inf::executer();
            }
            "tmps" => {
                commandes::tmps::executer();
            }
            "edt" => {
                // Transmet le reste de la ligne saisie après 'edt'
                let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
                commandes::edt::executer(reste_args);
            }
            "ls" => {
                commandes::ls::executer();
            }
            "cat" => {
                // Transmet l'ensemble de la ligne d'arguments après 'cat' (fichier + options)
                let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
                commandes::cat::executer(reste_args);
            }
            "ctr" => {
            let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
            commandes::ctr::executer(reste_args);
        }
            "cdr" => {
    let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
    commandes::cdr::executer(reste_args);
        }
            "aide" => {
                println!("Lexique des commandes QBX :");
                println!("  ntr  : Nettoyer l'écran");
                println!("  inf  : Informations système");
                println!("  tmps : Horloge temps réel");
                println!("  edt  : Éditeur de texte (ex: edt -l test.txt)");
                println!("  ls   : Lister les fichiers en mémoire");
                println!("  cat  : Afficher le contenu d'un fichier");
                println!("  mnl  : Manuel système (ex: mnl edt)");
                println!("  qtr  : Quitter le système");
            }
            cmd => {
                println!("Commande inconnue : '{}'", cmd);
            }
        }
    }

    // --- [FONCTION 1.9 : reinitialiser] ---
    // Description : Remet le tampon courant à zéro après exécution.
    fn reinitialiser(&mut self) {
        self.buffer = [0; BUFFER_SIZE];
        self.cursor = 0;
    }
}

// --- [STATIC 1 : SHELL] ---
// Mutex global permettant l'accès sécurisé au Shell.
pub static SHELL: Mutex<Shell> = Mutex::new(Shell::new());
