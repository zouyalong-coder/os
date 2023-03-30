use modular_bitfield::{
    bitfield,
    specifiers::{B2, B4},
};

/// cpuid eax=0x1 的 eax 结果。CPU 基础信息。
#[bitfield]
pub struct A0x1ResultEax {
    pub stepping: B4,
    pub model: B4,
    pub family: B4,
    pub processor_type: B2,
    reserved: B2,
    pub extended_model: B4,
    pub extended_family: B8,
}

/// cpuid eax=0x1 的 ecx 结果。 CPU 功能。
#[bitfield]
pub struct A0x1ResultEcx {}

/// cpuid eax=0x1 的 edx 结果。 CPU 功能。
#[bitfield]
pub struct A0x1ResultEdx {}
