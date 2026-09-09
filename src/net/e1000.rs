// QBX Centurion - Network Driver (Intel 82540EM / e1000)
// Fichier : src/net/e1000.rs
// Description : Descripteurs alignés sur 16 octets, accès DMA volatile et compteurs matériels

use spin::Mutex;
use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::{pci, memory};

// Registres MMIO e1000
#[allow(dead_code)]
const REG_CTRL: u32   = 0x0000;
const REG_STATUS: u32 = 0x0008;

// Registres RX
const REG_RCTL: u32   = 0x0100;
const REG_RDBAL: u32  = 0x2800;
const REG_RDBAH: u32  = 0x2804;
const REG_RDLEN: u32  = 0x2808;
const REG_RDH: u32    = 0x2810;
const REG_RDT: u32    = 0x2818;

// Registres TX
const REG_TCTL: u32   = 0x0400;
const REG_TDBAL: u32  = 0x3800;
const REG_TDBAH: u32  = 0x3804;
const REG_TDLEN: u32  = 0x3808;
const REG_TDH: u32    = 0x3810;
const REG_TDT: u32    = 0x3818;

// Adresses MAC et compteurs statistiques matériels
const REG_RAL: u32    = 0x5400;
const REG_RAH: u32    = 0x5404;
const REG_GPRC: u32   = 0x4074; // Good Packets Received Count
const REG_GPTC: u32   = 0x4080; // Good Packets Transmitted Count

// Configuration RCTL
const RCTL_EN: u32     = 1 << 1;
const RCTL_SBP: u32    = 1 << 2;
const RCTL_UPE: u32    = 1 << 3;
const RCTL_MPE: u32    = 1 << 4;
const RCTL_BAM: u32    = 1 << 15;
const RCTL_BSIZE_2048: u32 = 0 << 16;
const RCTL_SECRC: u32  = 1 << 26;

// Configuration TCTL
const TCTL_EN: u32     = 1 << 1;
const TCTL_PSP: u32    = 1 << 3;
const TCTL_CT_SHIFT: u32 = 4;
const TCTL_COLD_SHIFT: u32 = 12;

const TX_CMD_EOP: u8   = 1 << 0;
const TX_CMD_IFCS: u8  = 1 << 1;
const TX_CMD_RS: u8    = 1 << 3;

const NB_DESCRIPTEURS_RX: usize = 32;
const NB_DESCRIPTEURS_TX: usize = 16;
const TAILLE_TAMPON: usize = 2048;

/// Descripteur RX matériel aligné sur 16 octets
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct DescripteurRx {
    pub adresse_tampon: u64,
    pub longueur: u16,
    pub checksum: u16,
    pub statut: u8,
    pub erreurs: u8,
    pub special: u16,
}

/// Descripteur TX matériel aligné sur 16 octets
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct DescripteurTx {
    pub adresse_tampon: u64,
    pub longueur: u16,
    pub cso: u8,
    pub cmd: u8,
    pub statut: u8,
    pub css: u8,
    pub special: u16,
}

pub struct PaquetRecu {
    pub donnees: Vec<u8>,
}

pub struct CarteE1000 {
    base_virtuelle: usize,
    pub mac: [u8; 6],
    pub lien_actif: bool,

    descripteurs_rx: Box<[DescripteurRx; NB_DESCRIPTEURS_RX]>,
    tampons_rx: Vec<Box<[u8; TAILLE_TAMPON]>>,
    index_rx: usize,

    descripteurs_tx: Box<[DescripteurTx; NB_DESCRIPTEURS_TX]>,
    tampons_tx: Vec<Box<[u8; TAILLE_TAMPON]>>,
    index_tx: usize,
}

