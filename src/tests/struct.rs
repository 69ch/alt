#[cfg(test)]
mod tests {
    use crate::{compiler::llvm::llc_test, lexer::lex, parser::{bindings::Bindings, parse_program}};

    #[test]
    #[should_panic]
    fn struct1 () {
        compile_test!("./struct/struct1.alt");
    }
}