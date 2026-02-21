mod components;
mod ir;
pub mod codegen;
use std::{fs::{create_dir_all, File}, io::Write, path::PathBuf, process::Command};

use codegen::emit_llvm;

use crate::parser::{bindings::Bindings, Value};

pub fn compile (program: Vec<Value>, bindings: Bindings, opt: u8) {
    let result = emit_llvm(program);
    print!("--- llvm output ---\n{result}");
    let mut f = File::create("./temp.ll").expect("Cannot access file system");
    f.write_all(&result.as_bytes()).expect("Failed writing to file");

    // .ll to .o
    let x = Command::new("llc")
    .arg("--filetype=obj")
    .arg("temp.ll")
    .arg("-o")
    .arg("temp.o")
    .arg(format!("-O{opt}"))
    .status().expect("Failed to execute 'llc' command. Add LLVM binaries in PATH.");

    if !x.success() { panic!("llc failed to compile IR to object file") }

    let y = Command::new("lld-link")
    .arg("temp.o")
    .args(bindings.move_links().iter())
    .arg("/entry:main")
    .status().expect("Failed to execute 'lld-link' command");

    if !y.success() { panic!("Failed linking stage") }
}

#[allow(dead_code)]
pub fn llc_test (program: Vec<Value>, bindings: Bindings, opt: u8) {
    let result = emit_llvm(program);
    let path = format!("./tests/{}.ll", bindings.get_current_file_path().to_str().unwrap());
    create_dir_all(PathBuf::from(&path).parent().unwrap()).unwrap();
    let mut f = File::create(&path).expect("Cannot access file system");
    f.write_all(&result.as_bytes()).expect("Failed writing to file");

    // .ll to .o
    let x = Command::new("llc")
    .arg("--filetype=obj")
    .arg(&path)
    .arg("-o")
    .arg(format!("./tests/{}.o", bindings.get_current_file_path().to_str().unwrap()))
    .arg(format!("-O{opt}"))
    .status().expect("Failed to execute 'llc' command. Add LLVM binaries in PATH.");

    if !x.success() { panic!("llc failed to compile IR to object file") }
}

// #[allow(dead_code)]
pub fn emit_asm (program: Vec<Value>) {
    let result = emit_llvm(program);
    let mut f = File::create("./temp.ll").expect("Cannot access file system");
    f.write_all(&result.as_bytes()).expect("Failed writing to file");

    // .ll to .s
    let x = Command::new("llc")
    .arg("--filetype=asm")
    .arg("temp.ll")
    .arg("-o")
    .arg("temp.s")
    .arg("-O0")
    .status().expect("Cannot execute 'llc' command. Add LLVM binaries in PATH.");

    if !x.success() { panic!("llc failed to compile IR to object file") }
}