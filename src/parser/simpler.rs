use crate::{lexer::{Token, TokenKind}, parser::{components::{binds::access_type_member, pointer::take_pointer}, r#type::extract_type}};

use super::{bindings::Bindings, body::parse_pair_symbols, components::{args::{parse_args, Arg}, binds::{deref, load_address}}, message::err_expected_body, r#type::{parse_type, Type}, value, value_lookaround, Value};

pub fn next_body<'a> (off: &mut usize, tokens: &'a [Token], bindings: &Bindings, pair: (&'a str, &'a str)) -> &'a [Token] {
    let body = parse_pair_symbols(&tokens[*off..], pair).unwrap_or_else(|| err_expected_body(&tokens[*off], bindings, pair));
    let res = &tokens[*off+1..*off + body - 1];
    *off += body;
    res
}
pub fn next_body_optional<'a> (off: &mut usize, tokens: &'a [Token], pair: (&'a str, &'a str)) -> Option<&'a [Token]> {
    let body = parse_pair_symbols(tokens.get(*off..)?, pair)?;
    let res = &tokens[*off+1..*off + body - 1];
    *off += body;
    Some(res)
}

pub fn next_value<'a> (off: &mut usize, tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> Option<Value<'a>> {
    let ls = instructions.len();
    let value_jump = value_lookaround(tokens.get(*off..)?, instructions, bindings) + *off;
    let le = instructions.len();
    if ls != le {
        let res = instructions.pop()?;
        *off = value_jump;
        return Some(res)
    }
    None
}

#[macro_export]
macro_rules! nvalue {
    ($off:expr, $tokens:expr, $instructions:expr, $b:expr, $msg:expr) => {
        {
            // let abort = *$off;
            let Some(x) = next_value($off, $tokens, $instructions, $b) else { $b.gentle_error(&$tokens[*$off], $msg); return *$off };
            x
        }
    };
    ($off:expr, $tokens:expr, $instructions:expr, $b:expr, $msg:expr, $r:expr) => {
        {
            let Some(x) = next_value($off, $tokens, $instructions, $b) else { $b.gentle_error(&$tokens[*$off], $msg); return $r };
            x
        }
    };
    ($off:expr, $tokens:expr, $instructions:expr, $b:expr, $msg:expr, $o:expr, $r:expr) => {
        {
            let Some(x) = next_value($off, $tokens, $instructions, $b) else { $b.gentle_error($o, $msg); return $r };
            x
        }
    };
}

pub fn next_one_value<'a> (off: &mut usize, tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> Option<Value<'a>> {
    *off += value(tokens.get(*off..)?, instructions, bindings);
    instructions.pop()
}

#[macro_export]
macro_rules! novalue {
    ($off:expr, $tokens:expr, $instructions:expr, $b:expr, $msg:expr, $o:expr, $r:expr) => {
        {
            let Some(val) = next_one_value($off, $tokens, $instructions, $b) else { $b.gentle_error(&$tokens[0], $msg); return *$off };
            val
        }
    };
}

pub fn next_token<'a> (off: &mut usize, tokens: &'a [Token], value: Option<&'a str>, typ: Option<TokenKind>) -> Option<&'a Token> {
    let res = tokens.get(*off);
    if let Some(tk @ Token { typ: tktyp, value: tkvalue, .. }) = res {
        match (value, typ) {
            (Some(value), Some(typ)) => {
                if value == tkvalue && typ == *tktyp {
                    *off += 1;
                    return Some(tk)
                }
                return None
            }
            (_, Some(typ)) => {
                if typ == *tktyp {
                    *off += 1;
                    return Some(tk)
                }
                return None
            }
            (Some(value), _) => {
                if value == tkvalue {
                    *off += 1;
                    return Some(tk)
                }
                return None
            }
            _ => {
                *off += 1;
                return Some(tk)
            }
        }
    }
    None
}

pub fn next_args<'a> (off: &mut usize, tokens: &'a [Token], bindings: &mut Bindings<'a>) -> Vec<Arg<'a>> {
    let Some(tt) = next_body_optional(off, tokens, ("(", ")")) else { return vec![] };
    let (res, _) = parse_args(tt, bindings);
    res
}

pub fn next_type<'a> (off: &mut usize, tokens: &'a [Token], bindings: &mut Bindings<'a>) -> Option<Type> {
    let (typ, j) = parse_type(tokens.get(*off..)?, bindings)?;
    *off += j;
    Some(typ)
}

