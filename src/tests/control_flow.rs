#[cfg(test)]
mod tests {
    use crate::{compiler::llvm::llc_test, lexer::lex, parser::{bindings::Bindings, parse_program}};

    #[test]
    fn r#loop () {
        compile_test!("./control_flow/loop.alt");
    }

    #[test]
    fn if_else () {
        compile_test!("./control_flow/if_else.alt");
    }
}