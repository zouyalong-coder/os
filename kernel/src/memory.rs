use x86_64::{structures::paging::PageTable, VirtAddr};

pub fn active_level_4_table(physical_address_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    // Cr3 是 x86 存放当前页表地址的寄存器，它的值是当前页表的物理地址
    let (level_4_table_frame, _) = Cr3::read();
    // 由于 CPU 都是使用的虚拟地址，而页表又需要物理地址（如果是虚拟地址，则会陷入地址转换的循环）。如果直接采用
    // 一致映射，则不同进程的页表会冲突。所以页表部分地址映射采用偏移的方法，一方面使CPU能够正确访问页表，另一方面
    // 防止不同进程的页表冲突。
    let phys = level_4_table_frame.start_address();
    let virt = physical_address_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    unsafe { &mut *page_table_ptr }
}
