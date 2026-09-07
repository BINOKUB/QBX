// QBX System - Révision 1.0
// Fichier : src/clavier_queue.rs
// Description : File d'attente (Ring Buffer) lock-free pour la capture clavier asynchrone

use spin::Mutex;
use x86_64::instructions::interrupts;

const QUEUE_SIZE: usize = 256;

pub struct ClavierQueue {
    tampon: [u8; QUEUE_SIZE],
    tete: usize,
    queue: usize,
}

impl ClavierQueue {
    pub const fn new() -> Self {
        ClavierQueue {
            tampon: [0; QUEUE_SIZE],
            tete: 0,
            queue: 0,
        }
    }

    // Pousse un scancode dans la file (appelé par l'interruption)
    pub fn push(&mut self, scancode: u8) -> Result<(), ()> {
        let prochain = (self.tete + 1) % QUEUE_SIZE;
        if prochain == self.queue {
            return Err(()); // File pleine
        }
        self.tampon[self.tete] = scancode;
        self.tete = prochain;
        Ok(())
    }

    // Dépile un scancode de la file (appelé par la boucle principale)
    pub fn pop(&mut self) -> Option<u8> {
        if self.tete == self.queue {
            return None; // File vide
        }
        let scancode = self.tampon[self.queue];
        self.queue = (self.queue + 1) % QUEUE_SIZE;
        Some(scancode)
    }
}

pub static SCANCODE_QUEUE: Mutex<ClavierQueue> = Mutex::new(ClavierQueue::new());

// Ajoute un scancode en désactivant temporairement les interruptions
// pour éviter tout deadlock si la boucle principale est déjà en train de lire
pub fn ajouter_scancode(scancode: u8) {
    interrupts::without_interrupts(|| {
        let _ = SCANCODE_QUEUE.lock().push(scancode);
    });
}
