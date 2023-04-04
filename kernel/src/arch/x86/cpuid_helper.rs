use modular_bitfield::{
    bitfield,
    specifiers::{B1, B2, B4},
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

/// cpuid eax=0x1 的 ebx 结果。CPU 基础信息。
#[bitfield]
pub struct A0x1ResultEbx {
    /// this number provides an entry into a brand string table that contains brand strings for IA-32 processors.
    pub brand_index: B8,
    /// this number indicates the size of the cache line flushed by the CLFLUSH and CLFLUSHOPT instructions in 8-byte increments. 单位为 8 字节。
    pub clflush_size: B8,
    pub logical_processor_count: B8,
    /// this number indicates the number of times the local APIC timer increments per microsecond.this number is the 8-bit ID that is assigned to the local APIC on the processor during power up
    pub apic_id: B8,
}

/// cpuid eax=0x1 的 ecx 结果。 CPU 功能。
/// 参考：https://www.felixcloutier.com/x86/cpuid#tbl-3-10
#[bitfield]
pub struct A0x1ResultEcx {
    /// 支持 Streaming SIMD Extensions 3。
    pub sse3: B1,
    /// 支持 PCLMULQDQ 指令。
    pub pclmulqdq: B1,
    /// 支持 64 位的 DS area.
    pub dtes64: B1,
    /// 支持监视点调试扩展。MONITOR/MWAIT.
    pub monitor: B1,
    /// the processor supports the extensions to the Debug Store feature to allow for branch message storage qualified by CPL.
    pub ds_cpl: B1,
    /// 支持虚拟化技术。Virtual Machine Extensions.
    pub vmx: B1,
    /// Safer Mode Extensions.
    pub smx: B1,
    /// Enhanced Intel SpeedStep® technology.
    pub eist: B1,
    /// Thermal Monitor 2.
    pub tm2: B1,
    /// 支持 SSSE3 指令。Supplemental Streaming SIMD Extensions 3
    pub ssse3: B1,
    /// 支持 L1 Context ID. 1 indicates the L1 data cache mode can be set to either adaptive mode or shared mode.
    pub cnxt_id: B1,
    /// IA32_DEBUG_INTERFACE MSR for silicon debug.
    pub sdbg: B1,
    /// 支持 FMA 指令。Fused Multiply Add.
    pub fma: B1,
    /// 支持 CMPXCHG16B 指令。CMPXCHG8B/CMPXCHG16B—Compare and Exchange Bytes.
    pub cx16: B1,
    /// 支持 xTPR Update Control. supports changing IA32_MISC_ENABLE[bit 23].
    pub xtp: B1,
    /// 支持 PDCM 指令。Perfmon and Debug Capability. Perfmon and Debug Capability
    pub pdcm: B1,
    reserved: B1,
    /// 支持 PCID。Process-context identifiers. supports PCIDs and that software may set CR4.PCIDE to 1.
    pub pcid: B1,
    /// 支持 DCA。Direct Cache Access. supports the ability to prefetch data from a memory mapped device.
    pub dca: B1,
    /// 支持 SSE4.1 指令。Streaming SIMD Extensions 4.1.
    pub sse4_1: B1,
    /// 支持 SSE4.2 指令。Streaming SIMD Extensions 4.2.
    pub sse4_2: B1,
    /// 支持 x2APIC。Extended xAPIC Support.
    pub x2apic: B1,
    /// 支持 MOVBE 指令。MOVBE—Move Data After Swapping Bytes.
    pub movbe: B1,
    /// 支持 POPCNT 指令。POPCNT—Count Number of Bits Set to 1.
    pub popcnt: B1,
    /// 支持 TSC-Deadline。Local APIC supports one-shot operation using a TSC deadline value.
    pub tsc_deadline: B1,
    /// 支持 AESNI 指令。AES—Advanced Encryption Standard.
    pub aesni: B1,
    /// 支持 XSAVE/XRSTOR 指令。XSAVE, XRESTOR, XSETBV, XGETBV, XCR0
    pub xsave: B1,
    /// 支持 OSXSAVE 指令。the OS has set CR4.OSXSAVE[bit 18] to enable XSETBV/XGETBV instructions to access XCR0 and to support processor extended state management using XSAVE/XRSTOR.
    pub osxsave: B1,
    /// 支持 AVX 指令。Advanced Vector Extensions.
    pub avx: B1,
    /// 支持 F16C 指令。F16C—16-bit Floating-Point Conversion Instructions.
    pub f16c: B1,
    /// 支持 RDRAND 指令。RDRAND—On-Chip Random Number Generator.
    pub rdrand: B1,
    unused: B1,
}

/// cpuid eax=0x1 的 edx 结果。 CPU 功能。
#[bitfield]
pub struct A0x1ResultEdx {
    /// 支持 FPU 协处理器。Floating Point Unit On-Chip. The processor contains an x87 FPU.
    pub fpu: B1,
    /// Virtual 8086 Mode Enhancements. Virtual 8086 mode enhancements, including CR4.VME for controlling the feature, CR4.PVI for protected mode virtual interrupts, software interrupt indirection, expansion of the TSS with the software indirection bitmap, and EFLAGS.VIF and EFLAGS.VIP flags.
    pub vme: B1,
    /// 支持 DE 指令。Debugging Extensions. Support for I/O breakpoints, including CR4.DE for controlling the feature, and optional trapping of accesses to DR4 and DR5.
    pub de: B1,
    /// Page Size Extension. Large pages of size 4 MByte are supported, including CR4.PSE for controlling the feature, the defined dirty bit in PDE (Page Directory Entries), optional reserved bit trapping in CR3, PDEs, and PTEs.
    pub pse: B1,
    /// Time Stamp Counter. The RDTSC instruction is supported, including CR4.TSD for controlling privilege.
    pub tsc: B1,
    /// 支持 MSR 指令。Model Specific Registers RDMSR and WRMSR Instructions. The RDMSR and WRMSR instructions are supported. Some of the MSRs are implementation dependent.
    pub msr: B1,
    /// 支持 PAE。Physical Address Extension. Physical addresses greater than 32 bits are supported: extended page table entry formats, an extra level in the page translation tables is defined, 2-MByte pages are supported instead of 4 Mbyte pages if PAE bit is 1.
    pub pae: B1,
    /// Machine Check Exception. Exception 18 is defined for Machine Checks, including CR4.MCE for controlling the feature. This feature does not define the model-specific implementations of machine-check error logging, reporting, and processor shutdowns. Machine Check exception handlers may have to depend on processor version to do model specific processing of the exception, or test for the presence of the Machine Check feature.
    pub mce: B1,
    /// CMPXCHG8B Instruction. The compare-and-exchange 8 bytes (64 bits) instruction is supported (implicitly locked and atomic).
    pub cx8: B1,
    /// APIC On-Chip. The processor contains an Advanced Programmable Interrupt Controller (APIC), responding to memory mapped commands in the physical address range FFFE0000H to FFFE0FFFH (by default - some processors permit the APIC to be relocated).
    pub apic: B1,
    reserved: B1,
    /// SYSENTER and SYSEXIT Instructions. The SYSENTER and SYSEXIT and associated MSRs are supported.
    pub sep: B1,
    /// Memory Type Range Registers. MTRRs are supported. The MTRRcap MSR contains feature bits that describe what memory types are supported, how many variable MTRRs are supported, and whether fixed MTRRs are supported.
    pub mtrr: B1,
    /// Page Global Bit. The global bit is supported in paging-structure entries that map a page, indicating TLB entries that are common to different processes and need not be flushed. The CR4.PGE bit controls this feature.
    pub pge: B1,
    /// Machine Check Architecture. A value of 1 indicates the Machine Check Architecture of reporting machine errors is supported. The MCG_CAP MSR contains feature bits describing how many banks of error reporting MSRs are supported.
    pub mca: B1,
    /// Conditional Move Instructions. The conditional move instruction CMOV is supported. In addition, if x87 FPU is present as indicated by the CPUID.FPU feature bit, then the FCOMI and FCMOV instructions are supported
    pub cmov: B1,
    /// Page Attribute Table. Page Attribute Table is supported. This feature augments the Memory Type Range Registers (MTRRs), allowing an operating system to specify attributes of memory accessed through a linear address on a 4KB granularity.
    pub pat: B1,
    /// 36-Bit Page Size Extension. 4-MByte pages addressing physical memory beyond 4 GBytes are supported with 32-bit paging. This feature indicates that upper bits of the physical address of a 4-MByte page are encoded in bits 20:13 of the page-directory entry. Such physical addresses are limited by MAXPHYADDR and may be up to 40 bits in size.
    pub pse_36: B1,
    /// Processor Serial Number. The processor supports the 96-bit processor identification number feature and the feature is enabled.
    pub psn: B1,
    /// CLFLUSH Instruction. CLFLUSH Instruction is supported.
    pub clflush: B1,
    reserved_2: B1,
    /// Debug Store. The processor supports the ability to write debug information into a memory resident buffer. This feature is used by the branch trace store (BTS) and processor event-based sampling (PEBS) facilities (see Chapter 23, “Introduction to Virtual-Machine Extensions,” in the Intel® 64 and IA-32 Architectures Software Developer’s Manual, Volume 3C).
    pub ds: B1,
    /// Thermal Monitor and Software Controlled Clock Facilities. The processor implements internal MSRs that allow processor temperature to be monitored and processor performance to be modulated in predefined duty cycles under software control.
    pub acpi: B1,
    /// Intel MMX Technology. The processor supports the Intel MMX technology.
    pub mmx: B1,
    /// FXSAVE and FXRSTOR Instructions. The FXSAVE and FXRSTOR instructions are supported for fast save and restore of the floating point context. Presence of this bit also indicates that CR4.OSFXSR is available for an operating system to indicate that it supports the FXSAVE and FXRSTOR instructions.
    pub fxsr: B1,
    /// 	SSE. The processor supports the SSE extensions.
    pub sse: B1,
    /// SSE2. The processor supports the SSE2 extensions.
    pub sse2: B1,
    /// 	Self Snoop. The processor supports the management of conflicting memory types by performing a snoop of its own cache structure for transactions issued to the bus.
    pub ss: B1,
    /// Max APIC IDs reserved field is Valid. A value of 0 for HTT indicates there is only a single logical processor in the package and software should assume only a single APIC ID is reserved. A value of 1 for HTT indicates the value in CPUID.1.EBX[23:16] (the Maximum number of addressable IDs for logical processors in this package) is valid for the package.
    pub htt: B1,
    /// Thermal Monitor. The processor implements the thermal monitor automatic thermal control circuitry (TCC).
    pub tm: B1,
    reserved_3: B1,
    /// Pending Break Enable. The processor supports the use of the FERR#/PBE# pin when the processor is in the stop-clock state (STPCLK# is asserted) to signal the processor that an interrupt is pending and that the processor should return to normal operation to handle the interrupt.
    pub pbe: B1,
}
