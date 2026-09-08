// QBX Network Driver - Intel 82540EM (e1000)
// Fichier : src/net/e1000.rs
// Description : Configuration MMIO, Anneau de réception DMA (RX Ring) et Mode Promiscuous

use spin::Mutex;
use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::{pci, memory};

// Registres de contrôle de la puce Intel e1000
#[allow(dead_code)]
const REG_CTRL: u32   = 0x0000;
const REG_STATUS: u32 = 0x0008;
const REG_RCTL: u32   = 0x0100; // Receive Control
const REG_RDBAL: u32  = 0x2800; // RX Descriptor Base Low
const REG_RDBAH: u32  = 0x2804; // RX Descriptor Base High
const REG_RDLEN: u32  = 0x2808; // RX Descriptor Length
const REG_RDH: u32    = 0x2810; // RX Descriptor Head
const REG_RDT: u32    = 0x2818; // RX Descriptor Tail
const REG_RAL: u32    = 0x5400; // MAC basse
const REG_RAH: u32    = 0x5404; // MAC haute

// Bits de contrôle du registre RCTL
const RCTL_EN: u32     = 1 << 1;  // Activer le récepteur
const RCTL_SBP: u32    = 1 << 2;  // Conserver les paquets incomplets/défectueux
const RCTL_UPE: u32    = 1 << 3;  // Mode Promiscuous Unicast (capture tout)
const RCTL_MPE: u32    = 1 << 4;  // Mode Promiscuous Multicast
const RCTL_BAM: u32    = 1 << 15; // Accepter les diffusions (Broadcast)
const RCTL_BSIZE_2048: u32 = 0 << 16; // Tampons de 2048 octets
const RCTL_SECRC: u32  = 1 << 26; // Retirer le CRC Ethernet matériel

const NB_DESCRIPTEURS: usize = 32;
const TAILLE_TAMPON: usize = 2048;

/// Descripteur matériel de réception e1000 (16 octets alignés)
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct DescripteurRx {
    pub adresse_tampon: u64,
    pub longueur: u16,
    pub checksum: u16,
    pub statut: u8,
    pub erreurs: u8,
    pub special: u16,
}

pub struct PaquetRecu {
    pub donnees: Vec<u8>,
}

pub struct CarteE1000 {
    base_virtuelle: usize,
    pub mac: [u8; 6],
    pub lien_actif: bool,
    descripteurs: Box<[DescripteurRx; NB_DESCRIPTEURS]>,
    tampons: Vec<Box<[u8; TAILLE_TAMPON]>>,
    index_courant: usize,
}

impl CarteE1000 {
    pub unsafe fn new(base_virtuelle: usize) -> Option<Self> {
        let mut carte = CarteE1000 {
            base_virtuelle,
            mac: [0u8; 6],
            lien_actif: false,
            descripteurs: Box::new([DescripteurRx {
                adresse_tampon: 0,
                longueur: 0,
                checksum: 0,
                statut: 0,
                erreurs: 0,
                special: 0,
            }; NB_DESCRIPTEURS]),
            tampons: Vec::with_capacity(NB_DESCRIPTEURS),
            index_courant: 0,
        };

        carte.lire_adresse_mac();
        carte.rafraichir_statut();

        if !carte.initialiser_rx() {
            return None;
        }

        Some(carte)
    }

    #[inline]
    pub fn lire_reg(&self, decalage: u32) -> u32 {
        unsafe {
            let ptr = (self.base_virtuelle + decalage as usize) as *const u32;
            core::ptr::read_volatile(ptr)
        }
    }

    #[inline]
    pub fn ecrire_reg(&self, decalage: u32, valeur: u32) {
        unsafe {
            let ptr = (self.base_virtuelle + decalage as usize) as *mut u32;
            core::ptr::write_volatile(ptr, valeur);
        }
    }

    fn lire_adresse_mac(&mut self) {
        let ral = self.lire_reg(REG_RAL);
        let rah = self.lire_reg(REG_RAH);

        self.mac[0] = (ral & 0xFF) as u8;
        self.mac[1] = ((ral >> 8) & 0xFF) as u8;
        self.mac[2] = ((ral >> 16) & 0xFF) as u8;
        self.mac[3] = ((ral >> 24) & 0xFF) as u8;
        self.mac[4] = (rah & 0xFF) as u8;
        self.mac[5] = ((rah >> 8) & 0xFF) as u8;
    }

