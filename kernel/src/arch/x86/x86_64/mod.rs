pub enum Cpu {}

impl super::cpu::CpuInfo for Cpu {
    const clflush_size: u16 = 64;

    const phys_bits: u8 = 52;

    const virt_bits: u8 = 48;

    const cache_alignment: int = Self::clflush_size as int;
}
