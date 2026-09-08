// QBX Shell Module - Révision 1.5
// Fichier : src/shell.rs
// Description : Interpréteur avec historique, redirection, saisie masquée (*) et provisioning clandestin

use crate::{commandes, print, println, power, vga_buffer, session};
use spin::Mutex;
use core::iter::Iterator;
use alloc::vec::Vec;

const BUFFER_SIZE: usize = 256;
const HISTORIQUE_TAILLE: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionSecurite {
    Elevation(session::NiveauPrivilege),
    InitArchitecte,
    InitAdministrateur,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeSaisie {
    Normal,
    MotDePasse(ActionSecurite),
}

pub struct Shell {
    buffer: [u8; BUFFER_SIZE],
    cursor: usize,
    tampon_mdp: [u8; 64],
    cursor_mdp: usize,
    mode: ModeSaisie,
    historique: [[u8; BUFFER_SIZE]; HISTORIQUE_TAILLE],
    historique_lens: [usize; HISTORIQUE_TAILLE],
    historique_count: usize,
    historique_index: usize,
}

impl Shell {
    pub const fn new() -> Self {
        Shell {
            buffer: [0; BUFFER_SIZE],
            cursor: 0,
            tampon_mdp: [0; 64],
            cursor_mdp: 0,
            mode: ModeSaisie::Normal,
            historique: [[0; BUFFER_SIZE]; HISTORIQUE_TAILLE],
            historique_lens: [0; HISTORIQUE_TAILLE],
            historique_count: 0,
            historique_index: 0,
        }
    }

    pub fn afficher_prompt(&self) {
        let chemin = crate::fs::chemin_actuel();
        print!("qbx:{}{} ", chemin, session::symbole_prompt());
    }

    pub fn introduire_caractere(&mut self, c: char) {
        match self.mode {
            ModeSaisie::MotDePasse(action) => match c {
                '\n' | '\r' => {
                    println!();
                    let mdp = core::str::from_utf8(&self.tampon_mdp[..self.cursor_mdp]).unwrap_or("");
                    match action {
                        ActionSecurite::InitArchitecte => {
                            match session::sceller_cle_architecte(mdp) {
                                Ok(()) => println!("Clé Architecte scellée avec succès. Connectez-vous avec 'su -arc'."),
                                Err(e) => println!("qbx: {}", e),
                            }
                        }
                        ActionSecurite::InitAdministrateur => {
                            match session::sceller_cle_administrateur(mdp) {
                                Ok(()) => println!("Clé Administrateur définie avec succès."),
                                Err(e) => println!("qbx: {}", e),
                            }
                        }
                        ActionSecurite::Elevation(cible) => {
                            match session::tenter_elevation(cible, mdp) {
                                Ok(()) => println!("Privilèges accordés."),
                                Err(e) => println!("qbx: {}", e),
                            }
                        }
                    }
                    session::zeroiser(&mut self.tampon_mdp);
                    self.cursor_mdp = 0;
                    self.mode = ModeSaisie::Normal;
                    self.afficher_prompt();
                }
                '\x08' | '\x7f' => {
                    if self.cursor_mdp > 0 {
                        self.cursor_mdp -= 1;
                        self.tampon_mdp[self.cursor_mdp] = 0;
                        vga_buffer::ECRIVAIN.lock().effacer_dernier_caractere();
                    }
                }
                caractere if (caractere as u32) >= 0x20 && (caractere as u32) <= 0x7E => {
                    if self.cursor_mdp < 63 {
                        self.tampon_mdp[self.cursor_mdp] = caractere as u8;
                        self.cursor_mdp += 1;
                        print!("*");
                    }
                }
                _ => {}
            },
            ModeSaisie::Normal => match c {
                '\n' | '\r' => {
                    println!();
                    self.enregistrer_dans_historique();
                    self.executer_commande();
                    self.reinitialiser();
                    if self.mode == ModeSaisie::Normal {
                        self.afficher_prompt();
                    }
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
            },
        }
    }

    pub fn historique_precedent(&mut self) {
        if self.mode != ModeSaisie::Normal || self.historique_count == 0 || self.historique_index == 0 {
            return;
        }
        self.historique_index -= 1;
        self.remplacer_ligne_saisie(self.historique_index);
    }

    pub fn historique_suivant(&mut self) {
        if self.mode != ModeSaisie::Normal || self.historique_index >= self.historique_count {
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

    fn remplacer_ligne_saisie(&mut self, idx: usize) {
        self.effacer_ligne_courante();
        let len = self.historique_lens[idx];
        self.buffer[..len].copy_from_slice(&self.historique[idx][..len]);
        self.cursor = len;
        if let Ok(cmd_str) = core::str::from_utf8(&self.buffer[..self.cursor]) {
            print!("{}", cmd_str);
        }
    }

    fn effacer_ligne_courante(&mut self) {
        while self.cursor > 0 {
            vga_buffer::ECRIVAIN.lock().effacer_dernier_caractere();
            self.cursor -= 1;
        }
    }

    fn enregistrer_dans_historique(&mut self) {
        if self.cursor == 0 { return; }
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

    fn executer_commande(&mut self) {
        crate::allocator::verifier_et_etendre();
        let entree = match core::str::from_utf8(&self.buffer[..self.cursor]) {
            Ok(s) => alloc::string::String::from(s.trim()),
            Err(_) => return,
        };
        if entree.is_empty() { return; }

        let (commande_a_executer, redirection) = if let Some(idx) = entree.find(">>") {
            (entree[..idx].trim(), Some((entree[idx + 2..].trim(), true)))
        } else if let Some(idx) = entree.find('>') {
            (entree[..idx].trim(), Some((entree[idx + 1..].trim(), false)))
        } else {
            (entree.as_str(), None)
        };

        if let Some((cible_nom, est_ajout)) = redirection {
            if cible_nom.is_empty() {
                println!("qbx: Erreur de syntaxe : nom de fichier manquant pour la redirection");
                return;
            }
            vga_buffer::demarrer_capture();
            self.evaluer_commande(commande_a_executer);
            if let Some(texte_sortie) = vga_buffer::arreter_capture() {
                let mut donnees_finales = Vec::new();
                if est_ajout {
                    if let Some(existant) = crate::fs::lire(cible_nom) {
                        donnees_finales.extend_from_slice(&existant);
                    }
                }
                donnees_finales.extend_from_slice(texte_sortie.as_bytes());
                crate::fs::ecrire(cible_nom, &donnees_finales);
            }
        } else {
            self.evaluer_commande(commande_a_executer);
        }
    }

    fn evaluer_commande(&mut self, entree: &str) {
        let mut parties = entree.split_whitespace();
        let commande = parties.next().unwrap_or("");
        let argument = parties.next().unwrap_or("");

        match commande {
            "initarch" => {
                if session::est_archi_initialise() {
                    println!("qbx: Clé Architecte déjà scellée sur ce système.");
                    return;
                }
                self.mode = ModeSaisie::MotDePasse(ActionSecurite::InitArchitecte);
                self.cursor_mdp = 0;
                session::zeroiser(&mut self.tampon_mdp);
                print!("Définir mot de passe Architecte (!) : ");
            }
            "initadm" => {
                if !session::verifier_privilege(session::NiveauPrivilege::Architecte) {
                    println!("initadm: action réservée au niveau Architecte (EPERM)");
                    crate::klog!("[SEC] Tentative non autorisée d'accès à initadm");
                    return;
                }
                self.mode = ModeSaisie::MotDePasse(ActionSecurite::InitAdministrateur);
                self.cursor_mdp = 0;
                session::zeroiser(&mut self.tampon_mdp);
                print!("Définir mot de passe Administrateur (#) : ");
            }
            "su" => match argument {
                "-adm" => {
                    if !session::est_admin_initialise() {
                        println!("qbx: Compte Administrateur non configuré (initadm requis sous !).");
                        return;
                    }
                    self.mode = ModeSaisie::MotDePasse(ActionSecurite::Elevation(session::NiveauPrivilege::Administrateur));
                    self.cursor_mdp = 0;
                    session::zeroiser(&mut self.tampon_mdp);
                    print!("Mot de passe [Administrateur] : ");
                }
                "-arc" => {
                    if !session::est_archi_initialise() {
                        println!("qbx: Clé Architecte non configurée sur ce système.");
                        return;
                    }
                    self.mode = ModeSaisie::MotDePasse(ActionSecurite::Elevation(session::NiveauPrivilege::Architecte));
                    self.cursor_mdp = 0;
                    session::zeroiser(&mut self.tampon_mdp);
                    print!("Mot de passe [Architecte] : ");
                }
                "-d" => {
                    if session::retrograder() {
                        println!("Rétrogradation au mode Opérateur.");
                    } else {
                        println!("Déjà au mode Opérateur.");
                    }
                }
                _ => {
                    println!("Usage : su [-adm | -arc | -d]");
                    println!("  -adm : Élévation Administrateur ('#')");
                    println!("  -arc : Élévation Architecte ('!')");
                    println!("  -d   : Rétrogradation vers Opérateur ('>')");
                }
            },
            "exit" => {
                if session::retrograder() {
                    println!("Rétrogradation au mode Opérateur.");
                } else {
                    println!("Session minimale (Opérateur).");
                }
            }
            "qtr" => {
                if !session::verifier_privilege(session::NiveauPrivilege::Architecte) {
                    println!("qtr: extinction réservée au niveau Architecte (EPERM)");
                    crate::klog!("[SEC] Tentative d'extinction non autorisée");
                    return;
                }
                println!("[QBX] Extinction du système...");
                power::eteindre();
            }
            "ntr" => commandes::ntr::executer(),
            "mnl" => commandes::mnl::executer(argument),
            "inf" => commandes::inf::executer(),
            "tmps" => commandes::tmps::executer(),
            "cpr" => {
                let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
                let args_vec: Vec<&str> = reste_args.split_whitespace().collect();
                commandes::cpr::executer(&args_vec);
            }
            "edt" => {
                let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
                commandes::edt::executer(reste_args);
            }
            "ls" => commandes::ls::executer(),
            "cat" => {
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
            "spp" => {
                let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
                commandes::spp::executer(reste_args);
            }
            "rnm" => {
                let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
                commandes::rnm::executer(reste_args);
            }
            "dpc" => {
                let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
                let args_vec: Vec<&str> = reste_args.split_whitespace().collect();
                commandes::dpc::executer(&args_vec);
            }
            "afn" => commandes::afn::executer(),
            "tsk" => commandes::tsk::executer(),
            "mmr" => {
                let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
                commandes::mmr::executer(reste_args);
            }
            "ver" => {
                let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
                commandes::ver::executer(reste_args);
            }
            "tpf" => {
                if !session::verifier_privilege(session::NiveauPrivilege::Architecte) {
                    println!("tpf: opération réservée au niveau Architecte (EPERM)");
                    crate::klog!("[SEC] Tentative non autorisée de déclenchement Page Fault");
                    return;
                }
                println!("[QBX] Déclenchement volontaire d'un Page Fault sur 0xdeadbeef...");
                let ptr = 0xdead_beef as *mut u8;
                unsafe {
                    core::ptr::write_volatile(ptr, 42);
                }
            }
            "pci" => {
                commandes::pci::executer();
            }
            "net" => {
                commandes::net::executer();
            }
            "snf" => {
                commandes::snf::executer();
            }
            "aide" => {
                println!("Commandes QBX :");
                println!("  su       : Élévation de privilèges (su -adm, su -arc, su -d)");
                println!("  exit     : Rétrograder au mode Opérateur");
                println!("  tsk      : Lister les processus actifs");
                println!("  pci      : Auditer les périphériques du bus matériel");
                println!("  mmr      : Statistiques mémoire (-e/-t/-c réservés Architecte)");
                println!("  afn      : Journal d'audit et messages noyau");
                println!("  mnl      : Manuel système");
                println!("  pci      : Auditer les périphériques du bus matériel");
                println!("  net      : Statut de l'interface réseau et adresse MAC");
                println!("  snf      : Interception de trames réseau (mode Promiscuous)");
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
