use core::alloc::GlobalAlloc;

use super::{align_up, Locked};

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    /// 下一个可用的内存地址。
    next: usize,
    /// 记录发生过多少次分配。
    allocations: usize,
}

impl BumpAllocator {
    ///
    /// const fn，表示这个函数是一个常量函数。常量函数可以在编译时执行，而不是在运行时执行。常量函数的执行结果必须是常量，而且不能包含任何可变的局部变量。
    pub const fn new() -> Self {
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

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    // &self 的原因：GlobalAllocator 是通过 #[global_allocator] 属性指定的，而这个属性只能是 static 的，而 static 是不可变的（可变的话就无法共享了）。因此，我们只能使用 &self。
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut bump = self.lock();
        let alloc_start = align_up(bump.next, layout.align());
        // 检查是否超出 usize 的范围（溢出）。
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return core::ptr::null_mut(),
        };
        if alloc_end > bump.heap_end {
            // 堆空间不足。
            return core::ptr::null_mut();
        }
        bump.next = alloc_end;
        bump.allocations += 1;
        alloc_start as *mut u8
    }

    /// 这个版本的 dealloc 不会释放内存，而是只是记录分配次数。所以这个allocator的内存是不能复用的（除非）
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        let mut bump = self.lock();
        bump.allocations -= 1;
        if bump.allocations == 0 {
            // 如果没有分配，那么就把 next 重置为 heap_start。
            bump.next = bump.heap_start;
        }
    }
}
