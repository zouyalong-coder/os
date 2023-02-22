mod bump;

pub use bump::*;

use core::alloc::GlobalAlloc;

use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, page_table::PageTableEntry, FrameAllocator, Mapper, PageTableFlags,
        Size4KiB,
    },
    VirtAddr,
};

/// 测试接口。
pub struct Dummy;

// GlobalAlloc特性定义了一个堆分配器必须提供的功能。该trait很特别，因为程序员几乎从不直接使用它。相反，编译器会在使用alloc的分配和集合类型时自动插入对该trait方法的适当调用。
unsafe impl GlobalAlloc for Dummy {
    /// alloc 返回 null 表示分配失败，失败是需要调用alloc_error_handler
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        panic!("dealloc should be never called")
    }
}

/// #[global_allocator] 提供给 alloc crate 一个全局的堆分配器。当指定 extern crate alloc 时，必须由用户提供一个全局的堆分配器。
#[global_allocator]
// static ALLOCATOR: Dummy = Dummy;
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// #[alloc_error_handler] 用于处理 alloc crate 的分配失败，当使用 extern crate alloc 时，必须由用户提供一个 alloc_error_handler。参数 layout 是传入 alloc 的 layout。
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

/// 任意选择的堆起始地址（虚拟）和大小，只要不与内核代码重叠即可。
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        use x86_64::structures::paging::Page;

        // 返回包含指定虚拟地址的页。实际上就是做一个页对齐操作。
        let heap_start_page = Page::containing_address(VirtAddr::new(HEAP_START as u64));
        let heap_end_page =
            Page::containing_address(VirtAddr::new((HEAP_START + HEAP_SIZE - 1) as u64));
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)? //
                .flush(); // 刷新 TLB，使得新的映射生效。
        }
    }
    unsafe {
        ALLOCATOR
            .lock() // 可能并发分配，所以需要加锁。
            .init(HEAP_START, HEAP_SIZE); // 指定堆的起始地址和大小。堆是向上增长的。
    }
    Ok(())
}
