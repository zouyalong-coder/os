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
        unsafe {
        idt.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

use crate::{gdt, hlt_loop, println, task::keyboard::add_scan_code};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

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

/// double fault 就是当CPU执行错误处理函数失败时抛出的特殊异常。如果 double fault 异常没有被正确处理，CPU 会抛出 triple fault 异常，这是一个不能被处理的异常，会导致系统重启。
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT

{:#?}",
        stack_frame
    );
}

///                      ____________             _____
/// Timer ------------> |            |           |     |
/// Keyboard ---------> | Interrupt  |---------> | CPU |
/// Other Hardware ---> | Controller |           |_____|
/// Etc. -------------> |____________|
/// 可编程中断控制器：CPU 已定义的异常数量为 32 个，这里为了避开，从 32 开始定义中断。
///                      ____________                          ____________
/// Real Time Clock --> |            |   Timer -------------> |            |
/// ACPI -------------> |            |   Keyboard-----------> |            |      _____
/// Available --------> | Secondary  |----------------------> | Primary    |     |     |
/// Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |
/// Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|
/// Co-Processor -----> |            |   Parallel Port 2/3 -> |            |
/// Primary ATA ------> |            |   Floppy disk -------> |            |
/// Secondary ATA ----> |____________|   Parallel Port 1----> |____________|
/// PIC 有两个，分别是主片和从片，主片负责中断 0-7，从片负责中断 8-15。
/// 主控制器的端口地址为：指令 0x20，数据 0x21。
/// 从控制器的端口地址为：指令 0xA0，数据 0xA1。
/// Offset 为中断偏移量，即中断号。
pub const PIC_1_OFFSET: u8 = 32;
/// 从片的中断偏移量。
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// 这里设置的是主片和从片的 base 中断偏移量。
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET, // Timer 在 master 的第0个引脚，所以中断号为 32(0x20)
    Keyboard,             // Keyboard 在 master 的第1个引脚，所以中断号为 33(0x21)
    HardDisk = PIC_2_OFFSET + 6, // HardDisk 在 slave 的第6个引脚，所以中断号为 46(0x2E)
    SystemCall = 0x80,    // SystemCall 中断号为 0x80
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

/// Timer 中断处理函数。
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        // 要通知 PIC 中断已经处理完毕，否则后续中断会一直排队。
        // notify_end_of_interrupt 会自行判断中断信号发送的源头（主PIC或者副PIC），并使用指令和数据端口将信号发送到目标控制器。当然，如果是要发送到副PIC，那么结果上必然等同于同时发送到两个PIC，因为副PIC的输入管脚连在主PIC上面。
        // 这里的中断编码一定不可以写错，不然可能会导致某个中断信号迟迟得不到回应导致系统整体挂起。这也是该函数被标记为不安全的原因。
        // print!(".");
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

/// 键盘中断处理函数。
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    // 0x60 是键盘控制器的数据端口。需要从这个端口读取扫描码，才能知道用户按下了什么键。键盘中断只是通知我们有键盘输入，但是并不会告诉我们具体是什么键。
    // 键盘控制器会等我们读取完扫描码之后，才会发送下一个中断。
    let mut port = Port::new(0x60);
    // let mut keyboard = KEYBOARD.lock();
    let scan_code: u8 = unsafe { port.read() };
    add_scan_code(scan_code);

    // 通知 PIC 中断已经处理完毕。否则后续中断会一直排队。
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

// todo: 自己实现 x86 的页表
/// page fault 中断处理函数。
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    // 此处能工作的原因：x86强制要求内存模式必须是分页模式，所以在进入内核之前，bootloader 已经将页表激活了。
    // 除了 vga 外，其它目前使用的地址都是虚拟地址。vga 使用了一致映射，即虚拟地址和物理地址是一样的。
    println!("EXCEPTION: PAGE FAULT");
    // 在 page fault 发生时, x86 会自动将出错的地址写入到 CR2 寄存器中。
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}
