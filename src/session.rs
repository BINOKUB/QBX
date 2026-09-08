// QBX Core - Révision 0.1
// Fichier : src/session.rs
// Description : Gestion de session, authentification hiérarchique (RBAC) et contrôle des privilèges

use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NiveauPrivilege {
    Operateur = 0,      // Prompt ">" : consultation et exploitation normale
    Administrateur = 1, // Prompt "#" : gestion des tâches et surveillance (su -adm)
    Architecte = 2,     // Prompt "!" : modification noyau, matériel et mémoire (su -arc)
}

// Mots de passe système par défaut
const MDP_ADMINISTRATEUR: &str = "admin";
const MDP_ARCHITECTE: &str = "archi";

pub struct Session {
    niveau: NiveauPrivilege,
}

impl Session {
    pub const fn new() -> Self {
        Session {
            niveau: NiveauPrivilege::Operateur,
        }
    }

    pub fn niveau(&self) -> NiveauPrivilege {
        self.niveau
    }

    pub fn a_privilege(&self, requis: NiveauPrivilege) -> bool {
        self.niveau >= requis
    }

    pub fn symbole_prompt(&self) -> char {
        match self.niveau {
            NiveauPrivilege::Operateur => '>',
            NiveauPrivilege::Administrateur => '#',
            NiveauPrivilege::Architecte => '!',
        }
    }

    pub fn tenter_elevation(&mut self, cible: NiveauPrivilege, mot_de_passe: &str) -> bool {
        let mot_de_passe_attendu = match cible {
            NiveauPrivilege::Operateur => {
                self.niveau = NiveauPrivilege::Operateur;
                return true;
            }
            NiveauPrivilege::Administrateur => MDP_ADMINISTRATEUR,
            NiveauPrivilege::Architecte => MDP_ARCHITECTE,
        };

        if mot_de_passe == mot_de_passe_attendu {
            let ancien = self.niveau;
            self.niveau = cible;
            crate::klog!(
                "[AUTH] Élévation accordée : {:?} -> {:?} (symbole '{}')",
                ancien,
                cible,
                self.symbole_prompt()
            );
            true
        } else {
            crate::klog!(
                "[AUTH] Échec d'authentification pour le niveau {:?}",
                cible
            );
            false
        }
    }

    pub fn retrograder(&mut self) -> bool {
        match self.niveau {
            NiveauPrivilege::Architecte => {
                self.niveau = NiveauPrivilege::Administrateur;
                crate::klog!("[AUTH] Rétrogradation au niveau Administrateur ('#')");
                true
            }
            NiveauPrivilege::Administrateur => {
                self.niveau = NiveauPrivilege::Operateur;
                crate::klog!("[AUTH] Rétrogradation au niveau Opérateur ('>')");
                true
            }
            NiveauPrivilege::Operateur => false,
        }
    }
}

pub static SESSION: Mutex<Session> = Mutex::new(Session::new());

pub fn niveau_actuel() -> NiveauPrivilege {
    SESSION.lock().niveau()
}

pub fn verifier_privilege(requis: NiveauPrivilege) -> bool {
    SESSION.lock().a_privilege(requis)
}

pub fn symbole_prompt() -> char {
    SESSION.lock().symbole_prompt()
}

pub fn tenter_elevation(cible: NiveauPrivilege, mot_de_passe: &str) -> bool {
    SESSION.lock().tenter_elevation(cible, mot_de_passe)
}

pub fn retrograder() -> bool {
    SESSION.lock().retrograder()
}