impl CarteE1000 {
    pub unsafe fn new(base_virtuelle: usize) -> Option<Self> {
        let mut carte = CarteE1000 {
            base_virtuelle,
            mac: [0u8; 6],
            lien_actif: false,
            descripteurs_rx: Box::new([DescripteurRx {
                adresse_tampon: 0,
                longueur: 0,
                checksum: 0,
                statut: 0,
                erreurs: 0,
                special: 0,
            }; NB_DESCRIPTEURS_RX]),
            tampons_rx: Vec::with_capacity(NB_DESCRIPTEURS_RX),
            index_rx: 0,
            descripteurs_tx: Box::new([DescripteurTx {
                adresse_tampon: 0,
                longueur: 0,
                cso: 0,
                cmd: 0,
                statut: 1,
                css: 0,
                special: 0,
            }; NB_DESCRIPTEURS_TX]),
            tampons_tx: Vec::with_capacity(NB_DESCRIPTEURS_TX),
            index_tx: 0,
        };

        carte.lire_adresse_mac();
        carte.rafraichir_statut();

        if !carte.initialiser_rx() || !carte.initialiser_tx() {
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
        for _ in 0..NB_DESCRIPTEURS_RX {
            let tampon = Box::new([0u8; TAILLE_TAMPON]);
            let ptr_virtuel = tampon.as_ptr() as u64;

            if memory::traduire_virtuelle_vers_physique(ptr_virtuel).is_none() {
                return false;
            }
            self.tampons_rx.push(tampon);
        }

        for i in 0..NB_DESCRIPTEURS_RX {
            let ptr_virtuel = self.tampons_rx[i].as_ptr() as u64;
            let ptr_physique = memory::traduire_virtuelle_vers_physique(ptr_virtuel).unwrap();
            self.descripteurs_rx[i].adresse_tampon = ptr_physique;
            self.descripteurs_rx[i].statut = 0;
        }

        let ptr_desc_virt = self.descripteurs_rx.as_ptr() as u64;
        let ptr_desc_phys = match memory::traduire_virtuelle_vers_physique(ptr_desc_virt) {
            Some(p) => p,
            None => return false,
        };

        self.ecrire_reg(REG_RDBAL, (ptr_desc_phys & 0xFFFF_FFFF) as u32);
        self.ecrire_reg(REG_RDBAH, (ptr_desc_phys >> 32) as u32);

        let taille_anneau = (NB_DESCRIPTEURS_RX * core::mem::size_of::<DescripteurRx>()) as u32;
        self.ecrire_reg(REG_RDLEN, taille_anneau);

        self.ecrire_reg(REG_RDH, 0);
        self.ecrire_reg(REG_RDT, (NB_DESCRIPTEURS_RX - 1) as u32);

        let conf_rctl = RCTL_EN
            | RCTL_SBP
            | RCTL_UPE
            | RCTL_MPE
            | RCTL_BAM
            | RCTL_BSIZE_2048
            | RCTL_SECRC;

        self.ecrire_reg(REG_RCTL, conf_rctl);
        true
    }

    fn initialiser_tx(&mut self) -> bool {
        for _ in 0..NB_DESCRIPTEURS_TX {
            let tampon = Box::new([0u8; TAILLE_TAMPON]);
            let ptr_virtuel = tampon.as_ptr() as u64;

            if memory::traduire_virtuelle_vers_physique(ptr_virtuel).is_none() {
                return false;
            }
            self.tampons_tx.push(tampon);
        }

        for i in 0..NB_DESCRIPTEURS_TX {
            let ptr_virtuel = self.tampons_tx[i].as_ptr() as u64;
            let ptr_physique = memory::traduire_virtuelle_vers_physique(ptr_virtuel).unwrap();
            self.descripteurs_tx[i].adresse_tampon = ptr_physique;
            self.descripteurs_tx[i].statut = 1;
            self.descripteurs_tx[i].cmd = 0;
        }

        let ptr_desc_virt = self.descripteurs_tx.as_ptr() as u64;
        let ptr_desc_phys = match memory::traduire_virtuelle_vers_physique(ptr_desc_virt) {
            Some(p) => p,
            None => return false,
        };

        self.ecrire_reg(REG_TDBAL, (ptr_desc_phys & 0xFFFF_FFFF) as u32);
        self.ecrire_reg(REG_TDBAH, (ptr_desc_phys >> 32) as u32);

        let taille_anneau = (NB_DESCRIPTEURS_TX * core::mem::size_of::<DescripteurTx>()) as u32;
        self.ecrire_reg(REG_TDLEN, taille_anneau);

        self.ecrire_reg(REG_TDH, 0);
        self.ecrire_reg(REG_TDT, 0);

        let conf_tctl = TCTL_EN
            | TCTL_PSP
            | (15 << TCTL_CT_SHIFT)
            | (64 << TCTL_COLD_SHIFT);

        self.ecrire_reg(REG_TCTL, conf_tctl);
        true
    }

    pub fn intercepter_trame(&mut self) -> Option<PaquetRecu> {
        let desc = &mut self.descripteurs_rx[self.index_rx];
        let statut = unsafe { core::ptr::read_volatile(&desc.statut) };

        if (statut & 0x01) == 0 {
            return None;
        }

        let longueur = unsafe { core::ptr::read_volatile(&desc.longueur) } as usize;
        let mut donnees = Vec::with_capacity(longueur);
        donnees.extend_from_slice(&self.tampons_rx[self.index_rx][..longueur]);

        unsafe { core::ptr::write_volatile(&mut desc.statut, 0) };
        let ancien_index = self.index_rx;
        self.index_rx = (self.index_rx + 1) % NB_DESCRIPTEURS_RX;

        self.ecrire_reg(REG_RDT, ancien_index as u32);
        Some(PaquetRecu { donnees })
    }

    pub fn emettre_trame(&mut self, trame: &[u8]) -> bool {
        if trame.len() > TAILLE_TAMPON {
            return false;
        }

        let desc = &mut self.descripteurs_tx[self.index_tx];
        let statut = unsafe { core::ptr::read_volatile(&desc.statut) };

        if (statut & 0x01) == 0 {
            return false;
        }

        self.tampons_tx[self.index_tx][..trame.len()].copy_from_slice(trame);

        unsafe {
            core::ptr::write_volatile(&mut desc.longueur, trame.len() as u16);
            core::ptr::write_volatile(&mut desc.cmd, TX_CMD_EOP | TX_CMD_IFCS | TX_CMD_RS);
            core::ptr::write_volatile(&mut desc.statut, 0);
        }

        self.index_tx = (self.index_tx + 1) % NB_DESCRIPTEURS_TX;
        self.ecrire_reg(REG_TDT, self.index_tx as u32);
        true
    }

    pub fn relever_telemetrie(&self) -> (u32, u32, u32, u32, u32, u32) {
        (
            self.lire_reg(REG_TDH),
            self.lire_reg(REG_TDT),
            self.lire_reg(REG_GPTC),
            self.lire_reg(REG_RDH),
            self.lire_reg(REG_RDT),
            self.lire_reg(REG_GPRC),
        )
    }
}

pub static PILOTE_E1000: Mutex<Option<CarteE1000>> = Mutex::new(None);

pub fn initialiser() {
    if let Some(periph) = pci::trouver_e1000() {
        pci::activer_bus_master_et_memoire(periph.bus, periph.slot, periph.fonction);

        let base_physique = (periph.bar0 & !0x0F) as u64;
        let base_virtuelle = memory::physique_vers_virtuelle(base_physique) as usize;

        if let Some(carte) = unsafe { CarteE1000::new(base_virtuelle) } {
            crate::klog!("[NET] e1000 operationnel : RX Ring (32) + TX Ring (16) armes");
            *PILOTE_E1000.lock() = Some(carte);
        } else {
            crate::klog!("[NET] Erreur d'initialisation DMA de la carte e1000");
        }
    }
}

pub fn scruter() -> Option<PaquetRecu> {
    let mut guard = PILOTE_E1000.lock();
    guard.as_mut().and_then(|c| c.intercepter_trame())
}

pub fn envoyer(trame: &[u8]) -> bool {
    let mut guard = PILOTE_E1000.lock();
    guard.as_mut().map_or(false, |c| c.emettre_trame(trame))
}

pub fn adresse_mac() -> [u8; 6] {
    let guard = PILOTE_E1000.lock();
    guard.as_ref().map_or([0u8; 6], |c| c.mac)
}

pub fn diagnostique() -> (u32, u32, u32, u32, u32, u32) {
    let guard = PILOTE_E1000.lock();
    guard.as_ref().map_or((0, 0, 0, 0, 0, 0), |c| c.relever_telemetrie())
}
