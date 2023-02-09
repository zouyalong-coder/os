// use std::{env, process::Command};
use std::env;

use nasm_rs;

fn main() {
    // let output = env::var("OUT_DIR").unwrap();
    // println!("output: {}", output);
    nasm_rs::Build::new()
        .flag("-fbin")
        .file("src/asm/boot.asm")
        .target("boot.bin")
        .compile_objects()
        .unwrap();
    // // nasm_rs::build_asm()
    // // Command::new("nasm")
    // //     .args(["-f", "bin", "-o", format!("{output}/boot.bin").as_str()])
    // //     .arg(format!("src/asm/boot.asm"))
    // //     .status()
    // //     .unwrap();
    nasm_rs::compile_library("boot.bin", &["src/asm"]).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    // println!("cargo:rerun-if-changed=src/asm/");
}
