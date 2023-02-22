use core::alloc::{GlobalAlloc, Layout};

use super::{align_up, Locked};

/// A node in a linked list.
/// 实际管理的内存块在这个 header 的后面。
struct ListNode {
    /// 紧邻的内存块的大小
    size: usize,
    /// 下一个内存块的地址。由于是分配器，所以可以用 'static 生命周期。
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    pub const fn new(size: usize) -> Self {
        Self { size, next: None }
    }

    pub fn start_addr(&self) -> usize {
        self as *const _ as usize
    }

    pub fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    /// 指向链表的头部。即第一个空闲的内存块。
    head: ListNode,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0), // 头部并不是真正的内存块，所以大小为 0。
        }
    }

    ///
    /// # Safety
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        // 初始化时，只有一个内存块，且这个内存块的大小就是整个堆的大小。
        self.add_free_region(heap_start, heap_size)
    }

    /// 从头部插入空闲的内存块。
    /// # unsafe
    /// 由于调用方需要保证分配的内存是有效的，因此这个函数是 unsafe 的。
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // addr 必须是按 ListNode 对齐的。
        assert_eq!(addr, align_up(addr, core::mem::align_of::<ListNode>()));
        // 空闲内存必须至少能装下一个 ListNode header。
        assert!(size >= core::mem::size_of::<ListNode>());

        // 注意：这个是分配在栈上的。只有 Box::new 才会分配在堆上。
        let mut new_node = ListNode::new(size);
        new_node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        // 将 new_node 写入 node_ptr 指向的内存, 其实就是 copy
        node_ptr.write(new_node);
        self.head.next = Some(&mut *node_ptr);
    }

    /// 从头部开始查找第一个能够满足 size 和 align 的内存块。
    /// 返回的是一个 Option，如果找到了，就返回这个内存块的指针和内存块的起始地址。
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(region, size, align) {
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            }
            current = current.next.as_mut().unwrap();
        }
        None
    }

    // 使用 Result 的原因是可以方便地使用 ? 运算符。
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;
        if alloc_end > region.end_addr() {
            // 内存不够
            return Err(());
        }
        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < core::mem::size_of::<ListNode>() {
            // 剩余的内存不够放 ListNode header, 无法分割成两个内存块。
            return Err(());
        }
        Ok(alloc_start)
    }

    /// 计算对齐后的 size 和 align.
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(core::mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(core::mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();
        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                // 分割成两个内存块
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as _
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();
        allocator.add_free_region(ptr as usize, size);
    }
}
