// QBX Core - Révision 0.3
// Fichier : src/session.rs
// Description : RBAC, provisioning à froid (initarch/initadm), SHA-256 stack-only et purge volatile

use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NiveauPrivilege {
    Operateur = 0,      // Prompt ">" : aucun mot de passe, outils d'analyse et surveillance
    Administrateur = 1, // Prompt "#" : gestion opérationnelle (su -adm)
    Architecte = 2,     // Prompt "!" : contrôle absolu du matériel et du noyau (su -arc)
}

pub struct Session {
    niveau: NiveauPrivilege,
    hash_admin: Option<[u8; 32]>,
    hash_archi: Option<[u8; 32]>,
}

impl Session {
    pub const fn new() -> Self {
        Session {
            niveau: NiveauPrivilege::Operateur,
            hash_admin: None,
            hash_archi: None,
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

    pub fn est_archi_initialise(&self) -> bool {
        self.hash_archi.is_some()
    }

    pub fn est_admin_initialise(&self) -> bool {
        self.hash_admin.is_some()
    }

    // Scelle définitivement la clé Architecte à l'installation (une seule exécution possible)
    pub fn sceller_cle_architecte(&mut self, mdp: &str) -> Result<(), &'static str> {
        if self.hash_archi.is_some() {
            return Err("Clé Architecte déjà scellée. Commande désactivée.");
        }
        if mdp.len() < 4 {
            return Err("Mot de passe trop court (4 caractères minimum).");
        }
        self.hash_archi = Some(sha256(mdp.as_bytes()));
        crate::klog!("[SEC] Clé de sécurité Architecte scellée avec succès");
        Ok(())
    }

    // Définit la clé Administrateur (requiert le privilège Architecte actif)
    pub fn sceller_cle_administrateur(&mut self, mdp: &str) -> Result<(), &'static str> {
        if self.niveau != NiveauPrivilege::Architecte {
            return Err("EPERM : Seul l'Architecte (!) peut définir la clé Administrateur.");
        }
        if mdp.len() < 4 {
            return Err("Mot de passe trop court (4 caractères minimum).");
        }
        self.hash_admin = Some(sha256(mdp.as_bytes()));
        crate::klog!("[SEC] Clé de sécurité Administrateur scellée par l'Architecte");
        Ok(())
    }

    pub fn tenter_elevation(&mut self, cible: NiveauPrivilege, mot_de_passe: &str) -> Result<(), &'static str> {
        let hash_attendu = match cible {
            NiveauPrivilege::Operateur => {
                self.niveau = NiveauPrivilege::Operateur;
                return Ok(());
            }
            NiveauPrivilege::Administrateur => match &self.hash_admin {
                Some(h) => h,
                None => return Err("Clé Administrateur non initialisée (l'Architecte doit exécuter 'initadm')."),
            },
            NiveauPrivilege::Architecte => match &self.hash_archi {
                Some(h) => h,
                None => return Err("Clé Architecte non configurée. Tapez 'initarch' pour l'initialiser."),
            },
        };

        let hash_saisi = sha256(mot_de_passe.as_bytes());

        if comparaison_temps_constant(&hash_saisi, hash_attendu) {
            let ancien = self.niveau;
            self.niveau = cible;
            crate::klog!(
                "[AUTH] Élévation accordée : {:?} -> {:?} ('{}')",
                ancien,
                cible,
                self.symbole_prompt()
            );
            Ok(())
        } else {
            crate::klog!("[AUTH] Échec d'authentification pour {:?}", cible);
            Err("Mot de passe incorrect.")
        }
    }

    pub fn retrograder(&mut self) -> bool {
        if self.niveau != NiveauPrivilege::Operateur {
            self.niveau = NiveauPrivilege::Operateur;
            crate::klog!("[AUTH] Rétrogradation au niveau Opérateur ('>')");
            true
        } else {
            false
        }
    }
}

pub static SESSION: Mutex<Session> = Mutex::new(Session::new());

pub fn verifier_privilege(requis: NiveauPrivilege) -> bool {
    SESSION.lock().a_privilege(requis)
}

pub fn symbole_prompt() -> char {
    SESSION.lock().symbole_prompt()
}

pub fn est_archi_initialise() -> bool {
    SESSION.lock().est_archi_initialise()
}

pub fn est_admin_initialise() -> bool {
    SESSION.lock().est_admin_initialise()
}

pub fn sceller_cle_architecte(mdp: &str) -> Result<(), &'static str> {
    SESSION.lock().sceller_cle_architecte(mdp)
}

pub fn sceller_cle_administrateur(mdp: &str) -> Result<(), &'static str> {
    SESSION.lock().sceller_cle_administrateur(mdp)
}

pub fn tenter_elevation(cible: NiveauPrivilege, mdp: &str) -> Result<(), &'static str> {
    SESSION.lock().tenter_elevation(cible, mdp)
}

pub fn retrograder() -> bool {
    SESSION.lock().retrograder()
}

/// Écrasement immédiat des octets de la pile avec des zéros
pub fn zeroiser(tampon: &mut [u8]) {
    for octet in tampon.iter_mut() {
        unsafe { core::ptr::write_volatile(octet, 0) };
    }
}

fn comparaison_temps_constant(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff: u8 = 0;
    for i in 0..32 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// SHA-256 optimisé stack-only (aucune allocation heap, pour mots de passe <= 55 octets)
pub fn sha256(donnees: &[u8]) -> [u8; 32] {
    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let mut bloc = [0u8; 64];
    let taille = donnees.len().min(55);
    bloc[..taille].copy_from_slice(&donnees[..taille]);
    bloc[taille] = 0x80;

    let bits = (taille as u64) * 8;
    bloc[56..64].copy_from_slice(&bits.to_be_bytes());

    let mut w = [0u32; 64];
    for i in 0..16 {
        w[i] = u32::from_be_bytes([bloc[i * 4], bloc[i * 4 + 1], bloc[i * 4 + 2], bloc[i * 4 + 3]]);
    }
    for i in 16..64 {
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
    }

    let mut a = h[0]; let mut b = h[1]; let mut c = h[2]; let mut d = h[3];
    let mut e = h[4]; let mut f = h[5]; let mut g = h[6]; let mut h_v = h[7];

    for i in 0..64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ ((!e) & g);
        let temp1 = h_v.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);

        h_v = g; g = f; f = e; e = d.wrapping_add(temp1);
        d = c; c = b; b = a; a = temp1.wrapping_add(temp2);
    }

    h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b);
    h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f);
    h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(h_v);

    let mut sortie = [0u8; 32];
    for i in 0..8 {
        sortie[i * 4..(i + 1) * 4].copy_from_slice(&h[i].to_be_bytes());
    }
    sortie
}
