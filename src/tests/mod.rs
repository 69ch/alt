#[allow(unused)]
macro_rules! compile_test {
    ($p:literal) => {
        let code = include_str!($p);
        let tokens = lex(code);
        let mut bindings = Bindings::new(code, $p.into(), None);
        let program = parse_program(&tokens, &mut bindings);
        if bindings.is_compileable() {
            llc_test(program, bindings, 0);
        }
        else {
            panic!("This test isn't compilable");
        }
    };
}

mod namespaces;
mod fnptr;
mod control_flow;
mod tuples_and_arrays;
mod r#struct;