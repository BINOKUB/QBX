// QBX Core - Révision 0.1
// Fichier : src/pci.rs
// Description : Scrutateur du bus PCI (Ports 0xCF8/0xCFC), énumération matérielle et détection e1000

use alloc::vec::Vec;
use x86_64::instructions::port::Port;

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

// Identifiants matériel Intel e1000 (émulé par défaut sous QEMU : 82540EM)
pub const INTEL_VENDOR_ID: u16 = 0x8086;
pub const E1000_DEV_ID: u16 = 0x100E;

#[derive(Debug, Clone, Copy)]
pub struct PeripheriquePci {
    pub bus: u8,
    pub slot: u8,
    pub fonction: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub classe: u8,
    pub sous_classe: u8,
    pub bar0: u32,
    pub irq: u8,
}

impl PeripheriquePci {
    pub fn description(&self) -> &'static str {
        match (self.classe, self.sous_classe) {
            (0x02, 0x00) => "Contrôleur Ethernet",
            (0x03, 0x00) => "Contrôleur VGA compatible",
            (0x06, 0x00) => "Pont Host/PCI",
            (0x06, 0x01) => "Pont PCI/ISA",
            (0x01, 0x01) => "Contrôleur IDE/Stockage",
            _ => "Périphérique générique",
        }
    }
}

/// Écrit l'adresse cible dans CONFIG_ADDRESS et lit 32 bits dans CONFIG_DATA
pub fn lire_config_u32(bus: u8, slot: u8, fonction: u8, decalage: u8) -> u32 {
    let adresse = 0x8000_0000u32
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((fonction as u32) << 8)
        | ((decalage as u32) & 0xFC);

    unsafe {
        let mut port_adr = Port::new(PCI_CONFIG_ADDRESS);
        let mut port_data = Port::new(PCI_CONFIG_DATA);
        port_adr.write(adresse);
        port_data.read()
    }
}

pub fn lire_config_u16(bus: u8, slot: u8, fonction: u8, decalage: u8) -> u16 {
    let mot32 = lire_config_u32(bus, slot, fonction, decalage);
    ((mot32 >> ((decalage & 2) * 8)) & 0xFFFF) as u16
}

/// Énumère les 256 bus PCI pour répertorier l'ensemble des périphériques connectés
pub fn scruter_bus() -> Vec<PeripheriquePci> {
    let mut peripheriques = Vec::new();

    for bus in 0..=3 {
        for slot in 0..32 {
            let vendor_id = lire_config_u16(bus, slot, 0, 0x00);
            if vendor_id == 0xFFFF || vendor_id == 0x0000 {
                continue; // Emplacement libre
            }

            let device_id = lire_config_u16(bus, slot, 0, 0x02);
            let classe_reg = lire_config_u32(bus, slot, 0, 0x08);
            let classe = ((classe_reg >> 24) & 0xFF) as u8;
            let sous_classe = ((classe_reg >> 16) & 0xFF) as u8;

            let bar0 = lire_config_u32(bus, slot, 0, 0x10);
            let interrupt_reg = lire_config_u32(bus, slot, 0, 0x3C);
            let irq = (interrupt_reg & 0xFF) as u8;

            peripheriques.push(PeripheriquePci {
                bus,
                slot,
                fonction: 0,
                vendor_id,
                device_id,
                classe,
                sous_classe,
                bar0,
                irq,
            });
        }
    }

    peripheriques
}

/// Recherche spécifique de la carte Intel e1000
pub fn trouver_e1000() -> Option<PeripheriquePci> {
    scruter_bus().into_iter().find(|p| p.vendor_id == INTEL_VENDOR_ID && p.device_id == E1000_DEV_ID)
}

/// Déclenche la détection au démarrage et journalise le résultat
pub fn initialiser() {
    let periphs = scruter_bus();
    crate::klog!("[PCI] Scrutateur actif : {} peripherique(s) detecte(s)", periphs.len());

    if let Some(e1000) = trouver_e1000() {
        let mmio = e1000.bar0 & !0x0F;
        crate::klog!(
            "[NET] Intel e1000 detectee sur PCI {}:{}.0 (BAR0: 0x{:08X}, IRQ: {})",
            e1000.bus,
            e1000.slot,
            mmio,
            e1000.irq
        );
    } else {
        crate::klog!("[NET] Aucune interface Intel e1000 reperee");
    }
}

/// Écrit un mot de 32 bits dans l'espace de configuration PCI
pub fn ecrire_config_u32(bus: u8, slot: u8, fonction: u8, decalage: u8, valeur: u32) {
    let adresse = 0x8000_0000u32
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((fonction as u32) << 8)
        | ((decalage as u32) & 0xFC);

    unsafe {
        let mut port_adr = Port::new(PCI_CONFIG_ADDRESS);
        let mut port_data = Port::new(PCI_CONFIG_DATA);
        port_adr.write(adresse);
        port_data.write(valeur);
    }
}

/// Écrit un mot de 16 bits sans altérer les octets adjacents
pub fn ecrire_config_u16(bus: u8, slot: u8, fonction: u8, decalage: u8, valeur: u16) {
    let ancien = lire_config_u32(bus, slot, fonction, decalage);
    let shift = (decalage & 2) * 8;
    let masque = !(0xFFFFu32 << shift);
    let nouveau = (ancien & masque) | ((valeur as u32) << shift);
    ecrire_config_u32(bus, slot, fonction, decalage, nouveau);
}

/// Active le Bus Mastering (DMA) et l'accès à l'espace mémoire (MMIO) sur le périphérique
pub fn activer_bus_master_et_memoire(bus: u8, slot: u8, fonction: u8) {
    let cmd = lire_config_u16(bus, slot, fonction, 0x04);
    // Bit 1 = Memory Space Enable, Bit 2 = Bus Master Enable
    ecrire_config_u16(bus, slot, fonction, 0x04, cmd | 0x0006);
}
