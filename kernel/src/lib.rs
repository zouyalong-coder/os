// 使用 lib 分离，使得关于 no_std 部分的代码可以在不同的 crate 中复用，比如集成测试中的代码。
#![no_std]
#![cfg_attr(test, no_main)] // 在 test 模式下，采用 no_main
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)] // 指定测试框架
// 测试函数生成的 main 函数改为 test_main
#![reexport_test_harness_main = "test_main"] // 测试框架入口设置为 test_main

use core::panic::PanicInfo;

pub mod qemu;
pub mod serial;
pub mod vga_buffer;

// cargo test 的入口函数
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    qemu::exit_qemu(qemu::QemuExitCode::Failed);
    loop {}
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
