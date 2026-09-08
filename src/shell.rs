// QBX Shell Module - Révision 1.0
// Fichier : src/shell.rs
// Description : Gestionnaire de ligne de commande avec historique et redirection de flux (> et >>)

use crate::{commandes, print, println, power, vga_buffer};
use spin::Mutex;
use core::iter::Iterator;
use alloc::vec::Vec;

const BUFFER_SIZE: usize = 256;
const HISTORIQUE_TAILLE: usize = 10;

// --- [STRUCTURE 1 : Shell] ---
pub struct Shell {
    buffer: [u8; BUFFER_SIZE],
    cursor: usize,
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
            historique: [[0; BUFFER_SIZE]; HISTORIQUE_TAILLE],
            historique_lens: [0; HISTORIQUE_TAILLE],
            historique_count: 0,
            historique_index: 0,
        }
    }

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

    pub fn historique_precedent(&mut self) {
        if self.historique_count == 0 || self.historique_index == 0 {
            return;
        }

        self.historique_index -= 1;
        self.remplacer_ligne_saisie(self.historique_index);
    }

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
    // Gère le découpage des flux (> et >>) et active la capture de sortie si nécessaire.
    fn executer_commande(&mut self) {
        // On copie la saisie dans une String possédée pour libérer l'emprunt sur self.buffer
        let entree = match core::str::from_utf8(&self.buffer[..self.cursor]) {
            Ok(s) => alloc::string::String::from(s.trim()),
            Err(_) => return,
        };

        if entree.is_empty() {
            return;
        }

        // Détection des redirections
        let (commande_a_executer, redirection) = if let Some(idx) = entree.find(">>") {
            let cmd = entree[..idx].trim();
            let cible = entree[idx + 2..].trim();
            (cmd, Some((cible, true))) // true = mode append (>>)
        } else if let Some(idx) = entree.find('>') {
            let cmd = entree[..idx].trim();
            let cible = entree[idx + 1..].trim();
            (cmd, Some((cible, false))) // false = mode écrasement (>)
        } else {
            (entree.as_str(), None)
        };

        if let Some((cible_nom, est_ajout)) = redirection {
            if cible_nom.is_empty() {
                println!("qbx: Erreur de syntaxe : nom de fichier manquant pour la redirection");
                return;
            }

            // Démarrage de l'interception de sortie
            vga_buffer::demarrer_capture();

            // Exécution de la commande avec sortie redirigée
            Self::evaluer_commande(commande_a_executer);

            // Récupération de la sortie interceptée
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
            Self::evaluer_commande(commande_a_executer);
        }
    }

    // --- [FONCTION 1.9 : evaluer_commande] ---
    // Fonction associée n'empruntant pas self
    fn evaluer_commande(entree: &str) {
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
            "cpr" => {
                let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
                let args_vec: Vec<&str> = reste_args.split_whitespace().collect();
                commandes::cpr::executer(&args_vec);
            }
            "edt" => {
                let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
                commandes::edt::executer(reste_args);
            }
            "ls" => {
                commandes::ls::executer();
            }
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
            "afn" => {
                commandes::afn::executer();
            }
            "mmr" => {
                commandes::mmr::executer();
            }
            "ver" => {
                let reste_args = if entree.len() > 3 { entree[3..].trim() } else { "" };
                commandes::ver::executer(reste_args);
            }
            "aide" => {
                println!("Lexique des commandes QBX :");
                println!("  ntr  : Nettoyer l'écran");
                println!("  inf  : Informations système");
                println!("  tmps : Horloge temps réel");
                println!("  edt  : Éditeur de texte (ex: edt -l test.txt)");
                println!("  ls   : Lister les fichiers en mémoire");
                println!("  cat  : Afficher le contenu d'un fichier");
                println!("  ctr  : Créer un répertoire (ex: ctr monrep)");
                println!("  cdr  : Changer de répertoire (ex: cdr monrep, cdr ..)");
                println!("  spp  : Supprimer un fichier ou répertoire (ex: spp test.txt, spp -r monrep)");
                println!("  mnl  : Manuel système (ex: mnl edt)");
                println!("  qtr  : Quitter le système");
                println!("  rnm  : Renommer (ex: rnm f1.txt f2.txt, rnm -r rep1 rep2)");
                println!("  cpr  : Copier un fichier ou répertoire (ex: cpr f1.txt f2.txt, cpr -r rep1 rep2)");
                println!("  dpc  : Deplacer un fichier ou repertoire (ex: dpc f1.txt rep/)");
                println!("  afn  : Afficher les messages et informations du noyau");
                println!("  >    : Rediriger la sortie vers un fichier (ex: ls > liste.txt)");
                println!("  >>   : Ajouter la sortie a la fin d'un fichier (ex: afn >> journal.txt)");
                println!("  mmr  : Afficher les statistiques de la mémoire (Heap)");
                println!("  ver  : Informations système et version (ex: ver, ver -a, ver -r)");
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

// --- [STATIC 1 : SHELL] ---
pub static SHELL: Mutex<Shell> = Mutex::new(Shell::new());
