#[cfg(test)]
mod tests {
    use crate::{compiler::llvm::llc_test, lexer::lex, parser::{bindings::Bindings, parse_program}};

    #[test]
    fn tuple () {
        compile_test!("./tuples_and_arrays/tuple.alt");
    }

    #[test]
    fn array () {
        compile_test!("./tuples_and_arrays/array.alt");
    }
}