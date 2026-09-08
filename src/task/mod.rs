// QBX Core - Révision 0.2
// Fichier : src/task/mod.rs
// Description : Ordonnanceur coopératif Round-Robin et gestionnaire de tâches noyau

pub mod context;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use context::{ContexteTache, basculer_contexte};
use spin::Mutex;

const TAILLE_PILE: usize = 32 * 1024; // 32 Ko alloués sur le tas par tâche

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatTache {
    Prete,
    EnCours,
    Terminee,
}

pub struct Tache {
    pub id: TaskId,
    pub etat: EtatTache,
    pub contexte: ContexteTache,
    // Conserve le tampon de pile alloué sur le tas jusqu'à la destruction de la tâche
    #[allow(dead_code)]
    pile: Option<Vec<u8>>,
}

impl Tache {
    pub fn nouvelle(id: TaskId, point_entree: fn()) -> Self {
        let pile = alloc::vec![0u8; TAILLE_PILE];
        let fin_pile = pile.as_ptr() as usize + TAILLE_PILE;

        // Alignement strict de la pile sur frontière de 16 octets selon l'ABI AMD64
        let mut sommet = fin_pile & !0xF;

        // Réserve un cran pour conserver l'alignement (rsp + 8) % 16 == 0 à l'entrée de fonction
        sommet -= 8;

        // Place l'adresse de saut du 'ret' initial vers le trampoline
        sommet -= 8;
        unsafe {
            let ptr_ret = sommet as *mut usize;
            *ptr_ret = trampoline_tache as *const () as usize;
        }

        let mut contexte = ContexteTache::default();
        contexte.rsp = sommet;
        contexte.r12 = point_entree as *const () as usize;

        Tache {
            id,
            etat: EtatTache::Prete,
            contexte,
            pile: Some(pile),
        }
    }
}

// Trampoline d'entrée et de fin propre de tâche
extern "C" fn trampoline_tache() -> ! {
    let point_entree: fn();
    unsafe {
        core::arch::asm!("mov {}, r12", out(reg) point_entree);
    }

    point_entree();

    terminer_tache_courante();
    loop {
        ceder();
    }
}

pub struct Ordonnanceur {
    taches: VecDeque<Box<Tache>>,
    courante: Option<Box<Tache>>,
    prochain_id: usize,
    zombies: Vec<Box<Tache>>,
}

impl Ordonnanceur {
    pub const fn new() -> Self {
        Ordonnanceur {
            taches: VecDeque::new(),
            courante: None,
            prochain_id: 1,
            zombies: Vec::new(),
        }
    }
}

pub static ORDONNANCEUR: Mutex<Ordonnanceur> = Mutex::new(Ordonnanceur::new());

/// Initialise l'ordonnanceur en enregistrant le thread principal actuel comme Tâche 0
pub fn initialiser() {
    let mut ord = ORDONNANCEUR.lock();
    let tache_principale = Box::new(Tache {
        id: TaskId(0),
        etat: EtatTache::EnCours,
        contexte: ContexteTache::default(),
        pile: None,
    });
    ord.courante = Some(tache_principale);
    crate::klog!("[TSK] Ordonnanceur cooperatif initialise (Tache 0 active)");
}

/// Enregistre et place une nouvelle tâche dans la file d'attente
pub fn creer_tache(point_entree: fn()) -> TaskId {
    let mut ord = ORDONNANCEUR.lock();
    let id = TaskId(ord.prochain_id);
    ord.prochain_id += 1;

    let nouvelle = Box::new(Tache::nouvelle(id, point_entree));
    ord.taches.push_back(nouvelle);

    crate::klog!("[TSK] Tache {} creee et placee en file prete", id.0);
    id
}

/// Cède volontairement le processeur à la tâche suivante dans la file
pub fn ceder() {
    let (ancien_ptr, nouveau_ptr) = {
        let mut ord = ORDONNANCEUR.lock();

        // Libère les piles des tâches terminées lors du tour précédent
        ord.zombies.clear();

        if ord.taches.is_empty() {
            return;
        }

        let mut ancienne = match ord.courante.take() {
            Some(t) => t,
            None => return,
        };

        let mut nouvelle = match ord.taches.pop_front() {
            Some(t) => t,
            None => {
                ord.courante = Some(ancienne);
                return;
            }
        };

        if ancienne.etat == EtatTache::EnCours {
            ancienne.etat = EtatTache::Prete;
        }
        nouvelle.etat = EtatTache::EnCours;

        let ancien_ptr = &mut ancienne.contexte as *mut ContexteTache;
        let nouveau_ptr = &nouvelle.contexte as *const ContexteTache;

        if ancienne.etat == EtatTache::Terminee {
            ord.zombies.push(ancienne);
        } else {
            ord.taches.push_back(ancienne);
        }

        ord.courante = Some(nouvelle);

        (ancien_ptr, nouveau_ptr)
    };

    unsafe {
        basculer_contexte(ancien_ptr, nouveau_ptr);
    }
}

fn terminer_tache_courante() {
    let mut ord = ORDONNANCEUR.lock();
    if let Some(ref mut courante) = ord.courante {
        courante.etat = EtatTache::Terminee;
        crate::klog!("[TSK] Tache {} marquee terminee", courante.id.0);
    }
}