    pub fn rafraichir_statut(&mut self) {
        let statut = self.lire_reg(REG_STATUS);
        self.lien_actif = (statut & (1 << 1)) != 0;
    }

    fn initialiser_rx(&mut self) -> bool {
        // 1. Allocation des tampons de 2048 octets
        for _ in 0..NB_DESCRIPTEURS {
            let tampon = Box::new([0u8; TAILLE_TAMPON]);
            let ptr_virtuel = tampon.as_ptr() as u64;

            if memory::traduire_virtuelle_vers_physique(ptr_virtuel).is_none() {
                return false;
            }

            self.tampons.push(tampon);
        }

        for i in 0..NB_DESCRIPTEURS {
            let ptr_virtuel = self.tampons[i].as_ptr() as u64;
            let ptr_physique = memory::traduire_virtuelle_vers_physique(ptr_virtuel).unwrap();
            self.descripteurs[i].adresse_tampon = ptr_physique;
            self.descripteurs[i].statut = 0;
        }

        // 2. Adresse physique de base de l'anneau de descripteurs
        let ptr_desc_virt = self.descripteurs.as_ptr() as u64;
        let ptr_desc_phys = match memory::traduire_virtuelle_vers_physique(ptr_desc_virt) {
            Some(p) => p,
            None => return false,
        };

        self.ecrire_reg(REG_RDBAL, (ptr_desc_phys & 0xFFFF_FFFF) as u32);
        self.ecrire_reg(REG_RDBAH, (ptr_desc_phys >> 32) as u32);

        // 3. Taille totale de l'anneau (multiple strict de 128 octets)
        let taille_anneau = (NB_DESCRIPTEURS * core::mem::size_of::<DescripteurRx>()) as u32;
        self.ecrire_reg(REG_RDLEN, taille_anneau);

        // 4. Positionnement des index Head (0) et Tail (dernier descripteur disponible)
        self.ecrire_reg(REG_RDH, 0);
        self.ecrire_reg(REG_RDT, (NB_DESCRIPTEURS - 1) as u32);

        // 5. Activation du récepteur avec le mode Promiscuous complet
        let configuration_rctl = RCTL_EN
            | RCTL_SBP
            | RCTL_UPE
            | RCTL_MPE
            | RCTL_BAM
            | RCTL_BSIZE_2048
            | RCTL_SECRC;

        self.ecrire_reg(REG_RCTL, configuration_rctl);
        true
    }

    /// Scrutateur direct de réception : extrait une trame capturée si disponible
    pub fn intercepter_trame(&mut self) -> Option<PaquetRecu> {
        let desc = &mut self.descripteurs[self.index_courant];

        // Vérification du bit DD (Descriptor Done = 0x01)
        if (desc.statut & 0x01) == 0 {
            return None;
        }

        let longueur = desc.longueur as usize;
        let mut donnees = Vec::with_capacity(longueur);
        donnees.extend_from_slice(&self.tampons[self.index_courant][..longueur]);

        // Remise à zéro du descripteur pour le contrôleur matériel
        desc.statut = 0;
        let ancien_index = self.index_courant;
        self.index_courant = (self.index_courant + 1) % NB_DESCRIPTEURS;

        // Mise à jour de la queue matérielle (RDT) pour redonner le tampon à la carte
        self.ecrire_reg(REG_RDT, ancien_index as u32);

        Some(PaquetRecu { donnees })
    }
}

pub static PILOTE_E1000: Mutex<Option<CarteE1000>> = Mutex::new(None);

pub fn initialiser() {
    if let Some(periph) = pci::trouver_e1000() {
        pci::activer_bus_master_et_memoire(periph.bus, periph.slot, periph.fonction);

        let base_physique = (periph.bar0 & !0x0F) as u64;
        let base_virtuelle = memory::physique_vers_virtuelle(base_physique) as usize;

        if let Some(carte) = unsafe { CarteE1000::new(base_virtuelle) } {
            crate::klog!(
                "[NET] RX Ring 32 descripteurs arme en mode Promiscuous (UPE/MPE)"
            );
            *PILOTE_E1000.lock() = Some(carte);
        } else {
            crate::klog!("[NET] Erreur d'initialisation DMA de l'anneau RX");
        }
    }
}

pub fn scruter() -> Option<PaquetRecu> {
    let mut guard = PILOTE_E1000.lock();
    guard.as_mut().and_then(|c| c.intercepter_trame())
}
