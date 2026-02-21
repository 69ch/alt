#[cfg(test)]
mod tests {
    use crate::{compiler::llvm::llc_test, lexer::lex, parser::{bindings::Bindings, parse_program}};

    #[test]
    fn first_class () {
        compile_test!("./fnptr/first_class.alt");
    }
}