use core::alloc::GlobalAlloc;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    /// 下一个可用的内存地址。
    next: usize,
    /// 记录发生过多少次分配。
    allocations: usize,
}

impl BumpAllocator {
    pub fn new() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// 由于调用方需要保证分配的内存是有效的，因此这个函数是 unsafe 的。
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    // &self 的原因：GlobalAllocator 是通过 #[global_allocator] 属性指定的，而这个属性只能是 static 的，而 static 是不可变的（可变的话就无法共享了）。因此，我们只能使用 &self。
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let alloc_start = self.next;
        self.next = alloc_start + layout.size();
        self.allocations += 1;
        alloc_start as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        todo!()
    }
}
