use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

/// 通过当前页表的起始偏移，获取当前进程（或内核）的页表指针（虚拟地址）。
fn active_level_4_table(physical_address_offset: u64) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    // Cr3 是 x86 存放当前页表地址的寄存器，它的值是当前页表的物理地址
    let (level_4_table_frame, _) = Cr3::read();
    // 由于 CPU 都是使用的虚拟地址，而页表又需要物理地址（如果是虚拟地址，则会陷入地址转换的循环）。如果直接采用
    // 一致映射，则不同进程的页表会冲突。所以页表部分地址映射采用偏移的方法，一方面使CPU能够正确访问页表，另一方面
    // 防止不同进程的页表冲突。
    let phys = level_4_table_frame.start_address();
    // 寄存器内当前页表的物理地址加上固定的偏移，即为虚拟地址。
    let virt = VirtAddr::new(physical_address_offset + phys.as_u64());
    // 此时的虚拟地址指向的就是 4级页表的地址。
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    unsafe { &mut *page_table_ptr }
}

#[allow(unused)]
/// 自己实现的地址转换函数，用于将虚拟地址转换为物理地址。
fn translate_addr(addr: VirtAddr, phy_addr_offset: u64) -> Option<PhysAddr> {
    translate_addr_inner(addr, phy_addr_offset)
}

fn translate_addr_inner(addr: VirtAddr, phy_addr_offset: u64) -> Option<PhysAddr> {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    // 从 Cr3 读取当前页表的物理地址
    let (level_4_table_frame, _) = Cr3::read();
    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level_4_table_frame;
    for index in table_indexes {
        // 此时所有页表的虚拟地址=》物理地址的映射方式都是线性的，所以可以直接通过偏移计算出虚拟地址。
        let virt = VirtAddr::new(phy_addr_offset + frame.start_address().as_u64());
        let table_ptr: *const PageTable = virt.as_mut_ptr();
        let table = unsafe { &*table_ptr };
        let entry = &table[index];
        // 获取当前 PTE 指向的页帧
        // PTE 上 12~51 位是页帧的物理地址，共 40 位，指向的是 4KB 的页帧。 对 x86 来说，页大小 12位 + 40 位 = 52 位 地址空间。
        // x86_64平台仅支持52位物理地址，所以页帧的物理地址最大为 2^52 = 。
        // x86_64 仅支持48位虚拟地址，所以虚拟地址最大为 2^48 = 256TB（虚拟地址的 offset 是9位）。
        frame = match entry.frame() {
            // 正确映射，返回页帧
            Ok(frame) => frame,
            // 未映射，返回 None
            Err(FrameError::FrameNotPresent) => return None,
            // 大页帧，目前不支持
            Err(FrameError::HugeFrame) => unimplemented!("huge page is not supported"),
        };
    }
    // 目标页帧的物理地址 + 目标页帧内偏移 = 目标物理地址
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

pub fn init(phy_addr_offset: u64) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(phy_addr_offset);
    unsafe { OffsetPageTable::new(level_4_table, VirtAddr::new(phy_addr_offset)) }
}

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}

/// 用于测试的函数
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flag = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let map_to_result = unsafe {
        // 仅用于测试，map_to 的页表项必须是未映射的
        // frame 未vga所在的页帧，这里不同的page可能会触发中间页表的分配，所以这里需要传入frame_allocator
        mapper.map_to(page, frame, flag, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}