pub fn next_mutable_flag (off: &mut usize, tokens: &[Token]) -> bool {
    next_token(off, tokens, Some("mut"), Some(TokenKind::Word)).is_some()
}

// TODO: MAYBE MERGE THESE FUNCTIONS IN ONE (because of, for example, cases like "foo.bar[0].abc[2]")

pub fn next_load_address<'a> (off: &mut usize, tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>, mutable: bool) -> Option<()> {
    let mut value = &tokens.get(*off)?.value;
    while value == "[" {
        *off += load_address(&tokens[*off..], instructions, bindings, mutable);
        if let Some(Token { value: v, .. }) = tokens.get(*off) {
            value = v;
        }
        else { break }
    }
    Some(())
}
pub fn next_access_type_member<'a> (off: &mut usize, tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>, mutable: bool) -> Option<()> {
    let mut value = &tokens.get(*off)?.value;
    while value == "." {
        *off += access_type_member(&tokens[*off..], instructions, bindings, mutable);
        if let Some(Token { value: v, .. }) = tokens.get(*off) {
            value = v;
        }
        else { break }
    }
    Some(())
}

pub fn next_deref<'a> (off: &mut usize, tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> Option<()> {
    let mut token = &tokens.get(*off)?.value;
    while token == ".*" {
        *off += deref(&tokens[*off..], instructions, bindings);
        if let Some(Token { value: v, .. }) = tokens.get(*off) {
            token = v;
        }
        else { break }
    }
    Some(())
}

pub fn ignore_separator<'a> (off: &mut usize, tokens: &'a [Token]) {
    next_token(off, tokens, None, Some(TokenKind::Punctuation));
}

pub fn next_name<'a> (off: &mut usize, tokens: &'a [Token]) -> String {
    let mut buf = String::new();
    loop {
        let Some(Token { value: np, .. }) = next_token(off, tokens, None, Some(TokenKind::Word)) else { todo!() };
        buf += np;
        if let Some(Token { value: cc, .. }) = next_token(off, tokens, Some("::"), Some(TokenKind::Special)) {
            buf += cc;
        }
        else { break }
    }
    // todo generics
    buf
}

fn fill_argv<'a> (off: &mut usize, argt: Vec<Type>, argv: &mut Vec<Value<'a>>, body: &'a [Token], tokens: &'a [Token], spoint: usize, instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) {
    let argoff = argv.len();
    for (p, typ) in argt[..argoff].iter().enumerate() {
        typ.check_strict(&argv[p], &tokens[spoint..=spoint], bindings);
    }
    for typ in &argt[argoff..] {
        let start = *off;
        let value = nvalue!(off, body, instructions, bindings, "Expected function argument", &tokens[spoint+*off], ());
        typ.check_strict(&value, &body[start..*off], bindings);
        ignore_separator(off, body);
        argv.push(value);
    }
}

fn construct_call<'a> (x: Value<'a>, argv: Vec<Value<'a>>, tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, ret: Type) -> usize {
    instructions.push(Value::Call(Box::new(x), argv));
    if ret == Type::Noret { instructions.push(Value::Unreachable); return tokens.len() }
    0
}

pub fn next_call<'a> (off: &mut usize, tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) {
    let mut sb = *off;
    while let Some(body) = next_body_optional(off, tokens, ("(", ")")) {
        let x = instructions.pop().unwrap();
        let Some(Type::Fn(argt, ret)) = extract_type(&x) else { instructions.push(x); *off = sb; return () };
        let mut argv = vec![];
        fill_argv(&mut 0, argt, &mut argv, body, tokens, sb, instructions, bindings);
        *off += construct_call(x, argv, tokens, instructions, *ret);
        sb = *off;
    }
}
pub fn next_uniform_call<'a> (off: &mut usize, mut farg: Value<'a>, tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) {
    let sb = *off;
    if let Some(body) = next_body_optional(off, tokens, ("(", ")")) {
        let x = instructions.pop().unwrap();
        let Some(Type::Fn(argt, ret)) = extract_type(&x) else { instructions.push(x); *off = sb; return () };
        if let Some(Type::Ptr(Some(_t), m)) = argt.get(0) {
            if let None = argt[0].check(&farg) {
                farg = take_pointer(&tokens[..*off], bindings, *m, farg);
            }
            // t.check_strict(&farg, &tokens[..*off], bindings);
        }
        let mut argv = vec![farg];
        fill_argv(&mut 0, argt, &mut argv, body, tokens, sb, instructions, bindings);
        *off += construct_call(x, argv, tokens, instructions, *ret);
    }
}