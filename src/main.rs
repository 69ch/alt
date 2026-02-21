use std::{env, fs, path::absolute};

// use compiler::codegen;
use compiler::{
    // ir::{translate_all, x86_display::display_instructions, IRb},
    llvm::{compile, emit_asm},
};
use lexer::lex;
use parser::{bindings::Bindings, parse_program};

mod lexer;
mod parser;
mod compiler;
mod tests;

fn main () {
    let args: Vec<String> = env::args().collect();
    let mut asm = false;
    let mut optimization = 1;
    for arg in args {
        if arg.starts_with("-O") {
            let Ok(x) = arg.as_str()[2..].parse::<u8>() else { panic!("Failed to parse argument {arg}") };
            if x > 3 { panic!("Optimization level of {x} is invalid (maximum is 3)") }
            optimization = x;
        }
        match arg.as_str() {
            "--emit-asm" => asm = true,
            _ => {}
        }
    }
    let path = absolute("./bober.alt").unwrap();
    let f = String::from_utf8(fs::read(&path).unwrap()).unwrap();
    let tokens = lex(&f);

    let mut bindings = Bindings::new(&f, path, None);
    let program = parse_program(&tokens, &mut bindings);
    // println!("{program:#?}");

    // [DONE]: return handling
    // [DONE]: if-else and logical expressions
    // [DONE]: loops (btw, for this needed reimplement allocation)
    // [DONE]: arrays and pointer arithmetics
    // [DONE]: arrays as value through temp. returning arrays
    // [DONE]: tuples (FIX FOR CORRENT PTR HANDLING. MAYBE SAME IMPL AS ARRAYS)
    // [DONE]: multiple errors
    // [DONE]: namespaces, 'use' and 'pub'
    // [DONE]: partial parsing of functions, even with namespaces
    // (TODOOOOO): tests for all shit that I made so far (control flow instructions, mutability, namespaces, functions and function pointers, pointers, type casting)
    // (todo): structs
    // (todo): 'defer' and new variable handling for this if needed
    // (todo): floating point operations support
    // (todo): templates

    if bindings.is_compileable() {
        if asm { emit_asm(program) }
        else { compile(program, bindings, optimization) }
    }
}
