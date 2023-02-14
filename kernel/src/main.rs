#![no_std] // 不链接 Rust 标准库
#![no_main]
// 禁用所有 Rust 层级的入口点
// 自定义测试框架会生成 main 函数用于测试，但由于我们使用了 no_main，是入口变成了 _start
#![feature(custom_test_frameworks)] // no_std 禁用了默认的测试框架，需要自定义
#![test_runner(kernel::test_runner)] // 指定测试框架的入口
// 使得测试生成的 main 函数改为 test_main
#![reexport_test_harness_main = "test_main"] // 重新导出测试框架的入口

use core::panic::PanicInfo;
use kernel::{print, println};

#[no_mangle] // 不重整函数名
pub extern "C" fn _start() -> ! {
    // 因为编译器会寻找一个名为 `_start` 的函数，所以这个函数就是入口点
    // 默认命名为 `_start`
    #[cfg(test)]
    test_start();
    #[cfg(not(test))]
    run_start();

    loop {}
}

#[allow(unused)]
#[cfg(test)]
fn test_start() {
    test_main();
}

#[allow(unused)]
fn run_start() {
    kernel::init();
    // x86_64::instructions::interrupts::int3(); // new

    // fn stack_overflow() {
    // stack_overflow(); // 每一次递归都会将返回地址入栈
    // }

    // 触发 stack overflow
    // stack_overflow();
    loop {
        print!(">");
    }

    println!("here");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    #[cfg(test)]
    kernel::test_panic_handler(info);
    #[cfg(not(test))]
    {
        println!("panic: {}", info);
        loop {}
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
