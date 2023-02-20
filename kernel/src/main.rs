#![no_std] // 不链接 Rust 标准库
#![no_main]
// 禁用所有 Rust 层级的入口点
// 自定义测试框架会生成 main 函数用于测试，但由于我们使用了 no_main，是入口变成了 _start
#![feature(custom_test_frameworks)] // no_std 禁用了默认的测试框架，需要自定义
#![test_runner(kernel::test_runner)] // 指定测试框架的入口
// 使得测试生成的 main 函数改为 test_main
#![reexport_test_harness_main = "test_main"] // 重新导出测试框架的入口

extern crate alloc;

use alloc::boxed::Box;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::{memory, println};
use x86_64::{
    structures::paging::{Page, PageTable, Translate},
    VirtAddr,
};

#[cfg(not(test))]
entry_point!(kernel_entry); // 指定入口点, 替换 no_mangle extern "C" fn _start(boot_info: &'static BootInfo) -> !
#[cfg(test)]
entry_point!(test_entry);

#[allow(unused)]
fn kernel_entry(boot_info: &'static BootInfo) -> ! {
    kernel::init();
    // x86_64::instructions::interrupts::int3(); // new

    // fn stack_overflow() {
    // stack_overflow(); // 每一次递归都会将返回地址入栈
    // }

    // 触发 stack overflow
    // stack_overflow();
    // let ptr = 0xdeadbeaf as *mut u32;
    // unsafe {
    //     // 触发 page fault
    //     *ptr = 2;
    // }
    use x86_64::registers::control::Cr3;
    let (level_4_table, flags) = Cr3::read(); // cr3 寄存器中放的是当前4级（顶级）页表的物理地址
                                              // 输出可以看到 start_address 是 0x1000
    println!("Level 4 page table at: {:?}", level_4_table.start_address());
    println!("flags: {:?}", flags);
    let mut mapper = memory::init(boot_info.physical_memory_offset);
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    // 虚拟地址 0 是 bootloader 肯定不会映射的地址，它会用来判断空指针。所以这里使用这个虚拟地址来测试。
    // 因为我们使用的是 EmptyFrameAllocator，不会真的分配物理帧，而 bootloader 已经映射了一个 1MB 的空间，所以复用 0 不会产生分配。
    // let page = Page::containing_address(VirtAddr::new(0));
    // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // unsafe {
    //     page_ptr
    //         .offset(400) // vga 的400 偏移处
    //         .write_volatile(0x_f021_f077_f065_f04e) // “New!”
    // };
    // 测试需要分配新页的情况, 此时会 panic，因为 EmptyFrameAllocator 不能分配新页
    // 此地址的 L1 页表项不存在，所以会触发 page fault
    // let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));

    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // unsafe {
    //     page_ptr
    //         .offset(400) // vga 的400 偏移处
    //         .write_volatile(0x_f021_f077_f065_f04e) // “New!”
    // };
    let x = Box::new(1);
    println!("here");
    kernel::hlt_loop()
}

#[cfg(test)]
#[allow(unused)]
fn test_entry(boot_info: &'static BootInfo) -> ! {
    test_main();
    kernel::hlt_loop()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    #[cfg(test)]
    kernel::test_panic_handler(info);
    #[cfg(not(test))]
    {
        println!("panic: {}", info);
        kernel::hlt_loop()
    }
}

#[test_case]
fn trivial_assertion() {
    // serial_println!("trivial assertion... ");
    assert_eq!(1, 1);
    // serial_println!("[ok]");
}

#[cfg(test)]
mod tests {

    #[test_case]
    fn it_works() {
        // serial_println!("it works... ");
        assert_eq!(2 + 2, 4);
    }
}
