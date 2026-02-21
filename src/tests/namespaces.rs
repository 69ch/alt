#[allow(unused)]
macro_rules! nstest {
    ($p:literal, compileable) => {
        {
            let code = include_str!($p);
            let tokens = lex(code);
            let mut bindings = Bindings::new(code, $p.into(), None);
            parse_program(&tokens, &mut bindings);
            bindings.is_compileable()
        }
    };
}
#[cfg(test)]
mod tests {
    use crate::{compiler::llvm::llc_test, lexer::lex, parser::{bindings::Bindings, parse_program}};

    #[test]
    fn puberr () {
        if nstest!("./namespaces/puberr.alt", compileable) {
            panic!();
        }
    }
    #[test]
    fn pubnorm () {
        compile_test!("./namespaces/pubnorm.alt");
    }

    #[test]
    fn usenorm () {
        compile_test!("./namespaces/usenorm.alt");
    }
    #[test]
    fn useerr () {
        if nstest!("./namespaces/useerr.alt", compileable) {
            panic!();
        }
    }

    #[test]
    fn lazy_fn_parse () {
        compile_test!("./namespaces/lazy_fn_parse.alt");
    }
}