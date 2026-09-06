// QBX Shell Module - Révision 0.7
// Fichier : src/shell.rs
// Description : Gestionnaire de ligne de commande, traitement du tampon d'entrée, effacement et routage des commandes inf et tmps

use crate::{commandes, print, println, power, vga_buffer};
use spin::Mutex;
use core::iter::Iterator;

const BUFFER_SIZE: usize = 256;

// --- [STRUCTURE 1 : Shell] ---
// Description : Représente l'état de la ligne de commande avec son tampon circulaire et son pointeur de curseur.
pub struct Shell {
    buffer: [u8; BUFFER_SIZE],
    cursor: usize,
}

impl Shell {
    // --- [FONCTION 1.1 : new] ---
    // Description : Instancie un nouvel objet Shell réinitialisé avec un tampon à zéro.
    pub const fn new() -> Self {
        Shell {
            buffer: [0; BUFFER_SIZE],
            cursor: 0,
        }
    }

    // --- [FONCTION 1.2 : introduire_caractere] ---
    // Description : Reçoit un caractère saisi, gère l'exécution à la touche Entrée et le recul Backspace.
    pub fn introduire_caractere(&mut self, c: char) {
        match c {
            // Touche Entrée
            '\n' | '\r' => {
                println!();
                self.executer_commande();
                self.reinitialiser();
                print!("qbx> ");
            }
            // Touche Retour arrière (Backspace : 0x08 ou 0x7F)
            '\x08' | '\x7f' => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.buffer[self.cursor] = 0;
                    vga_buffer::ECRIVAIN.lock().effacer_dernier_caractere();
                }
            }
            // Caractères imprimables
            caractere => {
                if self.cursor < BUFFER_SIZE - 1 {
                    self.buffer[self.cursor] = caractere as u8;
                    self.cursor += 1;
                    print!("{}", caractere);
                }
            }
        }
    }

    // --- [FONCTION 1.3 : executer_commande] ---
    // Description : Découpe la ligne saisie et achemine la demande vers la commande correspondante dans src/commandes/.
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
            "aide" => {
                println!("Lexique des commandes QBX :");
                println!("  ntr  : Nettoyer l'écran");
                println!("  inf  : Informations système");
                println!("  tmps : Horloge temps réel");
                println!("  mnl  : Manuel système (ex: mnl ntr)");
                println!("  qtr  : Quitter le système");
            }
            cmd => {
                println!("Commande inconnue : '{}'", cmd);
            }
        }
    }

    // --- [FONCTION 1.4 : reinitialiser] ---
    // Description : Remet le curseur d'entrée à zéro et vide le tampon pour la prochaine commande.
    fn reinitialiser(&mut self) {
        self.buffer = [0; BUFFER_SIZE];
        self.cursor = 0;
    }
}

// --- [STATIC 1 : SHELL] ---
// Mutex global permettant l'accès sécurisé à l'instance unique du Shell depuis les interruptions.
pub static SHELL: Mutex<Shell> = Mutex::new(Shell::new());
