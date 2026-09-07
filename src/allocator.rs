// QBX System - Révision 0.1
// Fichier : src/allocator.rs
// Description : Gestionnaire d'allocation mémoire dynamique (Heap) pour QBX

use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

// Plage mémoire virtuelle dédiée au Heap (agrandi à 2 Mio pour les grands fichiers et le mode réel)
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 2 * 1024 * 1024; // 2 Mio

// --- [STATIC 1 : ALLOCATOR] ---
// Description : Allocateur global du noyau pour Vec, String, Box, etc.
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

// --- [FONCTION 1 : init_heap] ---
// Description : Mappe les pages virtuelles du Heap vers des frames physiques et initialise l'allocateur.
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
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
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    Ok(())
}
