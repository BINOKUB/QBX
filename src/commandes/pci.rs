// QBX - Commande pci (Audit du bus et périphériques matériels)
// Fichier : src/commandes/pci.rs

use crate::println;
use crate::pci;

pub fn executer() {
    let peripheriques = pci::scruter_bus();

    println!("--- Périphériques détectés sur le bus PCI ---");
    println!("  BUS:DEV.FN  IDENTIFIANT   BAR0 MEMOIRE   TYPE");

    for p in &peripheriques {
        let est_e1000 = p.vendor_id == pci::INTEL_VENDOR_ID && p.device_id == pci::E1000_DEV_ID;
        let marqueur = if est_e1000 { "[E1000]" } else { "" };

        println!(
            "  {:02}:{:02}.{}     {:04X}:{:04X}    0x{:08X}     {} {}",
            p.bus,
            p.slot,
            p.fonction,
            p.vendor_id,
            p.device_id,
            p.bar0 & !0x0F,
            p.description(),
            marqueur
        );
    }

    println!("---------------------------------------------");
    println!("  Total : {} périphérique(s)", peripheriques.len());
}
