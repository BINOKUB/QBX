// QBX Core - Révision 1.1
// Fichier : src/main.rs
// Description : Point d'entrée du noyau, initialisation mémoire, enregistrement
//               du contexte multitâche et boucle d'ordonnancement asynchrone

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
pub mod task; // Module d'ordonnancement coopératif et gestion des threads noyau

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use x86_64::VirtAddr;
use memory::BootInfoFrameAllocator;
use pc_keyboard::{Keyboard, ScancodeSet1, layouts, HandleControl};
use spin::Mutex;
use lazy_static::lazy_static;

// Tâche d'arrière-plan de démonstration pour le multitâche coopératif
fn tache_demo() {
    crate::klog!("[TSK] Tâche témoin démarrée (Étape 1/3)");
    task::ceder();
    crate::klog!("[TSK] Tâche témoin reprise (Étape 2/3)");
    task::ceder();
    crate::klog!("[TSK] Tâche témoin reprise (Étape 3/3)");
    task::ceder();
    crate::klog!("[TSK] Tâche témoin achevée");
}

// Définition de la fonction d'entrée appelée par le bootloader
entry_point!(kernel_main);

// --- [SECTION 1 : GESTIONNAIRE DE PANIQUE] ---
// Capture toute panique fatale du noyau, affiche l'erreur à l'écran et bloque le CPU.
#[panic_handler]
fn gestionnaire_panic(information: &PanicInfo) -> ! {
    println!("{}", information);
    loop {
        x86_64::instructions::hlt();
    }
}

// --- [SECTION 2 : GESTION DU CLAVIER] ---
// Décodeur d'événements clavier US-104 exécuté en dehors des interruptions matérielles.
lazy_static! {
    static ref KEYBOARD_DECODER: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore,
        ));
}

// Resynchronise l'état de la touche Caps Lock pour éviter les inversions de casse sous QEMU/GTK.
fn synchroniser_caps_lock() {
    let mut kb = KEYBOARD_DECODER.lock();
    // Émission du scancode Make (pression)
    if let Ok(Some(event)) = kb.add_byte(0x3A) {
        kb.process_keyevent(event);
    }
    // Émission du scancode Break (relâchement)
    if let Ok(Some(event)) = kb.add_byte(0xBA) {
        kb.process_keyevent(event);
    }
}

// --- [SECTION 3 : INITIALISATION PRINCIPALE DU NOYAU] ---
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Bannière de démarrage
    println!("  ____  ____  __  __");
    println!(" / __ \\|  _ \\ \\ \\/ /");
    println!("| |  | | |_) ) >  < ");
    println!("| |__| |  _ < / /\\ \\");
    println!(" \\___\\_\\____//_/  \\_\\");
    println!("=== QBX - EXP (Québec UNIX) v0.1 ===");
    println!("Initialisation du système...\n");

    // 1. Initialisation de la table des descripteurs d'interruption (IDT) et des PIC 8259
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    // 2. Initialisation de la mémoire paginée et de l'allocateur de cadres physiques
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // 3. Montage du tas initial de 2 Mio
    if let Err(e) = allocator::init_heap(&mut mapper, &mut frame_allocator) {
        panic!("Échec de l'initialisation du Heap : {:?}", e);
    }

    // 4. Sauvegarde du contexte de pagination global (requis pour l'auto-expansion dynamique)
    memory::init_contexte(mapper, frame_allocator);
    println!("Heap 2 MiB : OK");

   // 5. Initialisation du sous-système multitâche (Enregistre le thread principal comme Tâche 0)
    task::initialiser();

    // Enregistrement de la tâche témoin dans la file coopérative
    task::creer_tache(tache_demo);

    // 6. Journalisation des événements d'amorçage
    crate::klog!("[KRN] QBX Microkernel v0.1 démarre");
    crate::klog!("[CPU] Initialisation IDT et PIC terminée, interruptions activées");
    crate::klog!("[MEM] Pagination physique et virtuelle initialisée");
    crate::klog!("[ALC] Heap de 2 MiB initialisé avec succès");
    crate::klog!("[VFS] Système de fichiers en mémoire monté sur '/'");
    crate::klog!("[SHL] Shell interactif initialisé");

    println!("Système prêt.\n");
    print!("qbx:{}> ", fs::chemin_actuel());

    // Resynchronisation initiale de l'état des touches
    synchroniser_caps_lock();

    // --- [SECTION 4 : BOUCLE D'ÉVÉNEMENTS ET ORDONNANCEMENT] ---
    loop {
        // Extraction protégée d'un scancode en attente dans la file clavier
        let scancode_opt = x86_64::instructions::interrupts::without_interrupts(|| {
            clavier_queue::SCANCODE_QUEUE.lock().pop()
        });

        if let Some(scancode) = scancode_opt {
            // Un événement clavier est présent : décodage et routage vers l'application active
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
            // Aucun événement clavier : on cède le processeur aux tâches d'arrière-plan
            task::ceder();

            // Place le processeur en veille basse consommation jusqu'à la prochaine interruption matérielle
            x86_64::instructions::hlt();
        }
    }
}
