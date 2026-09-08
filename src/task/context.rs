// QBX Core - Révision 0.2
// Fichier : src/task/context.rs
// Description : Contexte d'exécution x86_64 (PCB) et routine de permutation de pile (cpu_switch)

use core::arch::global_asm;

// Registres préservés par l'appelé (callee-saved) selon la convention System V AMD64 ABI
#[repr(C)]
#[derive(Debug, Default)]
pub struct ContexteTache {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbx: usize,
    pub rbp: usize,
    pub rsp: usize,
}

// Routine assembleur directe de bascule matérielle inspirée de cpu_switch.S de FreeBSD
global_asm!(
    r#"
    .global basculer_contexte
    basculer_contexte:
        # rdi = pointeur vers l'ancien ContexteTache (*mut ContexteTache)
        # rsi = pointeur vers le nouveau ContexteTache (*const ContexteTache)

        # 1. Sauvegarde de l'état de la tâche sortante
        mov [rdi + 0x00], r15
        mov [rdi + 0x08], r14
        mov [rdi + 0x10], r13
        mov [rdi + 0x18], r12
        mov [rdi + 0x20], rbx
        mov [rdi + 0x28], rbp
        mov [rdi + 0x30], rsp

        # 2. Chargement de l'état de la tâche entrante
        mov r15, [rsi + 0x00]
        mov r14, [rsi + 0x08]
        mov r13, [rsi + 0x10]
        mov r12, [rsi + 0x18]
        mov rbx, [rsi + 0x20]
        mov rbp, [rsi + 0x28]
        mov rsp, [rsi + 0x30]

        # 3. Restitution du flux : dépile l'adresse de retour (RIP) de la nouvelle pile
        ret
    "#
);

extern "C" {
    pub fn basculer_contexte(ancien: *mut ContexteTache, nouveau: *const ContexteTache);
}
