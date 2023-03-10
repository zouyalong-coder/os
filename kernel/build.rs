use build_target::target_arch;

fn main() {
    match target_arch().unwrap() {
        build_target::Arch::X86_64 => {
            x86_linker();
        }
        _ => todo!(),
    }
}

fn x86_linker() {
    println!("cargo:rerun-if-changed=kernel/src/arch/x86/linker.ld");
    println!("cargo:rustc-link-arg=-Tkernel/src/arch/x86/linker.ld");
}
