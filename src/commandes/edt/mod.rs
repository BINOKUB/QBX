// QBX EDT Module - Révision 0.7
// Fichier : src/commandes/edt/mod.rs
// Description : Exportation des sous-modules et analyse des arguments de la commande 'edt'

pub mod affichage;
pub mod moteur;

pub use moteur::EDITEUR;

// --- [FONCTION 1 : executer] ---
// Description : Analyse la ligne d'arguments passée à la commande 'edt' et lance le moteur.
pub fn executer(args: &str) {
    let mut option_l = false;
    let mut nom_fichier = "";

    let mut parties = args.split_whitespace();
    while let Some(arg) = parties.next() {
        if arg == "-l" {
            option_l = true;
        } else if nom_fichier.is_empty() {
            nom_fichier = arg;
        }
    }

    // Si aucun nom de fichier n'est fourni, on passe une chaîne vide 
    // pour ouvrir un buffer anonyme et volatile (pas de création de fichier fantôme)[cite: 6].
    EDITEUR.lock().lancer(option_l, nom_fichier);
}
