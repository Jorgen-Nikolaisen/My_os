// ! Type-safe addresses and page constants

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PhysAddr(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VirtAddr(pub usize);

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;
pub const PTE_PER_TABLE: usize = 512; // 4 KiB / 8 bytes

pub const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

pub const fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align -1)
}

pub const fn vpn(level: usize, va: usize) -> usize {
    (va >> (12 + level * 9)) & 0x1FF
}