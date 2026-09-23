//! Sv39 page table setup.

use super::addr::{align_down, align_up, PhysAddr, VirtAddr, PAGE_SIZE, PTE_PER_TABLE, vpn};

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PageTableEntry(u64);

// Sv39 PTE flags
pub const PTE_V: u64 = 1 << 0;  // Valid
pub const PTE_R: u64 = 1 << 1;  // Read
pub const PTE_W: u64 = 1 << 2;  // Write
pub const PTE_X: u64 = 1 << 3;  // Execute
pub const PTE_U: u64 = 1 << 4;  // User
pub const PTE_G: u64 = 1 << 5;  // Global
pub const PTE_A: u64 = 1 << 6;  // Accessed
pub const PTE_D: u64 = 1 << 7;  // Dirty

impl PageTableEntry {
    pub const fn new(ppn: u64, flags: u64) -> Self {
        PageTableEntry((ppn << 10) | (flags & 0x3FF))
    }

    pub const fn ppn(self) -> u64 {
        self.0 >> 10
    }

    pub const fn is_valid(self) -> bool {
        (self.0 & PTE_V) != 0
    }
}

#[repr(C)]
#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; PTE_PER_TABLE],
}

impl PageTable {
    pub const fn empty() -> Self {
        PageTable {
            entries: [PageTableEntry(0); PTE_PER_TABLE],
        }
    }
}

// Root table lives in .bss — must be page-aligned.
static mut ROOT_TABLE: PageTable = PageTable::empty();

unsafe fn alloc_table_frame() -> *mut PageTable {
    let phys = super::pmm::frame_allocator()
        .alloc_frame()
        .expect("vmm: out of memory for page table");
    let ptr = phys as *mut PageTable;
    core::ptr::write_bytes(ptr as *mut u8, 0, PAGE_SIZE);
    ptr
}

/// Walk the page table, creating intermediate tables as needed.
/// Returns a pointer to the level-0 PTE that maps `vaddr`.
unsafe fn walk_create(vaddr: VirtAddr) -> *mut PageTableEntry {
    let mut table: *mut PageTable = &mut ROOT_TABLE;

    for level in [2, 1] {
        let idx = vpn(level, vaddr.0);
        let pte = &mut (*table).entries[idx];
        if !pte.is_valid() {
            let new_table = alloc_table_frame();
            *pte = PageTableEntry::new((new_table as u64) >> 12, PTE_V);
        }
        table = (pte.ppn() << 12) as *mut PageTable;
    }

    &mut (*table).entries[vpn(0, vaddr.0)]
}

pub unsafe fn map(vaddr: VirtAddr, paddr: PhysAddr, flags: u64) {
    assert!(vaddr.0 == align_down(vaddr.0, PAGE_SIZE));
    assert!(paddr.0 == align_down(paddr.0, PAGE_SIZE));
    let pte = walk_create(vaddr);
    *pte = PageTableEntry::new((paddr.0 as u64) >> 12, flags | PTE_V);
}

/// Identity map a contiguous range [start, end) using 4 KiB pages.
/// `flags` is a bitmask of PTE_R/W/X/U etc.
pub unsafe fn identity_map_range(start: usize, end: usize, flags: u64) {
    let mut addr = align_down(start, PAGE_SIZE);
    let end = align_up(end, PAGE_SIZE);
    while addr < end {
        map(VirtAddr(addr), PhysAddr(addr), flags);
        addr += PAGE_SIZE;
    }
}

/// Build identity map and enable Sv39 MMU.
pub unsafe fn init() {
    extern "C" {
        static _kernel_end: u8;
    }

    let kernel_end = &_kernel_end as *const u8 as usize;
    let ram_end: usize = 0x8000_0000 + 128 * 1024 * 1024;

    // Kernel RAM
    identity_map_range(0x8000_0000, ram_end, PTE_R | PTE_W | PTE_X);

    // UART
    identity_map_range(0x1000_0000, 0x1000_1000, PTE_R | PTE_W);

    // SiFive test finisher (QEMU exit device)
    identity_map_range(0x100000, 0x101000, PTE_R | PTE_W);

    // CLINT (timer registers)
    identity_map_range(0x2000000, 0x2100000, PTE_R | PTE_W);

    // satp = mode(8) << 60 | root_ppn
    let root_phys = &ROOT_TABLE as *const _ as u64;
    let satp: u64 = (8u64 << 60) | (root_phys >> 12);

    core::arch::asm!("csrw satp, {}", in(reg) satp);
    core::arch::asm!("sfence.vma zero, zero");
}