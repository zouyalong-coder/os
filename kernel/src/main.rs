#![no_std] // 不链接 Rust 标准库
#![no_main]
// 禁用所有 Rust 层级的入口点
// 自定义测试框架会生成 main 函数用于测试，但由于我们使用了 no_main，是入口变成了 _start
#![feature(custom_test_frameworks)] // no_std 禁用了默认的测试框架，需要自定义
#![test_runner(kernel::test_runner)] // 指定测试框架的入口
// 使得测试生成的 main 函数改为 test_main
#![reexport_test_harness_main = "test_main"] // 重新导出测试框架的入口

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::{memory::active_level_4_table, println};
use x86_64::{structures::paging::PageTable, VirtAddr};

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
    // boot_info
    let l4_table = active_level_4_table(boot_info.physical_memory_offset);
    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 Entry[{}]: {:?}", i, entry);
            let l3_phy = entry.frame().unwrap().start_address();
            let l3_vir = VirtAddr::new(boot_info.physical_memory_offset + l3_phy.as_u64());
            let ptr = l3_vir.as_mut_ptr();
            let l3_table: &PageTable = unsafe { &*ptr };

            // print non-empty entries of the level 3 table
            for (i, entry) in l3_table.iter().enumerate() {
                if !entry.is_unused() {
                    println!("  L3 Entry {}: {:?}", i, entry);
                }
            }
        }
    }
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
