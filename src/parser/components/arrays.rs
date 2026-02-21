use crate::{lexer::Token, parser::{bindings::{Bindings, Context}, parse, simpler::next_body, Value}};

// use super::binds::reserve_local;

pub fn array<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let mut off = 0;
    // reserve_local(bindings);
    let body = next_body(&mut off, tokens, bindings, ("[", "]"));
    
    bindings.context_scope(Context::Array, |bindings| {
        let body = parse(body, bindings);
        // for i in &body {
        //     if !produces_value(i) {
        //         error_range(&tokens[..off], bindings, "Arrays are not allowed yet to contain instructions")
        //     }
        // }
        instructions.push(Value::Array(body));
    });
    return off
}