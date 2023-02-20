use core::alloc::GlobalAlloc;

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
static ALLOCATOR: Dummy = Dummy;

/// #[alloc_error_handler] 用于处理 alloc crate 的分配失败，当使用 extern crate alloc 时，必须由用户提供一个 alloc_error_handler。参数 layout 是传入 alloc 的 layout。
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
