// QBX Shell Module - Révision 0.4
// Fichier : src/shell.rs
// Description : Gestionnaire de ligne de commande, effacement et routage vers src/commandes/

use crate::{commandes, print, println, power, vga_buffer};
use spin::Mutex;

const BUFFER_SIZE: usize = 256;

pub struct Shell {
    buffer: [u8; BUFFER_SIZE],
    cursor: usize,
}

impl Shell {
    pub const fn new() -> Self {
        Shell {
            buffer: [0; BUFFER_SIZE],
            cursor: 0,
        }
    }

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

    fn executer_commande(&mut self) {
        let entree = core::str::from_utf8(&self.buffer[..self.cursor])
            .unwrap_or("")
            .trim();

        if entree.is_empty() {
            return;
        }

        let mut parties = entree.split_whitespace();
        let commande = parties.next().unwrap_or("");

        match commande {
            // Extinction du système
            "qtr" => {
                println!("[QBX] Extinction du système...");
                power::eteindre();
            }
            // Nettoyage de l'écran
            "ntr" => {
                commandes::ntr::executer();
            }
            "aide" => {
                println!("Lexique des commandes QBX :");
                println!("  ntr  : Nettoyer l'écran");
                println!("  qtr  : Quitter le système");
                println!("  aide : Afficher ce menu");
            }
            cmd => {
                println!("Commande inconnue : '{}'", cmd);
            }
        }
    }

    fn reinitialiser(&mut self) {
        self.buffer = [0; BUFFER_SIZE];
        self.cursor = 0;
    }
}

pub static SHELL: Mutex<Shell> = Mutex::new(Shell::new());
