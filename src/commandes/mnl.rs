// QBX Centurion - Manuel Système Modulaire
// Fichier : src/commandes/mnl.rs
// Description : Aiguilleur générique chargeant chaque page via include_str! avec pagination automatique

use crate::println;
use crate::commandes::pager;

struct PageManuel {
    nom: &'static str,
    contenu: &'static str,
}

static PAGES: &[PageManuel] = &[
    PageManuel { nom: "afn",  contenu: include_str!("docs/afn.txt") },
    PageManuel { nom: "aide", contenu: include_str!("docs/aide.txt") },
    PageManuel { nom: "cat",  contenu: include_str!("docs/cat.txt") },
    PageManuel { nom: "cdr",  contenu: include_str!("docs/cdr.txt") },
    PageManuel { nom: "cpr",  contenu: include_str!("docs/cpr.txt") },
    PageManuel { nom: "ctr",  contenu: include_str!("docs/ctr.txt") },
    PageManuel { nom: "df",   contenu: include_str!("docs/df.txt") },
    PageManuel { nom: "dpc",  contenu: include_str!("docs/dpc.txt") },
    PageManuel { nom: "dsk",  contenu: include_str!("docs/dsk.txt") },
    PageManuel { nom: "echo", contenu: include_str!("docs/echo.txt") },
    PageManuel { nom: "edt",  contenu: include_str!("docs/edt.txt") },
    PageManuel { nom: "inf",  contenu: include_str!("docs/inf.txt") },
    PageManuel { nom: "ls",   contenu: include_str!("docs/ls.txt") },
    PageManuel { nom: "mmr",  contenu: include_str!("docs/mmr.txt") },
    PageManuel { nom: "mnl",  contenu: include_str!("docs/mnl.txt") },
    PageManuel { nom: "net",  contenu: include_str!("docs/net.txt") },
    PageManuel { nom: "ntr",  contenu: include_str!("docs/ntr.txt") },
    PageManuel { nom: "pager",contenu: include_str!("docs/pager.txt") },
    PageManuel { nom: "pci",  contenu: include_str!("docs/pci.txt") },
    PageManuel { nom: "probe",contenu: include_str!("docs/probe.txt") },
    PageManuel { nom: "qtr",  contenu: include_str!("docs/qtr.txt") },
    PageManuel { nom: "rnm",  contenu: include_str!("docs/rnm.txt") },
    PageManuel { nom: "snf",  contenu: include_str!("docs/snf.txt") },
    PageManuel { nom: "spp",  contenu: include_str!("docs/spp.txt") },
    PageManuel { nom: "su",   contenu: include_str!("docs/su.txt") },
    PageManuel { nom: "sync", contenu: include_str!("docs/sync.txt") },
    PageManuel { nom: "tmps", contenu: include_str!("docs/tmps.txt") },
    PageManuel { nom: "tpf",  contenu: include_str!("docs/tpf.txt") },
    PageManuel { nom: "tsk",  contenu: include_str!("docs/tsk.txt") },
    PageManuel { nom: "ver",  contenu: include_str!("docs/ver.txt") },
];

pub fn executer(commande: &str) {
    let cmd = commande.trim();

    if cmd.is_empty() {
        println!("Usage : mnl <commande>");
        println!("Pages de manuel disponibles :");
        for chunk in PAGES.chunks(6) {
            let noms: alloc::vec::Vec<&str> = chunk.iter().map(|p| p.nom).collect();
            println!("  {}", noms.join(", "));
        }
        return;
    }

    if let Some(page) = PAGES.iter().find(|p| p.nom == cmd) {
        pager::afficher_avec_pagination(page.contenu);
    } else {
        println!("Aucune page de manuel pour : {}", cmd);
        println!("Tapez 'mnl' sans argument pour voir la liste des commandes.");
    }
}
