// QBX Interrupts Module - Révision 0.8
// Fichier : src/interrupts.rs
// Description : Gestionnaire d'interruption (Exceptions x86, horloge, clavier et Page Fault #PF)

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::registers::control::Cr2;
use crate::{println, clavier_queue, klog};
use lazy_static::lazy_static;
use spin::Mutex;
use pic8259::ChainedPics;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

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

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// --- [INTERRUPTION 14 : Page Fault #PF] ---
// Capture les accès aux adresses mémoire non mappées ou non autorisées
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let adresse_fautive = Cr2::read();

    println!("\n================ ERREUR SYSTEME QBX ================");
    println!("Type              : Défaut de page (Page Fault #PF)");
    println!("Adresse fautive   : {:?}", adresse_fautive);
    println!("Code d'erreur     : {:?}", error_code);
    println!("Pointeur d'ordre  : {:#?}", stack_frame.instruction_pointer);
    println!("====================================================");

    klog!("[ERR] Defaut de page intercepte a {:?}", adresse_fautive);

    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

// --- [FONCTION 4 : keyboard_interrupt_handler] ---
// Interception matérielle asynchrone (Pousse le scancode et clôture immédiatement l'IRQ)
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    clavier_queue::ajouter_scancode(scancode);

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
