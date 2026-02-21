use crate::{lexer::{Token, TokenKind}, parser::{bindings::Bindings, simpler::{ignore_separator, next_mutable_flag, next_token, next_type}, r#type::Type}};

#[derive(Debug, Clone)]
pub struct Arg<'a> { pub name: &'a str, pub typ: Type, pub mutable: bool }

// MAYBE TODO: automatic type filling (exmaple: a b i32 = a i32 b i32) by special Guess type

pub fn parse_args<'a> (tokens: &'a [Token], bindings: &mut Bindings<'a>) -> (Vec<Arg<'a>>, usize) {
    let mut args: Vec<Arg> = vec![];
    let mut last = 0;
    loop {
        let mutable = next_mutable_flag(&mut last, tokens);
        if let Some(i) = next_token(&mut last, tokens, None, Some(TokenKind::Word)) {
            let Some(typ) = next_type(&mut last, tokens, bindings) else { return (args, last) };
            args.push(Arg { name: &i.value, typ, mutable });
            ignore_separator(&mut last, tokens);            
        }
        else { break }
    }
    
    (args, last)
}