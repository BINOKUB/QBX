// QBX Core - Révision 0.3
// Fichier : src/task/mod.rs
// Description : Ordonnanceur coopératif Round-Robin, descripteur de tâche avec nommage et API d'inspection

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

/// Initialise l'ordonnanceur en enregistrant le thread principal comme Tâche 0 ("noyau")
pub fn initialiser() {
    let mut ord = ORDONNANCEUR.lock();
    let tache_principale = Box::new(Tache {
        id: TaskId(0),
        nom: "noyau",
        etat: EtatTache::EnCours,
        contexte: ContexteTache::default(),
        pile: None,
    });
    ord.courante = Some(tache_principale);
    crate::klog!("[TSK] Ordonnanceur cooperatif initialise (Tache 0 active)");
}

/// Enregistre et place une nouvelle tâche nommée dans la file d'attente
pub fn creer_tache(nom: &'static str, point_entree: fn()) -> TaskId {
    let mut ord = ORDONNANCEUR.lock();
    let id = TaskId(ord.prochain_id);
    ord.prochain_id += 1;

    let nouvelle = Box::new(Tache::nouvelle(id, nom, point_entree));
    ord.taches.push_back(nouvelle);

    crate::klog!("[TSK] Tache {} ({}) creee et placee en file prete", id.0, nom);
    id
}

/// Extrait une photographie de l'état actuel de toutes les tâches pour inspection
pub fn lister_taches() -> Vec<InfoTache> {
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
}

pub fn ceder() {
    let (ancien_ptr, nouveau_ptr) = {
        let mut ord = ORDONNANCEUR.lock();
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
