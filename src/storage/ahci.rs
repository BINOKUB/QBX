// QBX Centurion - Pilote de stockage SATA / AHCI
// Fichier : src/storage/ahci.rs
// Description : Moteur DMA sécurisé avec tampon de rebond aligné, timeout et mode scrutation (sans tempête IRQ)

use crate::println;
use x86_64::instructions::port::Port;

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

const CLASSE_STOCKAGE: u8 = 0x01;
const SOUS_CLASSE_SATA: u8 = 0x06;
const PROG_IF_AHCI: u8 = 0x01;

const ATA_CMD_IDENTIFY: u8 = 0xEC;
const ATA_CMD_READ_DMA_EXT: u8 = 0x25;
const ATA_CMD_WRITE_DMA_EXT: u8 = 0x35;
const FIS_TYPE_REG_H2D: u8 = 0x27;

#[repr(C, align(1024))]
struct PortCommandList([u8; 1024]);

#[repr(C, align(256))]
struct PortReceivedFis([u8; 256]);

#[repr(C, align(128))]
struct PortCommandTable {
    cfis: [u8; 64],
    acmd: [u8; 16],
    rsv: [u8; 48],
    prdt: [PrdtEntry; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct PrdtEntry {
    dba: u32,
    dbau: u32,
    rsv0: u32,
    dbc: u32,
}

// Tampon DMA statique garanti aligné sur 512 octets (ne traverse jamais une frontière de page 4 Ko)
#[repr(C, align(512))]
struct DmaBounceBuffer([u8; 512]);

static mut CMD_LIST: PortCommandList = PortCommandList([0; 1024]);
static mut RECV_FIS: PortReceivedFis = PortReceivedFis([0; 256]);
static mut CMD_TABLE: PortCommandTable = PortCommandTable {
    cfis: [0; 64],
    acmd: [0; 16],
    rsv: [0; 48],
    prdt: [PrdtEntry { dba: 0, dbau: 0, rsv0: 0, dbc: 0 }; 1],
};
static mut DMA_BOUNCE_BUF: DmaBounceBuffer = DmaBounceBuffer([0; 512]);

static mut PORT_0_BASE: usize = 0;

fn pci_lire_u32(bus: u8, slot: u8, fonction: u8, offset: u8) -> u32 {
    let adresse = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((fonction as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x8000_0000;

    unsafe {
        let mut port_addr = Port::new(PCI_CONFIG_ADDRESS);
        let mut port_data = Port::new(PCI_CONFIG_DATA);
        port_addr.write(adresse);
        port_data.read()
    }
}

fn pci_ecrire_u32(bus: u8, slot: u8, fonction: u8, offset: u8, valeur: u32) {
    let adresse = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((fonction as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x8000_0000;

    unsafe {
        let mut port_addr = Port::new(PCI_CONFIG_ADDRESS);
        let mut port_data = Port::new(PCI_CONFIG_DATA);
        port_addr.write(adresse);
        port_data.write(valeur);
    }
}

pub fn initialiser() {
    println!("[AHCI] Analyse du bus PCI à la recherche du contrôleur SATA...");

    for bus in 0..=255 {
        for slot in 0..32 {
            let id = pci_lire_u32(bus, slot, 0, 0x00);
            if (id & 0xFFFF) == 0xFFFF {
                continue;
            }

            let class_rev = pci_lire_u32(bus, slot, 0, 0x08);
            let classe = ((class_rev >> 24) & 0xFF) as u8;
            let sous_classe = ((class_rev >> 16) & 0xFF) as u8;
            let prog_if = ((class_rev >> 8) & 0xFF) as u8;

            if classe == CLASSE_STOCKAGE && sous_classe == SOUS_CLASSE_SATA && prog_if == PROG_IF_AHCI {
                let vendor_id = (id & 0xFFFF) as u16;
                let device_id = ((id >> 16) & 0xFFFF) as u16;

                println!("[AHCI] Contrôleur détecté sur PCI {:02X}:{:02X}.0", bus, slot);
                println!("       Identifiants : Vendor 0x{:04X}, Device 0x{:04X}", vendor_id, device_id);

                let mut cmd = pci_lire_u32(bus, slot, 0, 0x04);
                cmd |= (1 << 1) | (1 << 2);
                pci_ecrire_u32(bus, slot, 0, 0x04, cmd);

                let abar_phys = pci_lire_u32(bus, slot, 0, 0x24) & 0xFFFF_FFF0;
                let abar_virt = crate::memory::physique_vers_virtuelle(abar_phys as u64) as usize;

                configurer_controleur(abar_virt);
                return;
            }
        }
    }

    println!("[AHCI] Aucun contrôleur AHCI SATA détecté sur le bus.");
}

fn configurer_controleur(base_abar: usize) {
    let ptr_pi = (base_abar + 0x0C) as *const u32;

    unsafe {
        let pi = core::ptr::read_volatile(ptr_pi);

        if (pi & 1) != 0 {
            let port_base = base_abar + 0x100;
            let ssts = core::ptr::read_volatile((port_base + 0x28) as *const u32);
            let det = ssts & 0x0F;
            let ipm = (ssts >> 8) & 0x0F;

            if det == 3 && ipm == 1 {
                println!("[AHCI] Initialisation du moteur DMA sur le Port 0...");
                if initialiser_port_dma(port_base) {
                    interroger_disque(port_base);
                    core::ptr::write_volatile(core::ptr::addr_of_mut!(PORT_0_BASE), port_base);
                }
            }
        }
    }
}

unsafe fn arreter_port_dma(port: usize) {
    let ptr_cmd = (port + 0x18) as *mut u32;

    let mut cmd = core::ptr::read_volatile(ptr_cmd);
    cmd &= !(1 << 0);
    cmd &= !(1 << 4);
    core::ptr::write_volatile(ptr_cmd, cmd);

    let mut timeout = 100_000;
    loop {
        let val = core::ptr::read_volatile(ptr_cmd);
        if (val & ((1 << 15) | (1 << 14))) == 0 || timeout == 0 {
            break;
        }
        timeout -= 1;
    }
}

unsafe fn demarrer_port_dma(port: usize) {
    let ptr_cmd = (port + 0x18) as *mut u32;

    let mut timeout = 100_000;
    while (core::ptr::read_volatile(ptr_cmd) & (1 << 15)) != 0 && timeout > 0 {
        timeout -= 1;
    }

    let mut cmd = core::ptr::read_volatile(ptr_cmd);
    cmd |= 1 << 4;
    cmd |= 1 << 0;
    core::ptr::write_volatile(ptr_cmd, cmd);
}

unsafe fn initialiser_port_dma(port: usize) -> bool {
    arreter_port_dma(port);

    let ptr_cmd_list = core::ptr::addr_of!(CMD_LIST) as u64;
    let ptr_recv_fis = core::ptr::addr_of!(RECV_FIS) as u64;

    let phys_cmd_list = match crate::memory::traduire_virtuelle_vers_physique(ptr_cmd_list) {
        Some(addr) => addr,
        None => return false,
    };
    let phys_recv_fis = match crate::memory::traduire_virtuelle_vers_physique(ptr_recv_fis) {
        Some(addr) => addr,
        None => return false,
    };

    core::ptr::write_volatile(port as *mut u32, phys_cmd_list as u32);
    core::ptr::write_volatile((port + 0x04) as *mut u32, (phys_cmd_list >> 32) as u32);

    core::ptr::write_volatile((port + 0x08) as *mut u32, phys_recv_fis as u32);
    core::ptr::write_volatile((port + 0x0C) as *mut u32, (phys_recv_fis >> 32) as u32);

    core::ptr::write_volatile((port + 0x10) as *mut u32, 0xFFFF_FFFF);
    core::ptr::write_volatile((port + 0x30) as *mut u32, 0xFFFF_FFFF);

    demarrer_port_dma(port);
    true
}

unsafe fn interroger_disque(port: usize) {
    let ptr_cmd_table = core::ptr::addr_of!(CMD_TABLE) as u64;
    let ptr_bounce = core::ptr::addr_of!(DMA_BOUNCE_BUF) as u64;

    let phys_cmd_table = match crate::memory::traduire_virtuelle_vers_physique(ptr_cmd_table) {
        Some(a) => a,
        None => return,
    };
    let phys_bounce = match crate::memory::traduire_virtuelle_vers_physique(ptr_bounce) {
        Some(a) => a,
        None => return,
    };

    let header_ptr = core::ptr::addr_of_mut!(CMD_LIST.0) as *mut u32;
    core::ptr::write_volatile(header_ptr, 5 | (1 << 16));
    core::ptr::write_volatile(header_ptr.add(1), 0);
    core::ptr::write_volatile(header_ptr.add(2), phys_cmd_table as u32);
    core::ptr::write_volatile(header_ptr.add(3), (phys_cmd_table >> 32) as u32);

    let prdt_ptr = core::ptr::addr_of_mut!(CMD_TABLE.prdt) as *mut PrdtEntry;
    core::ptr::write_volatile(prdt_ptr, PrdtEntry {
        dba: phys_bounce as u32,
        dbau: (phys_bounce >> 32) as u32,
        rsv0: 0,
        dbc: 512 - 1, // Pas de bit 31 : pas d'interruption
    });

    let cfis_ptr = core::ptr::addr_of_mut!(CMD_TABLE.cfis) as *mut u8;
    for i in 0..64 { core::ptr::write_volatile(cfis_ptr.add(i), 0); }
    core::ptr::write_volatile(cfis_ptr.add(0), FIS_TYPE_REG_H2D);
    core::ptr::write_volatile(cfis_ptr.add(1), 0x80);
    core::ptr::write_volatile(cfis_ptr.add(2), ATA_CMD_IDENTIFY);
    core::ptr::write_volatile(cfis_ptr.add(3), 0);
    core::ptr::write_volatile(cfis_ptr.add(7), 0);

    let ptr_tfd = (port + 0x20) as *const u32;
    let mut timeout = 100_000;
    while (core::ptr::read_volatile(ptr_tfd) & (0x80 | 0x08)) != 0 && timeout > 0 {
        timeout -= 1;
    }

    let ptr_ci = (port + 0x38) as *mut u32;
    core::ptr::write_volatile(ptr_ci, 1);

    timeout = 1_000_000;
    loop {
        let ci = core::ptr::read_volatile(ptr_ci);
        if (ci & 1) == 0 { break; }
        if (core::ptr::read_volatile((port + 0x10) as *const u32) & (1 << 30)) != 0 || timeout == 0 {
            return;
        }
        timeout -= 1;
        core::hint::spin_loop();
    }

    let raw_ptr = core::ptr::addr_of!(DMA_BOUNCE_BUF.0) as *const u16;

    let mut model_bytes = [0u8; 40];
    for i in 0..20 {
        let word = core::ptr::read(raw_ptr.add(27 + i));
        model_bytes[i * 2] = (word >> 8) as u8;
        model_bytes[i * 2 + 1] = (word & 0xFF) as u8;
    }
    let model = core::str::from_utf8(&model_bytes).unwrap_or("Inconnu").trim();

    let mut serial_bytes = [0u8; 20];
    for i in 0..10 {
        let word = core::ptr::read(raw_ptr.add(10 + i));
        serial_bytes[i * 2] = (word >> 8) as u8;
        serial_bytes[i * 2 + 1] = (word & 0xFF) as u8;
    }
    let serial = core::str::from_utf8(&serial_bytes).unwrap_or("Inconnu").trim();

    let w100 = core::ptr::read(raw_ptr.add(100)) as u64;
    let w101 = core::ptr::read(raw_ptr.add(101)) as u64;
    let w102 = core::ptr::read(raw_ptr.add(102)) as u64;
    let w103 = core::ptr::read(raw_ptr.add(103)) as u64;
    let secteurs_lba48 = w100 | (w101 << 16) | (w102 << 32) | (w103 << 48);

    let total_secteurs = if secteurs_lba48 > 0 {
        secteurs_lba48
    } else {
        let w60 = core::ptr::read(raw_ptr.add(60)) as u64;
        let w61 = core::ptr::read(raw_ptr.add(61)) as u64;
        w60 | (w61 << 16)
    };

    let taille_mo = (total_secteurs * 512) / (1024 * 1024);

    println!("       ┌── Disque SATA Port 0 ────────────────────────┐");
    println!("       │ Modèle    : {:<33}│", model);
    println!("       │ Série     : {:<33}│", serial);
    println!("       │ Capacité  : {} Mo ({} secteurs LBA)  │", taille_mo, total_secteurs);
    println!("       └──────────────────────────────────────────────┘");
}

pub fn lire_secteur(lba: u64, tampon: &mut [u8; 512]) -> Result<(), &'static str> {
    unsafe {
        let port = core::ptr::read_volatile(core::ptr::addr_of!(PORT_0_BASE));
        if port == 0 {
            return Err("Port AHCI 0 non initialisé");
        }
        let bounce_ptr = core::ptr::addr_of_mut!(DMA_BOUNCE_BUF.0) as *mut u8;
        executer_transfert_dma(port, lba, bounce_ptr, false)?;
        core::ptr::copy_nonoverlapping(bounce_ptr, tampon.as_mut_ptr(), 512);
        Ok(())
    }
}

pub fn ecrire_secteur(lba: u64, donnees: &[u8; 512]) -> Result<(), &'static str> {
    unsafe {
        let port = core::ptr::read_volatile(core::ptr::addr_of!(PORT_0_BASE));
        if port == 0 {
            return Err("Port AHCI 0 non initialisé");
        }
        let bounce_ptr = core::ptr::addr_of_mut!(DMA_BOUNCE_BUF.0) as *mut u8;
        core::ptr::copy_nonoverlapping(donnees.as_ptr(), bounce_ptr, 512);
        executer_transfert_dma(port, lba, bounce_ptr, true)
    }
}

unsafe fn executer_transfert_dma(port: usize, lba: u64, buf_virt: *mut u8, est_ecriture: bool) -> Result<(), &'static str> {
    let phys_cmd_table = crate::memory::traduire_virtuelle_vers_physique(core::ptr::addr_of!(CMD_TABLE) as u64)
        .ok_or("Échec résolution physique Command Table")?;
    let phys_buf = crate::memory::traduire_virtuelle_vers_physique(buf_virt as u64)
        .ok_or("Échec résolution physique tampon de données")?;

    let header_ptr = core::ptr::addr_of_mut!(CMD_LIST.0) as *mut u32;
    let mut flags: u32 = 5 | (1 << 16);
    if est_ecriture {
        flags |= 1 << 6;
    }
    core::ptr::write_volatile(header_ptr, flags);
    core::ptr::write_volatile(header_ptr.add(1), 0);
    core::ptr::write_volatile(header_ptr.add(2), phys_cmd_table as u32);
    core::ptr::write_volatile(header_ptr.add(3), (phys_cmd_table >> 32) as u32);

    let prdt_ptr = core::ptr::addr_of_mut!(CMD_TABLE.prdt) as *mut PrdtEntry;
    core::ptr::write_volatile(prdt_ptr, PrdtEntry {
        dba: phys_buf as u32,
        dbau: (phys_buf >> 32) as u32,
        rsv0: 0,
        dbc: 512 - 1, // DBC sans bit 31 : désactivation de l'interruption matérielle
    });

    let cfis_ptr = core::ptr::addr_of_mut!(CMD_TABLE.cfis) as *mut u8;
    for i in 0..64 { core::ptr::write_volatile(cfis_ptr.add(i), 0); }

    core::ptr::write_volatile(cfis_ptr.add(0), FIS_TYPE_REG_H2D);
    core::ptr::write_volatile(cfis_ptr.add(1), 0x80);
    core::ptr::write_volatile(
        cfis_ptr.add(2),
        if est_ecriture { ATA_CMD_WRITE_DMA_EXT } else { ATA_CMD_READ_DMA_EXT },
    );
    core::ptr::write_volatile(cfis_ptr.add(7), 1 << 6);

    core::ptr::write_volatile(cfis_ptr.add(4), (lba & 0xFF) as u8);
    core::ptr::write_volatile(cfis_ptr.add(5), ((lba >> 8) & 0xFF) as u8);
    core::ptr::write_volatile(cfis_ptr.add(6), ((lba >> 16) & 0xFF) as u8);
    core::ptr::write_volatile(cfis_ptr.add(8), ((lba >> 24) & 0xFF) as u8);
    core::ptr::write_volatile(cfis_ptr.add(9), ((lba >> 32) & 0xFF) as u8);
    core::ptr::write_volatile(cfis_ptr.add(10), ((lba >> 40) & 0xFF) as u8);

    core::ptr::write_volatile(cfis_ptr.add(12), 1);
    core::ptr::write_volatile(cfis_ptr.add(13), 0);

    // Acquittement des statuts résiduels
    core::ptr::write_volatile((port + 0x10) as *mut u32, 0xFFFF_FFFF);

    let ptr_tfd = (port + 0x20) as *const u32;
    let mut timeout = 100_000;
    while (core::ptr::read_volatile(ptr_tfd) & (0x80 | 0x08)) != 0 && timeout > 0 {
        timeout -= 1;
    }
    if timeout == 0 {
        return Err("Timeout Port AHCI occupé");
    }

    let ptr_ci = (port + 0x38) as *mut u32;
    core::ptr::write_volatile(ptr_ci, 1);

    // Boucle d'attente avec timeout d'échappement anti-gel
    timeout = 1_000_000;
    loop {
        let ci = core::ptr::read_volatile(ptr_ci);
        if (ci & 1) == 0 {
            break;
        }
        let is = core::ptr::read_volatile((port + 0x10) as *const u32);
        if (is & (1 << 30)) != 0 {
            return Err("Erreur matérielle retournée par le disque lors du transfert");
        }
        timeout -= 1;
        if timeout == 0 {
            return Err("Timeout AHCI : le disque n'a pas répondu à temps");
        }
        core::hint::spin_loop();
    }

    // Acquittement post-commande
    core::ptr::write_volatile((port + 0x10) as *mut u32, 0xFFFF_FFFF);
    Ok(())
}
