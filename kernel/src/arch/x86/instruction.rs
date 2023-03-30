use core::arch::x86_64::{CpuidResult, __cpuid_count};

/// 检查 CPU 是否支持 CPUID 指令。
pub fn has_cpuid() -> bool {
    #[cfg(target_arch = "x86_64")]
    true
}

/// 获取 CPUID 指令的结果。
/// op: CPUID 指令的参数。
#[inline[always]]
pub unsafe fn cpuid(op: u32) -> CpuidResult {
    cpuid_count(op, 0)
}

#[inline[always]]
pub unsafe fn cpuid_count(op: u32, count: u32) -> CpuidResult {
    __cpuid_count(op, count)
}
