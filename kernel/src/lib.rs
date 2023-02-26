// 使用 lib 分离，使得关于 no_std 部分的代码可以在不同的 crate 中复用，比如集成测试中的代码。
#![no_std]
#![cfg_attr(test, no_main)] // 在 test 模式下，采用 no_main
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)] // 指定测试框架
// 测试函数生成的 main 函数改为 test_main
#![reexport_test_harness_main = "test_main"] // 测试框架入口设置为 test_main
#![feature(abi_x86_interrupt)] // 开启 x86 中断
// 开启 alloc_error_handler
#![feature(alloc_error_handler)]
// mutable references are not allowed in constant functions
// any use of mutable references in const functions (including setting the next field to None) is still unstable.
#![feature(const_mut_refs)]
use core::panic::PanicInfo;

#[allow(unused)]
use bootloader::entry_point;

pub mod interrupts;

pub mod gdt;
pub mod memory;
pub mod qemu;
pub mod serial;
pub mod vga_buffer;
// alloc 是标准库的一部分，所以不应该在 Cargo.toml 中添加依赖
// 但是由于我们是在为一个自定义的目标进行编译，所以不能直接使用标准库中的alloc，所以需要使用 extern crate 语法。（以前所有的依赖都需要 extern crate，现在只在这种情况下需要。）
// extern crate 会使得 Rust 编译器重新编译 alloc。
extern crate alloc;
pub mod allocator;
pub mod task;

// #[cfg(test)]
// #[no_mangle]
// pub extern "C" fn _start() -> ! {
//     init();
//     test_main();
//     hlt_loop()
// }

// cargo test 的入口函数
#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    init();
    test_main();
    hlt_loop()
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    qemu::exit_qemu(qemu::QemuExitCode::Failed);
    hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        // 获取函数名
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    qemu::exit_qemu(qemu::QemuExitCode::Success);
}

/// 无限循环，等待中断。使用 hlt 指令，避免 CPU 100% 占用。
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// 初始化内核。
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe {
        // todo 了解PIC初始化过程
        interrupts::PICS.lock().initialize();
    }
    // 启动中断。
    x86_64::instructions::interrupts::enable();
}

// #[cfg(test)]
// mod tests {
#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}
// }
