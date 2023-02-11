//! 通常而言，当异常发生时，CPU会执行如下步骤：
//! 将一些寄存器数据入栈，包括指令指针以及 RFLAGS 寄存器。
//! 读取中断描述符表（IDT）的对应条目，比如当发生 page fault 异常时，调用14号条目。
//! 判断该条目确实存在，如果不存在，则触发 double fault 异常。
//! 如果该条目属于中断门（interrupt gate，bit 40 被设置为0），则禁用硬件中断。
//! 将 GDT 选择器载入代码段寄存器（CS segment）。
//! 跳转执行处理函数。
//!
//! 中断与普通函数的区别在于：
//! 普通函数由类似 call/ret 的指令调用，指令会自动完成返回地址入栈出栈的操作；而中断处理函数则需要自己进行处理。
//! 中断处理由于一般会发生上下文变化（比如SS、CPU flags 等），所以无法用一个简单指令完成，所以需要中断处理程序进行现场保护。
//!
//! 使用 x86 库封装了很多细节，要重新实现，可以参考：https://os.phil-opp.com/edition-1/extra/naked-exceptions/

lazy_static! {
    // 前32个是中断，后32个是异常.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

/// 初始化中断描述符表
pub fn init_idt() {
    IDT.load();
}

/// extern "x86-interrupt" 表示这是一个中断处理函数，而不是一个普通函数。
/// extern 表示遵守外部调用约定，这里表示遵守 x86 的中断调用约定。
/// breakpoint_handler 用于处理 int3 指令，即断点异常。
/// breakpoint exception 通常被用在调试器中：当程序员为程序打上断点，调试器会将对应的位置覆写为 int3 指令，CPU执行该指令后，就会抛出 breakpoint exception 异常。在调试完毕，需要程序继续运行时，调试器就会将原指令覆写回 int3 的位置。
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
