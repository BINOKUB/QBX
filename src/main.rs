// QBX Core - Révision 1.2
// Fichier : src/main.rs
// Description : Point d'entrée du noyau, initialisation mémoire, ordonnanceur
//               préemptif et boucle de gestion asynchrone

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

// --- [MODULES INTERNES DU NOYAU] ---
mod vga_buffer;
mod interrupts;
mod shell;
mod power;
mod commandes;
mod clavier_queue;
pub mod fs;
pub mod allocator;
pub mod memory;
pub mod journal;
pub mod task; // Module d'ordonnancement préemptif et threads noyau

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use x86_64::VirtAddr;
use memory::BootInfoFrameAllocator;
use pc_keyboard::{Keyboard, ScancodeSet1, layouts, HandleControl};
use spin::Mutex;
use lazy_static::lazy_static;

// Définition du point d'entrée appelé par le chargeur d'amorçage
entry_point!(kernel_main);

// --- [SECTION 1 : GESTIONNAIRE DE PANIQUE] ---
#[panic_handler]
fn gestionnaire_panic(information: &PanicInfo) -> ! {
    println!("{}", information);
    loop {
        x86_64::instructions::hlt();
    }
}

// --- [SECTION 2 : DÉCODEUR CLAVIER] ---
lazy_static! {
    static ref KEYBOARD_DECODER: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore,
        ));
}

fn synchroniser_caps_lock() {
    let mut kb = KEYBOARD_DECODER.lock();
    if let Ok(Some(event)) = kb.add_byte(0x3A) {
        kb.process_keyevent(event);
    }
    if let Ok(Some(event)) = kb.add_byte(0xBA) {
        kb.process_keyevent(event);
    }
}

// --- [SECTION 3 : TÂCHE SENTINELLE D'ÉPREUVE PRÉEMPTIVE] ---
// Boucle infinie stricte sans aucun appel coopératif à ceder()
fn tache_epreuve_preemption() {
    let mut compteur: u64 = 0;
    loop {
        compteur = compteur.wrapping_add(1);
        // Toutes les ~150 millions d'itérations, écrit une trace dans le journal
        if compteur % 150_000_000 == 0 {
            crate::klog!("[TEST] Sentinelle en rotation preemptive continue");
        }
    }
}

// --- [SECTION 4 : INITIALISATION DU NOYAU] ---
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("  ____  ____  __  __");
    println!(" / __ \\|  _ \\ \\ \\/ /");
    println!("| |  | | |_) ) >  < ");
    println!("| |__| |  _ < / /\\ \\");
    println!(" \\___\\_\\____//_/  \\_\\");
    println!("=== QBX - EXP (Québec UNIX) v0.1 ===");
    println!("Initialisation du système...\n");

    // 1. Initialisation IDT, contrôleur PIC et activation des interruptions
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    // 2. Initialisation de la mémoire paginée et de l'allocateur de cadres
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // 3. Montage du tas initial de 2 Mio
    if let Err(e) = allocator::init_heap(&mut mapper, &mut frame_allocator) {
        panic!("Échec de l'initialisation du Heap : {:?}", e);
    }

    // 4. Sauvegarde du contexte de pagination pour l'auto-expansion dynamique
    memory::init_contexte(mapper, frame_allocator);
    println!("Heap 2 MiB : OK");

    // 5. Initialisation du sous-système multitâche (Tâche 0 = noyau / shell)
    task::initialiser();

    // 6. Démarrage de la tâche sentinelle en arrière-plan
    task::creer_tache("sentinelle", tache_epreuve_preemption);

    // 7. Journalisation des étapes d'initialisation
    crate::klog!("[KRN] QBX Microkernel v0.1 démarre");
    crate::klog!("[CPU] Initialisation IDT et PIC terminée, interruptions activées");
    crate::klog!("[MEM] Pagination physique et virtuelle initialisée");
    crate::klog!("[ALC] Heap de 2 MiB initialisé avec succès");
    crate::klog!("[VFS] Système de fichiers en mémoire monté sur '/'");
    crate::klog!("[SHL] Shell interactif initialisé");

    println!("Système prêt.\n");
    print!("qbx:{}> ", fs::chemin_actuel());

    synchroniser_caps_lock();

    // --- [SECTION 5 : BOUCLE D'ÉVÉNEMENTS DU SHELL] ---
    loop {
        let scancode_opt = x86_64::instructions::interrupts::without_interrupts(|| {
            clavier_queue::SCANCODE_QUEUE.lock().pop()
        });

        if let Some(scancode) = scancode_opt {
            let mut kb = KEYBOARD_DECODER.lock();
            if let Ok(Some(key_event)) = kb.add_byte(scancode) {
                if let Some(key) = kb.process_keyevent(key_event) {
                    let editeur_actif = commandes::edt::EDITEUR.lock().est_actif();

                    if editeur_actif {
                        commandes::edt::EDITEUR.lock().traiter_touche(key);
                    } else {
                        match key {
                            pc_keyboard::DecodedKey::Unicode(character) => {
                                shell::SHELL.lock().introduire_caractere(character);
                            }
                            pc_keyboard::DecodedKey::RawKey(key_code) => match key_code {
                                pc_keyboard::KeyCode::ArrowUp => shell::SHELL.lock().historique_precedent(),
                                pc_keyboard::KeyCode::ArrowDown => shell::SHELL.lock().historique_suivant(),
                                _ => {}
                            }
                        }
                    }
                }
            }
        } else {
            // Cession volontaire si rien n'est tapé, avant l'attente basse consommation
            task::ceder();
            x86_64::instructions::hlt();
        }
    }
}
