// QBX System - Révision 0.3
// Fichier : src/allocator.rs
// Description : Gestionnaire d'allocation dynamique extensible avec surveillance automatique du Heap

use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
use spin::Mutex;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE_INITIAL: usize = 2 * 1024 * 1024; // 2 Mio
pub const HEAP_SIZE: usize = HEAP_SIZE_INITIAL;

// Seuils pour l'extension automatique
pub const HEAP_SEUIL_CRITIQUE: usize = 256 * 1024;       // 256 Ko restants
pub const HEAP_PAS_EXTENSION: usize = 1024 * 1024;       // Palier d'extension de 1 Mio
pub const HEAP_PLAFOND_MAX: usize = 16 * 1024 * 1024;    // Plafond absolu de sécurité à 16 Mio

pub static HEAP_TAILLE_ACTUELLE: Mutex<usize> = Mutex::new(HEAP_SIZE_INITIAL);

// --- [STATIC 1 : ALLOCATOR] ---
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

// --- [FONCTION 1 : init_heap] ---
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE_INITIAL - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.ignore();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE_INITIAL);
    }

    Ok(())
}

/// Retourne (octets_utilises, octets_libres, taille_totale)
pub fn obtenir_statistiques() -> (usize, usize, usize) {
    let heap = ALLOCATOR.lock();
    let total = *HEAP_TAILLE_ACTUELLE.lock();
    (heap.used(), heap.free(), total)
}

/// Alloue et mappe dynamiquement de nouvelles pages physiques consécutives au tas existant
pub fn etendre_heap(octets_supplementaires: usize) -> Result<usize, &'static str> {
    if octets_supplementaires == 0 {
        return Ok(*HEAP_TAILLE_ACTUELLE.lock());
    }

    let mut taille_heap = HEAP_TAILLE_ACTUELLE.lock();

    // Protection contre l'emballement mémoire au-delà du plafond autorisé
    if *taille_heap + octets_supplementaires > HEAP_PLAFOND_MAX {
        return Err("Plafond maximal du Heap atteint (16 Mio)");
    }

    // Alignement strict sur la frontière d'une page (4096 octets)
    let taille_alignee = (octets_supplementaires + 4095) & !4095;

    let mut lock_contexte = crate::memory::CONTEXTE_MEMOIRE.lock();
    let contexte = lock_contexte.as_mut().ok_or("Contexte de pagination non disponible")?;

    let debut_virtuel = VirtAddr::new((HEAP_START + *taille_heap) as u64);
    let fin_virtuelle = debut_virtuel + taille_alignee as u64 - 1u64;

    let start_page = Page::containing_address(debut_virtuel);
    let end_page = Page::containing_address(fin_virtuelle);
    let plage_pages = Page::range_inclusive(start_page, end_page);

    for page in plage_pages {
        let frame = contexte
            .frame_allocator
            .allocate_frame()
            .ok_or("Mémoire physique épuisée")?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            contexte
                .mapper
                .map_to(page, frame, flags, &mut contexte.frame_allocator)
                .map_err(|_| "Échec du mappage de page")?
                .flush();
        }
    }

    // Incorporation immédiate du bloc physique dans la liste de l'allocateur
    unsafe {
        ALLOCATOR.lock().extend(taille_alignee);
    }

    *taille_heap += taille_alignee;

    crate::klog!(
        "[ALC] Heap etendu de {} Ko (Nouvelle capacite : {} Ko)",
        taille_alignee / 1024,
        *taille_heap / 1024
    );

    Ok(*taille_heap)
}

/// Contrôle préventif de l'espace libre avec déclenchement automatique de l'extension
pub fn verifier_et_etendre() {
    let (_, libre, total) = obtenir_statistiques();

    if libre < HEAP_SEUIL_CRITIQUE && total < HEAP_PLAFOND_MAX {
        crate::klog!(
            "[ALC] Seuil critique franchi ({} Ko restants) : auto-extension declenchee",
            libre / 1024
        );
        let _ = etendre_heap(HEAP_PAS_EXTENSION);
    }
}
