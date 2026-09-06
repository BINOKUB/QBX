use core::fmt;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Couleur {
    Noir = 0,
    Bleu = 1,
    Vert = 2,
    Cyan = 3,
    Rouge = 4,
    Magenta = 5,
    Marron = 6,
    GrisClair = 7,
    GrisFonce = 8,
    BleuClair = 9,
    VertClair = 10,
    CyanClair = 11,
    RougeClair = 12,
    Rose = 13,
    Jaune = 14,
    Blanc = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct CodeCouleur(u8);

impl CodeCouleur {
    fn nouveau(texte: Couleur, fond: Couleur) -> CodeCouleur {
        CodeCouleur((fond as u8) << 4 | (texte as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct CaractereEcran {
    caractere_ascii: u8,
    code_couleur: CodeCouleur,
}

const HAUTEUR_BUFFER: usize = 25;
const LARGEUR_BUFFER: usize = 80;

#[repr(transparent)]
struct Buffer {
    caracteres: [[Volatile<CaractereEcran>; LARGEUR_BUFFER]; HAUTEUR_BUFFER],
}

pub struct Ecrivain {
    colonne_position: usize,
    code_couleur: CodeCouleur,
    buffer: &'static mut Buffer,
}

impl Ecrivain {
    // Convertit un char Unicode (UTF-8) vers la table de caractères CP437 du mode texte VGA
    fn char_vers_cp437(c: char) -> u8 {
        match c {
            'é' => 0x82,
            'è' => 0x8a,
            'ê' => 0x88,
            'ë' => 0x89,
            'à' => 0x85,
            'â' => 0x83,
            'ä' => 0x84,
            'î' => 0x8c,
            'ï' => 0x8b,
            'ô' => 0x93,
            'ö' => 0x94,
            'ù' => 0x97,
            'û' => 0x96,
            'ü' => 0x81,
            'ç' => 0x87,
            'É' => 0x90,
            'À' => 0x80,
            'Ç' => 0x80,
            c if (c as u32) <= 0x7F => c as u8, // Caractères ASCII standard (0-127)
            _ => 0xfe, // Caractère non supporté (carré plein)
        }
    }

    pub fn ecrire_octet(&mut self, octet: u8) {
        match octet {
            b'\n' => self.nouvelle_ligne(),
            octet => {
                if self.colonne_position >= LARGEUR_BUFFER {
                    self.nouvelle_ligne();
                }

                let ligne = HAUTEUR_BUFFER - 1;
                let colonne = self.colonne_position;

                let code_couleur = self.code_couleur;
                self.buffer.caracteres[ligne][colonne].write(CaractereEcran {
                    caractere_ascii: octet,
                    code_couleur,
                });
                self.colonne_position += 1;
            }
        }
    }

    pub fn ecrire_chaine(&mut self, s: &str) {
        for c in s.chars() {
            match c {
                '\n' => self.nouvelle_ligne(),
                caractere => {
                    let octet_cp437 = Self::char_vers_cp437(caractere);
                    
                    if self.colonne_position >= LARGEUR_BUFFER {
                        self.nouvelle_ligne();
                    }

                    let ligne = HAUTEUR_BUFFER - 1;
                    let colonne = self.colonne_position;
                    let code_couleur = self.code_couleur;

                    self.buffer.caracteres[ligne][colonne].write(CaractereEcran {
                        caractere_ascii: octet_cp437,
                        code_couleur,
                    });
                    self.colonne_position += 1;
                }
            }
        }
    }

    fn nouvelle_ligne(&mut self) {
        for ligne in 1..HAUTEUR_BUFFER {
            for colonne in 0..LARGEUR_BUFFER {
                let caractere = self.buffer.caracteres[ligne][colonne].read();
                self.buffer.caracteres[ligne - 1][colonne].write(caractere);
            }
        }
        self.vider_ligne(HAUTEUR_BUFFER - 1);
        self.colonne_position = 0;
    }

    fn vider_ligne(&mut self, ligne: usize) {
        let vide = CaractereEcran {
            caractere_ascii: b' ',
            code_couleur: self.code_couleur,
        };
        for colonne in 0..LARGEUR_BUFFER {
            self.buffer.caracteres[ligne][colonne].write(vide);
        }
    }
}

impl fmt::Write for Ecrivain {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.ecrire_chaine(s);
        Ok(())
    }
}

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref ECRIVAIN: Mutex<Ecrivain> = Mutex::new(Ecrivain {
        colonne_position: 0,
        code_couleur: CodeCouleur::nouveau(Couleur::VertClair, Couleur::Noir),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    ECRIVAIN.lock().write_fmt(args).unwrap();
}
