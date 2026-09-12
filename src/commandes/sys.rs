// QBX Sys Command
// Fichier : src/commandes/sys.rs
// Description : Permet de tester les appels système (syscalls) depuis la console

use crate::syscalls::executer_syscall;
use crate::println;
use alloc::vec::Vec;

pub fn executer(args: &str) {
    let parties: Vec<&str> = args.split_whitespace().collect();
    if parties.is_empty() {
        println!("Usage : sys <id> [argument]");
        println!("Exemple : sys 4 mon_fichier.sh");
        return;
    }

    let id = match parties[0].parse::<usize>() {
        Ok(n) => n,
        Err(_) => {
            println!("sys: identifiant invalide");
            return;
        }
    };

    // Récupération de l'argument textuel (s'il existe)
    let arg_str = parties.get(1).copied().unwrap_or("");
    let arg1 = arg_str.as_ptr() as usize;
    let arg2 = arg_str.len();

    // Appel du répartiteur avec les pointeurs
    let code_retour = executer_syscall(id, arg1, arg2);
    println!("[SYSCALL] Appel {} exécuté (code retour : {})", id, code_retour);
}
