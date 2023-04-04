use core::arch::x86_64::CpuidResult;

use super::{cpu_features::*, cpuid_helper, instruction};

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
    capability: CpuCapabilityBits,
    /// CPU 扩展功能。Max extended CPUID function supported.
    extended_cpuid_level: u32,
    power: u32,
    smp_index: u16,
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
            capability: cpu_features::CpuCapabilityBits::new(),
        }
    }

    /// feature 检测。
    pub fn has_feature(&self, pos: &CapabilityBitLocation) -> bool {
        self.capability.get_bit(pos)
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
            let ebx_result = cpuid_helper::A0x1ResultEbx::from(misc);
            self.family = eax_result.family() as u8;
            self.model = eax_result.model() as u8;
            self.mask = eax_result.stepping() as u8;
            // extend 修正。
            match self.family {
                0xf => self.family += (eax_result.extended_family() as u8),
                0x6 => self.model += (eax_result.extended_model() as u8) << 4,
                _ => (),
            }
            let edx_result = cpuid_helper::A0x1ResultEdx::from(cap0);
            if edx_result.clflush() > 0 {
                self.clflush_size = ebx_result.clflush_size() as u16 * 8;
                self.cache_alignment = self.clflush_size as u32;
            }
        }
    }

    pub fn get_cpu_cap(&mut self) {
        /* Intel-defined flags: level 0x00000001 */
        if self.cpu_level >= 0x1 {
            let CpuidResult {
                ecx: excap,
                edx: capability,
                ..
            } = unsafe { instruction::cpuid(0x1) };
            self.capability[0] = capability;
            self.capability[4] = excap;
        }
        /* Additional Intel-defined flags: level 0x00000007 */
        if self.cpuid_level >= 0x7 {
            let CpuidResult { eax, ebx, .. } = unsafe { instruction::cpuid_count(0x7, 0) };
            if eax > 0 {
                self.capability[9] = ebx;
            }
        }
        /* AMD-defined flags: level 0x80000001 */
        let CpuidResult { xlvl, .. } = unsafe { instruction::cpuid(0x80000000) };
        self.extended_cpuid_level = xlvl;

        if xlvl & 0xffff_0000 == 0x8000_0000 {
            if xlvl >= 0x8000_0001 {
                let CpuidResult { ecx, edx, .. } = unsafe { instruction::cpuid(0x8000_0001) };
                self.capability[1] = edx;
                self.capability[6] = ecx;
            }
        }
        if xlvl >= 0x8000_0008 {
            let CpuidResult { eax, .. } = unsafe { instruction::cpuid(0x8000_0008) };
            self.virt_bits = (eax >> 8) as u8;
            self.phys_bits = eax as u8;
        }
        if xlvl >= 0x8000_0007 {
            let CpuidResult { edx, .. } = unsafe { instruction::cpuid(0x8000_0007) };
            self.power = edx;
        }
        init_scattered_cpuid_features(&mut self);
    }
}

macro_rules! def_cpuid_bit {
    ($name:ident, $reg:ident, $offset:literal, $level:literal, $sub_leaf:literal) => {
        CpuidBit {
            feature: $name,
            reg: Reg::$reg,
            offset: $offset,
            level: $level,
            sub_leaf: $sub_leaf,
        }
    };
}

fn init_scattered_cpuid_features(cpu: &mut Cpu) {
    enum Reg {
        EAX = 0,
        EBX,
        ECX,
        EDX,
    };
    struct CpuidBit {
        feature: CapabilityBitLocation,
        reg: Reg,
        offset: u8,
        level: u32,
        sub_leaf: u32,
    }
    let cpuid_bits: [CpuidBit; _] = [
        def_cpuid_bit!(FPU, EAX, 0, 0x6, 0),
        def_cpuid_bit!(IDA, EAX, 1, 0x6, 0),
        def_cpuid_bit!(ARAT, EAX, 2, 0x6, 0),
        def_cpuid_bit!(PLN, EAX, 4, 0x6, 0),
        def_cpuid_bit!(PTS, EAX, 6, 0x6, 0),
        def_cpuid_bit!(APERFMPERF, ECX, 0, 0x6, 0),
        def_cpuid_bit!(EPB, ECX, 3, 0x6, 0),
        def_cpuid_bit!(XSAVEOPT, EAX, 0, 0xd, 1),
        def_cpuid_bit!(CPB, EDX, 9, 0x8000_0007, 0),
        def_cpuid_bit!(NPT, EDX, 0, 0x8000_000a, 0),
        def_cpuid_bit!(LBRV, EDX, 1, 0x8000_000a, 0),
        def_cpuid_bit!(SVML, EDX, 2, 0x8000_000a, 0),
        def_cpuid_bit!(NRIPS, EDX, 3, 0x8000_000a, 0),
        def_cpuid_bit!(TSCRATEMSR, EDX, 4, 0x8000_000a, 0),
        def_cpuid_bit!(VMCBCLEAN, EDX, 5, 0x8000_000a, 0),
        def_cpuid_bit!(FLUSHBYASID, EDX, 6, 0x8000_000a, 0),
        def_cpuid_bit!(DECODEASSISTS, EDX, 7, 0x8000_000a, 0),
        def_cpuid_bit!(PAUSEFILTER, EDX, 10, 0x8000_000a, 0),
        def_cpuid_bit!(PFTHRESHOLD, EDX, 12, 0x8000_000a, 0),
    ];
    for cb in cpuid_bits {
        let CpuidResult { eax: max_level, .. } =
            unsafe { instruction::cpuid(cb.level & 0xffff_0000) };
        if max_level < cb.level || max_level > cb.level | 0xffff {
            continue;
        }
        let CpuidResult { eax, ebx, ecx, edx } =
            unsafe { instruction::cpuid_count(cb.level, cb.sub_leaf) };
        let registers = [eax, ebx, ecx, edx];
        if registers[cb.reg as usize] & (1 << cb.offset) != 0 {
            cpu.capability.set_bit(&cb.feature);
        }
    }
}

struct CpuidDependentFeature {
    feature: CapabilityBitLocation,
    level: u32,
}

// lazy_static! {
//     static ref CPUID_DEPENDENT_FEATURES: [CpuidDependentFeature; _] = [
//         CpuidDependentFeature {
//             feature: MWAIT,
//             level: 0x5
//         },
//         CpuidDependentFeature {
//             feature: DCA,
//             level: 0x9
//         },
//         CpuidDependentFeature {
//             feature: XSAVE,
//             level: 0xd
//         },
//     ];
// }

fn filter_cpu_features(cpu: &mut Cpu) {
    let CPUID_DEPENDENT_FEATURES: [CpuidDependentFeature; _] = [
        CpuidDependentFeature {
            feature: MWAIT,
            level: 0x5,
        },
        CpuidDependentFeature {
            feature: DCA,
            level: 0x9,
        },
        CpuidDependentFeature {
            feature: XSAVE,
            level: 0xd,
        },
    ];
    for df in CPUID_DEPENDENT_FEATURES.iter() {
        if !cpu.has_feature(&df.feature) {
            continue;
        }
        /*
         * Note: cpuid_level is set to -1 if unavailable, but
         * extended_extended_level is set to 0 if unavailable
         * and the legitimate extended levels are all negative
         * when signed; hence the weird messing around with
         * signs here...
         */
        if (df.level as i32) < 0 {
            if df.level <= cpu.extended_cpuid_level {
                continue;
            }
        } else {
            if df.level as i32 <= cpu.cpuid_level {
                continue;
            }
        }
        cpu.capability.clear_bit(&df.feature);
    }
}
