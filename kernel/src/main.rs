#![no_std] // 不链接 Rust 标准库
#![no_main]
// 禁用所有 Rust 层级的入口点
// 自定义测试框架会生成 main 函数用于测试，但由于我们使用了 no_main，是入口变成了 _start
#![feature(custom_test_frameworks)] // no_std 禁用了默认的测试框架，需要自定义
#![test_runner(crate::test::test_runner)] // 指定测试框架的入口
// 使得测试生成的 main 函数改为 test_main
#![reexport_test_harness_main = "test_main"] // 重新导出测试框架的入口

mod qemu;
pub mod serial;
mod test;
mod vga_buffer;

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

#[no_mangle] // 不重整函数名
pub extern "C" fn _start() -> ! {
    // 因为编译器会寻找一个名为 `_start` 的函数，所以这个函数就是入口点
    // 默认命名为 `_start`

    // let vga_buffer = 0xb8000 as *mut u8;

    // for (i, &byte) in HELLO.iter().enumerate() {
    //     unsafe {
    //         *vga_buffer.offset(i as isize * 2) = byte;
    //         *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
    //     }
    // }
    use core::fmt::Write;
    vga_buffer::WRITER.lock().write_string("Hello again!\n");
    {
        writeln!(
            vga_buffer::WRITER.lock(),
            ", some numbers: {} {}",
            42,
            1.337
        );
    }

    #[cfg(test)]
    test_main();

    loop {}
}

/// 这个函数将在 panic 时被调用
#[panic_handler]
#[cfg(not(test))]
fn panic(info: &PanicInfo) -> ! {
    println!("panic: {}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    qemu::exit_qemu(qemu::QemuExitCode::Failed);
    loop {}
}

#[test_case]
fn trivial_assertion() {
    // serial_println!("trivial assertion... ");
    assert_eq!(1, 0);
    // serial_println!("[ok]");
}

#[cfg(test)]
mod tests {
    use super::serial_println;

    #[test_case]
    fn it_works() {
        // serial_println!("it works... ");
        assert_eq!(2 + 2, 4);
    }
}
