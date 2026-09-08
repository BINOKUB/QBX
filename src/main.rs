// QBX Core - Révision 0.9
// Fichier : src/main.rs
// Description : Initialisation directe du Heap, Boucle d'événements asynchrone et synchronisation Caps Lock

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod vga_buffer;
mod interrupts;
mod shell;
mod power;
mod commandes;
mod clavier_queue;
pub mod fs;
pub mod allocator;
pub mod memory;
// mod ops;

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use x86_64::VirtAddr;
use memory::BootInfoFrameAllocator;
use pc_keyboard::{Keyboard, ScancodeSet1, layouts, HandleControl};
use spin::Mutex;
use lazy_static::lazy_static;

entry_point!(kernel_main);

// --- [FONCTION 1 : gestionnaire_panic] ---
// Description : Gère les paniques du noyau.
#[panic_handler]
fn gestionnaire_panic(information: &PanicInfo) -> ! {
    println!("{}", information);
    loop {}
}

// Le décodeur clavier est déplacé ici, hors de l'interruption !
lazy_static! {
    static ref KEYBOARD_DECODER: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore,
        ));
}

// --- [FONCTION 2 : synchroniser_caps_lock] ---
// Description : Simule un appui sur Caps Lock pour corriger le bug d'inversion QEMU/GTK.
fn synchroniser_caps_lock() {
    let mut kb = KEYBOARD_DECODER.lock();
    // Envoi du scancode Make (Appui) pour Caps Lock (0x3A)
    if let Ok(Some(event)) = kb.add_byte(0x3A) {
        kb.process_keyevent(event);
    }
    // Envoi du scancode Break (Relâchement) pour Caps Lock (0xBA)
    if let Ok(Some(event)) = kb.add_byte(0xBA) {
        kb.process_keyevent(event);
    }
}

// --- [FONCTION 3 : kernel_main] ---
// Description : Point d'entrée principal du noyau.
fn kernel_main(boot_info: &'static BootInfo) -> ! {
            println!("  ____  ____  __  __");
            println!(" / __ \\|  _ \\ \\ \\/ /");
            println!("| |  | | |_) ) >  < ");
            println!("| |__| |  _ < / /\\ \\");
            println!(" \\___\\_\\____//_/  \\_\\");
            println!("=== QBX - EXP (Québec UNIX) v0.1 ===");
            println!("Initialisation du système...\n");
           

    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    if let Err(e) = allocator::init_heap(&mut mapper, &mut frame_allocator) {
        panic!("Échec de l'initialisation du Heap : {:?}", e);
    }

    println!("Heap 2 MiB : OK");
    println!("Système prêt.\n");
    print!("qbx:{}> ", fs::chemin_actuel()); // <-- Le prompt corrigé est bien placé ici

    // --- CORRECTION : Resynchronisation du clavier avant la boucle ---
    synchroniser_caps_lock();

    // --- BOUCLE D'ÉVÉNEMENTS ASYNCHRONE ---
    loop {
        // Extraction du scancode stocké par l'interruption
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
            // S'il n'y a pas de touche à traiter, on met le CPU en veille jusqu'à la prochaine interruption
            x86_64::instructions::hlt();
        }
    }
}
