// QBX System - Révision 0.2
// Fichier : src/memory.rs
// Description : Gestionnaire de pagination, allocateur de frames physiques et conversion d'adresses DMA

use x86_64::{
    structures::paging::{
        FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB, Translate,
    },
    PhysAddr, VirtAddr,
};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

// --- [SECTION 1 : INITIALISATION DE LA PAGINATION] ---

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

// --- [SECTION 2 : ALLOCATEUR DE FRAMES DU BOOTLOADER] ---

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

// --- [SECTION 3 : CONTEXTE MÉMOIRE GLOBAL] ---

pub struct ContexteMemoire {
    pub mapper: OffsetPageTable<'static>,
    pub frame_allocator: BootInfoFrameAllocator,
}

pub static CONTEXTE_MEMOIRE: Mutex<Option<ContexteMemoire>> = Mutex::new(None);

pub fn init_contexte(mapper: OffsetPageTable<'static>, frame_allocator: BootInfoFrameAllocator) {
    *CONTEXTE_MEMOIRE.lock() = Some(ContexteMemoire {
        mapper,
        frame_allocator,
    });
}

// --- [SECTION 4 : CONVERSIONS PHYSIQUE / VIRTUELLE POUR LE DMA ET LES PILOTES] ---

static OFFSET_MEMOIRE_PHYSIQUE: AtomicU64 = AtomicU64::new(0);

pub fn enregistrer_offset_physique(offset: u64) {
    OFFSET_MEMOIRE_PHYSIQUE.store(offset, Ordering::SeqCst);
}

/// Convertit une adresse physique matérielle en adresse virtuelle accessible par le noyau
pub fn physique_vers_virtuelle(adresse_physique: u64) -> u64 {
    adresse_physique + OFFSET_MEMOIRE_PHYSIQUE.load(Ordering::Relaxed)
}

/// Traduit une adresse virtuelle du noyau en adresse physique réelle pour les contrôleurs DMA
pub fn traduire_virtuelle_vers_physique(virt: u64) -> Option<u64> {
    let guard = CONTEXTE_MEMOIRE.lock();
    guard.as_ref().and_then(|ctx| {
        ctx.mapper.translate_addr(VirtAddr::new(virt)).map(|p| p.as_u64())
    })
}
