// Physical memory manager: simple free-list frame allocator

pub const PAGE_SIZE: usize = 4096;

struct FreeNode {
    next: *mut FreeNode,
}

pub struct FrameAllocator {
    free_list: *mut FreeNode,
}

impl FrameAllocator {
    pub const fn new() -> Self {
        FrameAllocator { free_list: core::ptr::null_mut() }
    }

    pub unsafe fn init(&mut self, start: usize, end: usize) {
        let mut addr = align_up(start, PAGE_SIZE);
        while addr + PAGE_SIZE <= end {
            self.free_frame(addr);
            addr += PAGE_SIZE;
        }
    }
    
    pub fn alloc_frame(&mut self) -> Option<usize> {
        self.alloc_frames(1)
    }

    pub fn alloc_frames(&mut self, n: usize) -> Option<usize> {
        if n == 0 {
            return None;
        }
        unsafe {
            let mut node = self.free_list;
            if node.is_null() {
                return None;
            }
            for _ in 1..n {
                let next = (*node).next;
                if next.is_null() {
                    return None;
                }
                // descending list: next + PAGE_SIZE must equal current node
                if (next as usize).wrapping_add(PAGE_SIZE) != node as usize {
                    return None; // fragmented
                }
                node = next;
            }
            // `node` is now the base (lowest address) of the block.
            self.free_list = (*node).next; // unlink the block
            Some(node as usize)
        }
    }


    pub unsafe fn free_frame(&mut self, addr: usize) {
        debug_assert_eq!(addr % PAGE_SIZE, 0, "freed frame must be page aligned");
        let node = addr as *mut FreeNode;
        (*node).next = self.free_list;
        self.free_list = node;
    }

    pub fn free_count(&self) -> usize {
        let mut count = 0;
        let mut node = self.free_list;
        unsafe {
            while !node.is_null() {
                count += 1;
                node = (*node).next;
            }
        }
        count
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align -1) & !(align - 1)
}

static mut FRAME_ALLOCATOR: FrameAllocator = FrameAllocator::new();

pub unsafe fn frame_allocator() -> &'static mut FrameAllocator {
    &mut *core::ptr::addr_of_mut!(FRAME_ALLOCATOR)
}
