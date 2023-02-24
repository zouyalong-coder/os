//! 固定尺寸分配器：将 linked_list(或者其它) 分配器作为兜底分配器，提前分配部分固定大小的块，放在对应
//! 的链表中。分配时：先从对应的链表中分配，如果没有，则从兜底分配器中分配。
//! 优点：
//! 1. 缓存思想：优先从固定大小的链表中分配，速度快。alloc时，所有块的大小都是已知、固定的，只需要操作头部即可，不需要遍历链表；dealloc时，不需要保证链表顺序，所以只需要换到头部即可。
//! 2. 适用于大部分分配的大小都是固定的场景。
//! 缺点：
//! 1. 会有一定的内碎片。
//! 本实现只是固定块分配器的一种实现，其变体有：slab allocator、bitmap allocator、buddy system 等。
//! 当前实现还有一些明显的问题可以优化：
//! 1. 固定块大小的选择：目前是写死的，可以根据实际情况进行调整。
//! 2. 目前使用的是 lazy 方式分配，可以改为在初始化时就分配好一部分，加快早期分配的速度。
//! 3. 分配出来的块没有回收，可能导致碎片或者内存耗尽。

use core::{
    alloc::{GlobalAlloc, Layout},
    ptr,
};

use super::Locked;

struct ListNode {
    next: Option<&'static mut ListNode>,
}

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

pub struct FixedSizeBlockAllocator {
    /// 每个固定块链表的头部。
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    /// 兜底分配器。超出固定块大小的分配，或者没有可用的固定块时，使用这个分配器。
    fallback_allocator: linked_list_allocator::Heap,
}

impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        Self {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    /// 遍历确定最小的能容纳 `layout` 的固定块大小。
    fn list_index(layout: Layout) -> Option<usize> {
        let size = layout.size().max(layout.align());
        BLOCK_SIZES.iter().position(|&s| s >= size)
    }

    fn do_alloc(&mut self, layout: Layout) -> Option<ptr::NonNull<u8>> {
        match Self::list_index(layout) {
            Some(index) => match self.list_heads[index].take() {
                Some(node) => {
                    self.list_heads[index] = node.next.take();
                    unsafe { Some(ptr::NonNull::new_unchecked(node as *mut _ as _)) }
                }
                None => {
                    // 没有可用的固定块，从兜底分配器中分配。
                    self.fallback_allocator.allocate_first_fit(layout).ok()
                }
            },
            None => self.fallback_allocator.allocate_first_fit(layout).ok(),
        }
    }

    fn do_dealloc(&mut self, addr: *mut u8, layout: Layout) {
        match Self::list_index(layout) {
            Some(index) => {
                let node = unsafe { &mut *(addr as *mut ListNode) };
                node.next = self.list_heads[index].take();
                self.list_heads[index] = Some(node);
            }
            None => unsafe {
                self.fallback_allocator
                    .deallocate(ptr::NonNull::new_unchecked(addr), layout)
            },
        }
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.lock()
            .do_alloc(layout)
            .map_or(ptr::null_mut(), |x| x.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.lock().do_dealloc(ptr, layout)
    }
}
