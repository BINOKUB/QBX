// QBX Interrupts Module - Révision 0.6
// Fichier : src/interrupts.rs
// Description : Configuration de l'IDT, gestion des deux contrôleurs PIC et traitement direct du clavier

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use lazy_static::lazy_static;
use spin::Mutex;
use pic8259::ChainedPics;
use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// --- [STATIC 1 : PICS] ---
// Description : Contrôleurs d'interruptions programmables (PIC 8259 maître et esclave).
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// --- [STATIC 2 : KEYBOARD] ---
// Description : Moteur de décodage du clavier x86 réglé sur la disposition US104.
pub static KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
    Mutex::new(Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    ));

// --- [ENUMERATION 1 : InterruptIndex] ---
// Description : Index des lignes IRQ du matériel.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

// --- [LAZY STATIC 1 : IDT] ---
// Description : Table des descripteurs d'interruptions système.
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

// --- [FONCTION 1 : init_idt] ---
// Description : Charge la table IDT en mémoire processeur.
pub fn init_idt() {
    IDT.load();
}

// --- [FONCTION 2 : breakpoint_handler] ---
// Description : Routine d'exception pour les points d'arrêt (Breakpoint).
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// --- [FONCTION 3 : timer_interrupt_handler] ---
// Description : Routine de l'horloge système (Timer PIT).
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

// --- [FONCTION 4 : keyboard_interrupt_handler] ---
// Description : Lit le scancode sur le port 0x60, gère les caractères Unicode et intercepte les flèches directionnelles pour l'historique du Shell.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    use pc_keyboard::KeyCode;
    use crate::shell::SHELL;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                pc_keyboard::DecodedKey::Unicode(character) => {
                    SHELL.lock().introduire_caractere(character);
                }
                pc_keyboard::DecodedKey::RawKey(key_code) => match key_code {
                    KeyCode::ArrowUp => {
                        SHELL.lock().historique_precedent();
                    }
                    KeyCode::ArrowDown => {
                        SHELL.lock().historique_suivant();
                    }
                    _ => {}
                },
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
