#![no_std]
#![no_main]

use core::arch::global_asm;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// global_asm!(include_str!("./asm/entry.asm"));
