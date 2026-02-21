use std::{cell::RefCell, collections::HashMap, rc::Rc};

use insordmap::InsordMap;

use crate::{lexer::{Token, TokenKind}, parser::{components::{arrays::array, binds::{ASSIGN_NOT_EXPECTED, external_word, r#impl, join_by_path, namespace, r#pub, r#use, var}, control_flow::{r#break, r#continue, r#if, r#loop}, r#fn::{extrn, r#fn, parse_fn, r#return}, types::{parse_struct, r#struct, typecast}, unary::unary}, simpler::{next_access_type_member, next_body, next_token}}};

use super::{bindings::Bindings, components::{args::Arg, binds::modify, expr::expr}, simpler::{next_deref, next_load_address}, strings::EscapeGen, r#type::Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation { Add, Sub, Mul, Div, Rem, LE, LT, GT, GE, Eq, NE, And, Or, Not, BitAnd, BitOr }

#[derive(Debug, Clone)]
pub enum Value<'a> {
    String(String), Int(usize), SInt(isize), Bool(bool), Float(f64), Expr(Box<(Value<'a>, Value<'a>, Operation)>), Unary(Box<(Operation, Value<'a>)>),
    Array(Vec<Value<'a>>), Tuple(Vec<Value<'a>>),
    Get(&'a str, Type),
    Ptr(Box<Value<'a>>, bool),
    LoadFromPtr(Box<Value<'a>>, Type),
    LoadAddress(Box<Value<'a>>, Box<Value<'a>>, Type, bool),
    // LoadField(Box<Value<'a>>, usize, Type),
    
    // instructions
    // Function { name: &'a str, args: Vec<Arg<'a>>, body: Vec<Value<'a>>, ret: Type },
    Function { name: String, args: Vec<Arg<'a>>, body: Vec<Value<'a>>, ret: Type },
    PromisedFunction { name: String, args: Vec<Arg<'a>>, body: &'a [Token], ret: Type, token: &'a Token },
    FunctionPointer(String, Type, Vec<Type>),
    AnonFunction { args: Vec<Arg<'a>>, body: Vec<Value<'a>>, ret: Type },
    Call(Box<Value<'a>>, Vec<Value<'a>>),
    InitVar(&'a str, Type, Option<Box<Value<'a>>>), ModifyVar(&'a str, Type, Box<Value<'a>>),
    ModifyByPointer(Box<(Value<'a>, Type, Value<'a>)>),
    Return(Box<(Option<Value<'a>>, Type)>), ReturnMark,

    Extern(&'a str, Vec<Type>, Type),

    If { condition: Box<Value<'a>>, body: Vec<Value<'a>>, else_then: Box<Option<Value<'a>>> },
    Else(Vec<Value<'a>>),
    Loop(Vec<Value<'a>>, Option<&'a str>), Break(Option<&'a str>), Continue(Option<&'a str>), Unreachable,
    
    Typecast(Box<Value<'a>>, Type, Type),
    Namespace(String),
    PromisedStruct { name: String, body: &'a [Token] }, Struct { name: String, kv: InsordMap<String, Type>, alignment: u32, size: usize },
    StructInit(String, HashMap<&'a String, Value<'a>>),

    SharedValue(SharedValue<'a>)
}

pub type SharedValue<'a> = Rc<RefCell<Value<'a>>>;

pub fn parse_inplace<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) {
    let mut i = 0;
    let len = tokens.len();
    loop {
        if i >= len { break }
        i += value_lookaround(&tokens[i..], instructions, bindings);
    }
}

pub fn parse<'a> (tokens: &'a [Token], bindings: &mut Bindings<'a>) -> Vec<Value<'a>> {
    let mut instructions = vec![];
    parse_inplace(tokens, &mut instructions, bindings);
    instructions
}

pub fn parse_np<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) {
    parse_inplace(tokens, instructions, bindings);
    fn_pass(bindings);
    // type_pass(bindings);
}

pub fn parse_program<'a> (tokens: &'a [Token], bindings: &mut Bindings<'a>) -> Vec<Value<'a>> {
    let instructions = parse(tokens, bindings);
    type_pass(bindings);
    fn_pass(bindings);
    instructions
}

pub fn value<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let Some(token) = tokens.get(0) else { return 1 };

    match token.typ {
        TokenKind::String => instructions.push(Value::String(token.value.to_string().escape_escaping())),
        TokenKind::Int => instructions.push(Value::Int(token.value.parse().unwrap())),
        TokenKind::Arithmetic | TokenKind::Logical | TokenKind::Bitwise | TokenKind::Special => return unary(tokens, instructions, bindings),
        TokenKind::Assign => bindings.gentle_error(token, ASSIGN_NOT_EXPECTED),
        // TokenKind::In => return load_field(tokens, instructions, bindings),
        TokenKind::Brackets => match token.value.as_str() {
            "(" => {
                let mut off = 0;
                let body = next_body(&mut off, tokens, bindings, ("(", ")"));
                let mut bl = parse(body, bindings);
                if bl.len() != 1 {
                    // reserve_local(bindings);
                    instructions.push(Value::Tuple(bl));
                }
                else {
                    instructions.push(bl.pop().unwrap());
                }
                return off
            }
            "[" => return array(tokens, instructions, bindings),
            _ => { dbg!(&token.value); }
        }
        TokenKind::Meta => match &token.value.as_str()[1..] {
            "link" => {
                let mut off = 1;
                let Some(Token { value: lib, .. }) = next_token(&mut off, tokens, None, Some(TokenKind::String)) else { bindings.gentle_error(&tokens[0], "#link requires path (string) next to it"); return off };
                bindings.link(&lib[1..lib.len()-1]);
                return off
            },
            _ => todo!()
        }
        TokenKind::Word => match token.value.as_str() {
            "extern" => return extrn(tokens, instructions, bindings),
            "fn" => return r#fn(tokens, instructions, bindings, false),
            "return" => return r#return(tokens, instructions, bindings),
            "let" => return var(tokens, instructions, bindings),

            "true" | "false" => instructions.push(Value::Bool(token.value == "true")),
            "if" => return r#if(tokens, instructions, bindings),
            "loop" => return r#loop(tokens, instructions, bindings),
            "break" => return r#break(tokens, instructions, bindings),
            "continue" => return r#continue(tokens, instructions, bindings),
            "unreachable" => { instructions.push(Value::Unreachable); return tokens.len() },

            "namespace" => return namespace(tokens, instructions, bindings),
            "use" => return r#use(tokens, bindings),
            "pub" => return r#pub(tokens, instructions, bindings),

            "struct" => return r#struct(tokens, instructions, bindings),
            "impl" => return r#impl(tokens, instructions, bindings),

            _ => return external_word(tokens, instructions, bindings)
        }
        _ => {}
    }

    return 1
}

pub fn value_loop<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>, v_off: usize, ignore: &[(Option<TokenKind>, Option<&str>)]) -> usize {
    let Some(tk) = tokens.get(v_off) else { return v_off };
    for ignore in ignore {
        match ignore {
            (Some(ignore_typ), None) => if ignore_typ == &tk.typ { return v_off },
            (None, Some(ignore_val)) => if ignore_val == &tk.value { return v_off },
            (Some(ignore_typ), Some(ignore_val)) => if ignore_typ == &tk.typ && ignore_val == &tk.value { return v_off },
            _ => {}
        }
    }
    let o = match tk {
        Token { typ: TokenKind::Arithmetic | TokenKind::Logical | TokenKind::Bitwise, .. } => {
            let y = expr(&tokens[v_off..], instructions, bindings);
            v_off + y
        }
        Token { typ: TokenKind::Assign, .. } => {
            let mut last = v_off;
            modify(&mut last, tokens, instructions, bindings);
            last
        }
        Token { typ: TokenKind::Brackets, .. } => {
            let mut off = v_off;
            next_load_address(&mut off, tokens, instructions, bindings, false);
            off
        }
        Token { typ: TokenKind::Special, value, .. } => {
            let mut off = v_off;
            match value.as_str() {
                ".*" => { next_deref(&mut off, tokens, instructions, bindings); },
                // "::" => {
                //     let Some(Value::Namespace(mut name)) = instructions.pop() else { error(&tokens[0], bindings, "Not a namespace") };
                //     let Some(Token { value: next, .. }) = next_token(&mut off, tokens, None, Some(TokenKind::Word)) else { error(&tokens[0], bindings, "Unexpected end of path") };
                //     name += "::";
                //     name += next;
                // },
                _ => {}
            }
            off
        }
        Token { typ: TokenKind::In, .. } => {
            let mut off = v_off;
            next_access_type_member(&mut off, tokens, instructions, bindings, false);
            off
        }
        Token { typ: TokenKind::Word, value, .. } => {
            if value == "as" {
                let y = typecast(&tokens[v_off..], instructions, bindings);
                v_off + y
            }
            else { v_off }
        }
        _ => v_off
    };
    if o > v_off { return value_loop(tokens, instructions, bindings, o, ignore) }
    return o
}

pub fn value_lookaround<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let ls = instructions.len();
    let x = value(tokens, instructions, bindings);
    if ls != instructions.len() {
        return value_loop(tokens, instructions, bindings, x, &[]);
    }
    return x
}

fn fn_pass<'a> (bindings: &mut Bindings<'a>) {
    for i in bindings.move_functions() {
        let ia = i.replace(Value::Unreachable);
        if let Value::PromisedFunction { name, args, body, ret, token } = ia {
            join_by_path(&name.clone(), bindings, |bindings| {
                let body = parse_fn(token, body, bindings, &args, ret.clone());
                i.replace(Value::Function { name, args, body, ret });
            });
        }
        else { i.replace(ia); }
    }
    if bindings.get_functions().len() > 0 { return fn_pass(bindings) }
}

fn type_pass<'a> (bindings: &mut Bindings<'a>) {
    for i in bindings.move_types() {
        let ia = i.replace(Value::Unreachable);
        if let Value::PromisedStruct { name, body } = ia {
            i.replace(parse_struct(name, body, bindings));
        }
        else { i.replace(ia); }
    }
    if bindings.get_types().len() > 0 { return type_pass(bindings) }
}