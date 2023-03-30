use core::arch::x86_64::CpuidResult;

use super::{cpuid_helper, instruction};

const CPU_CAPABILITY_SIZE: usize = 10;

pub struct Cpu {
    /// CPU family
    family: u8,
    /// CPU 型号
    model: u8,
    /// CPU mask
    mask: u8,
    /// cache line size.
    clflush_size: u16,
    /// 物理地址位数
    phys_bits: u8,
    /// 虚拟地址位数
    virt_bits: u8,
    /// L1 cache line 大小，单位为字节。默认值为 clflush_size。
    cache_alignment: u32,

    /// CPU 厂商 ID
    vendor_id: [u8; 16],
    /// CPU 型号
    cpu_level: u32,
    /// CPU 功能
    capability: [u32; CPU_CAPABILITY_SIZE],
}

impl Cpu {
    pub fn new() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            let clflush_size = 64;
            let phys_bits = 52;
            let virt_bits = 48;
        }
        let cache_alignment = clflush_size as u32;
        Self {
            family: 4,
            model: 0,
            mask: 0,
            clflush_size,
            phys_bits,
            virt_bits,
            cache_alignment,
            vendor_id: [0; 16],
            cpu_level: 0,
            capability: [0; CPU_CAPABILITY_SIZE],
        }
    }

    /// cpu 检测。
    pub fn detect(&mut self) {
        // get vendor name
        let result = unsafe { instruction::cpuid(0) };
        self.cpu_level = result.eax;
        self.vendor_id[0..4] = result.ebx;
        self.vendor_id[4] = result.edx;
        self.vendor_id[8] = result.ecx;
        if self.cpu_level >= 1 {
            let CpuidResult {
                eax: junk,
                ebx: misc,
                ecx: junk,
                edx: cap0,
            } = unsafe { instruction::cpuid(1) };
            let eax_result = cpuid_helper::A0x1ResultEax::from(junk);
            self.family = eax_result.family() as u8;
            self.model = eax_result.model() as u8;
            self.mask = eax_result.stepping() as u8;
            // extend 修正。
            match self.family {
                0xf => self.family += (eax_result.extended_family() as u8),
                0x6 => self.model += (eax_result.extended_model() as u8) << 4,
                _ => (),
            }
        }
    }
}
