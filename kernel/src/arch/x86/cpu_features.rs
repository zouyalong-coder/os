const CPU_CAPABILITY_SIZE: usize = 10;

pub struct CpuCapabilityBits([u32; CPU_CAPABILITY_SIZE]);

impl CpuCapabilityBits {
    pub fn new() -> Self {
        Self([0; CPU_CAPABILITY_SIZE])
    }

    pub fn set_bit(&mut self, pos: &CapabilityBitLocation) {
        self.0[pos.slot as usize] |= 1 << pos.offset;
    }

    pub fn get_bit(&self, pos: &CapabilityBitLocation) -> bool {
        self.0[pos.slot as usize] & (1 << pos.offset) != 0
    }

    pub fn clear_bit(&mut self, pos: &CapabilityBitLocation) {
        self.0[pos.slot as usize] &= !(1 << pos.offset);
    }

    pub fn set_slot(&mut self, slot: usize, value: u32) {
        self.0[slot] = value;
    }

    pub fn get_slot_ref(&self, slot: usize) -> &u32 {
        &self.0[slot]
    }

    pub fn get_slot_mut(&mut self, slot: usize) -> &mut u32 {
        &mut self.0[slot]
    }
}

// #[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct CapabilityBitLocation {
    slot: u16,
    offset: u16,
}

impl CapabilityBitLocation {
    pub fn get_position(&self) -> u16 {
        self.slot * 32 + self.offset
    }

    pub fn from_position(pos: u16) -> Self {
        Self {
            slot: pos / 32,
            offset: pos % 32,
        }
    }
}

impl From<u16> for CapabilityBitLocation {
    fn from(pos: u16) -> Self {
        Self::from_position(pos)
    }
}

macro_rules! define_capability_bit {
    ($name:ident, $slot:expr, $offset:expr) => {
        pub const $name: CapabilityBitLocation = CapabilityBitLocation {
            slot: $slot,
            offset: $offset,
        };
    };
}

/// Onboard FPU
define_capability_bit!(FPU, 0, 0);
define_capability_bit!(APERFMPERF, 3, 28);
/// "monitor" Monitor/Mwait support
define_capability_bit!(MWAIT, 4, 3);
/// Direct Cache Access
define_capability_bit!(DCA, 4, 18);
/// XSAVE/XRSTOR/XSETBV/XGETBV
define_capability_bit!(XSAVE, 4, 26);
/// Intel Dynamic Acceleration
define_capability_bit!(IDA, 7, 0);
/// Always Running APIC Timer
define_capability_bit!(ARAT, 7, 1);
/// AMD Core Performance Boost
define_capability_bit!(CPB, 7, 2);
/// IA32_ENERGY_PERF_BIAS support
define_capability_bit!(EPB, 7, 3);
/// Optimized Xsave
define_capability_bit!(XSAVEOPT, 7, 4);
/// Intel Power Limit Notification
define_capability_bit!(PLN, 7, 5);
/// Intel Package Thermal Status
define_capability_bit!(PTS, 7, 6);
/// Digital Thermal Sensor
define_capability_bit!(DTS, 7, 7);
/// AMD Nested Page Table support
define_capability_bit!(NPT, 8, 5);
/// AMD LBR Virtualization support
define_capability_bit!(LBRV, 8, 6);
/// "svm_lock" AMD SVM locking MSR
define_capability_bit!(SVML, 8, 7);
/// "nrip_save" AMD SVM next_rip save
define_capability_bit!(NRIPS, 8, 8);
/// "tsc_scale" AMD TSC scaling support
define_capability_bit!(TSCRATEMSR, 8, 9);
/// "vmcb_clean" AMD VMCB clean bits support
define_capability_bit!(VMCBCLEAN, 8, 10);
/// AMD flush-by-ASID support
define_capability_bit!(FLUSHBYASID, 8, 11);
/// AMD Decode Assists support
define_capability_bit!(DECODEASSISTS, 8, 12);
/// AMD filtered pause intercept
define_capability_bit!(PAUSEFILTER, 8, 13);
/// AMD pause filter threshold
define_capability_bit!(PFTHRESHOLD, 8, 14);
