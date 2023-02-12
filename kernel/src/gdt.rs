use x86_64::{
    instructions::tables::load_tss,
    registers::segmentation::{Segment, CS},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static::lazy_static! {
    // TSS: 在32位模式下，TSS的作用是存储一些用于上下切换的零碎信息，如处理器的寄存器状态等。
    // 而在 64 位模式下，由于信息多，上下文切换已经不能使用硬件完成，所以 TSS 在64位模式已经没啥用了。
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        // ist 是一个数组，每个元素都是一个栈指针，用于处理特定的中断
        // ist 的作用是防止内核栈溢出时中断处理无法正常工作。比如，在内核中发生中断时，如果不进行栈切换，则一直在内核栈上递归触发中断的话，可能造成内核栈溢出，这是不可恢复的。所以 IST 实际上是定义了几个确定可用的栈供中断使用，当发生中断时，CPU 会自动切换到这些栈上，在进行中断处理程序调用，从而避免内核栈溢出。

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // 由于目前还不能分配空间，所以先用一个固定的栈
            const STACK_SIZE: usize = 4096*5;
            // 使用 mut 是为了让 bootloader 将它放到非只读空间
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // 栈：从高地址向低地址增长
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss

    };

    /// GDT: 全局描述符表，用于存储内核的段描述符。X86 CPU 由于兼容历史的原因，仍然是以段方式进行内存访问。
    /// 在页模式成为标准前，主要使用 GDT 来进行段访问控制。
    /// 所以当前使用 GDT 来进行对 TSS 栈进行加载
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

struct Selectors {
    code_selector: x86_64::structures::gdt::SegmentSelector,
    tss_selector: x86_64::structures::gdt::SegmentSelector,
}

pub fn init() {
    // lgdt 指令
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
