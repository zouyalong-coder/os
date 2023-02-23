#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![cfg(allocator = "linked_list")]
extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::allocator::{self, HEAP_SIZE};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use kernel::allocator;
    use kernel::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    kernel::init();
    let phys_mem_offset = boot_info.physical_memory_offset;
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

/// build a large vector, to test both large allocations and multiple allocations (due to reallocations)
/// We verify the sum by comparing it with the formula for the n-th partial sum. This gives us some confidence that the allocated values are all correct.
#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

/// we create ten thousand allocations after each other. This test ensures that the allocator reuses freed memory for subsequent allocations since it would run out of memory otherwise. This might seem like an obvious requirement for an allocator, but there are allocator designs that don’t do this. An example is the bump allocator design that will be explained in the next post.
#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[cfg(allocator = "linked_list")]
#[test_case]
fn memory_reuse_for_linked_list_allocator() {
    // 第一个堆分配肯定发生在堆起始处。
    let x = Box::new(41);
    let x_ptr = &*x as *const i32;
    assert_eq!(x_ptr as usize, allocator::HEAP_START);
    let y = Box::new([0u64; 100]);
    let y_offset = &*y as *const [u64; 100] as usize - allocator::HEAP_START;
    drop(y);
    let y = Box::new([0u64; 1000]);
    assert_eq!(
        y_offset,
        &*y as *const [u64; 1000] as usize - allocator::HEAP_START
    );
    drop(y);
    drop(x);

    let x = Box::new([0u64; 1000]);
    let x_ptr = &*x as *const _;
    assert_eq!(x_ptr as usize, allocator::HEAP_START);
}
