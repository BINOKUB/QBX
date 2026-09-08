// QBX Core - Révision 0.4
// Fichier : src/task/mod.rs
// Description : Ordonnanceur préemptif, cadencement par interruption Timer et commutation sécurisée

pub mod context;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use context::{ContexteTache, basculer_contexte};
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

const TAILLE_PILE: usize = 32 * 1024; // 32 Ko par tâche
const QUANTUM_TICKS: usize = 1;      // Nombre de ticks PIT avant préemption (~55 ms par défaut)

static COMPTEUR_TICKS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatTache {
    Prete,
    EnCours,
    Terminee,
}

pub struct InfoTache {
    pub id: usize,
    pub nom: &'static str,
    pub etat: EtatTache,
}

pub struct Tache {
    pub id: TaskId,
    pub nom: &'static str,
    pub etat: EtatTache,
    pub contexte: ContexteTache,
    #[allow(dead_code)]
    pile: Option<Vec<u8>>,
}

impl Tache {
    pub fn nouvelle(id: TaskId, nom: &'static str, point_entree: fn()) -> Self {
        let pile = alloc::vec![0u8; TAILLE_PILE];
        let fin_pile = pile.as_ptr() as usize + TAILLE_PILE;

        let mut sommet = fin_pile & !0xF;
        sommet -= 8;
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
            nom,
            etat: EtatTache::Prete,
            contexte,
            pile: Some(pile),
        }
    }
}

// Point d'entrée des tâches : réactive les interruptions pour permettre la préemption
extern "C" fn trampoline_tache() -> ! {
    x86_64::instructions::interrupts::enable();

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

pub fn initialiser() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut ord = ORDONNANCEUR.lock();
        let tache_principale = Box::new(Tache {
            id: TaskId(0),
            nom: "noyau",
            etat: EtatTache::EnCours,
            contexte: ContexteTache::default(),
            pile: None,
        });
        ord.courante = Some(tache_principale);
        crate::klog!("[TSK] Ordonnanceur initialise (Preemption active, Tache 0)");
    });
}

pub fn creer_tache(nom: &'static str, point_entree: fn()) -> TaskId {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut ord = ORDONNANCEUR.lock();
        let id = TaskId(ord.prochain_id);
        ord.prochain_id += 1;

        let nouvelle = Box::new(Tache::nouvelle(id, nom, point_entree));
        ord.taches.push_back(nouvelle);

        crate::klog!("[TSK] Tache {} ({}) creee", id.0, nom);
        id
    })
}

pub fn lister_taches() -> Vec<InfoTache> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let ord = ORDONNANCEUR.lock();
        let mut liste = Vec::new();

        if let Some(ref courante) = ord.courante {
            liste.push(InfoTache {
                id: courante.id.0,
                nom: courante.nom,
                etat: courante.etat,
            });
        }

        for t in &ord.taches {
            liste.push(InfoTache {
                id: t.id.0,
                nom: t.nom,
                etat: t.etat,
            });
        }

        liste.sort_by_key(|t| t.id);
        liste
    })
}

/// Déclenché par l'interruption Timer PIT (IRQ 0)
pub fn cadencer_preemption() {
    let ticks = COMPTEUR_TICKS.fetch_add(1, Ordering::Relaxed) + 1;
    if ticks >= QUANTUM_TICKS {
        COMPTEUR_TICKS.store(0, Ordering::Relaxed);
        ceder();
    }
}

/// Permutation sécurisée de tâche avec verrouillage des interruptions
pub fn ceder() {
    let interruptions_etaient_actives = x86_64::instructions::interrupts::are_enabled();
    x86_64::instructions::interrupts::disable();

    let (ancien_ptr, nouveau_ptr) = {
        let mut ord = ORDONNANCEUR.lock();
        ord.zombies.clear();

        if ord.taches.is_empty() {
            if interruptions_etaient_actives {
                x86_64::instructions::interrupts::enable();
            }
            return;
        }

        let mut ancienne = match ord.courante.take() {
            Some(t) => t,
            None => {
                if interruptions_etaient_actives {
                    x86_64::instructions::interrupts::enable();
                }
                return;
            }
        };

        let mut nouvelle = match ord.taches.pop_front() {
            Some(t) => t,
            None => {
                ord.courante = Some(ancienne);
                if interruptions_etaient_actives {
                    x86_64::instructions::interrupts::enable();
                }
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

    if interruptions_etaient_actives {
        x86_64::instructions::interrupts::enable();
    }
}

fn terminer_tache_courante() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut ord = ORDONNANCEUR.lock();
        if let Some(ref mut courante) = ord.courante {
            courante.etat = EtatTache::Terminee;
            crate::klog!("[TSK] Tache {} terminee", courante.id.0);
        }
    });
}
