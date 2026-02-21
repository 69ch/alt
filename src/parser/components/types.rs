use std::collections::HashMap;

use insordmap::InsordMap;

use crate::{lexer::{Token, TokenKind}, parser::{SharedValue, Value, bindings::{Bind, Bindings}, components::binds::join_by_path, message::{error, error_range}, simpler::{ignore_separator, next_body, next_body_optional, next_token, next_type, next_value}, r#type::{Type, extract_type, parse_type}}};

const VALUE_REQUIRED_ERR: &str = "Typecast operation requires value on left side";

pub fn typecast<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let mut off = 1;
    let (to, toff) = parse_type(&tokens[off..], bindings).unwrap_or_else(|| error(&tokens[off], bindings, "Expected type"));
    off += toff;
    
    let value = instructions.pop().unwrap_or_else(|| error(&tokens[0], bindings, VALUE_REQUIRED_ERR));
    // dbg!(&value);
    // let from = default_type(&value);
    let from = if let Some(x) = extract_type(&value) { x }
    else if let Some(_) = to.check(&value) { to.clone() }
    else { todo!() };

    match (&from, &to) {
        (Type::I(_) | Type::U(_) | Type::Bool, Type::U(_) | Type::I(_) | Type::Bool)
        => instructions.push(Value::Typecast(Box::new(value), from, to)),
        _ => error_range(&tokens[1..off], bindings, "Typecast only available for primitive number-types")
    }

    off
}

pub fn r#struct<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let mut off = 1;

    let Token { value: name, .. } = next_token(&mut off, tokens, None, Some(TokenKind::Word))
    .unwrap_or_else(|| error(&tokens[0], bindings, "Struct must have name"));

    let global_name = bindings.global_name(&name);
    let Some(body) = next_body_optional(&mut off, tokens, ("{", "}")) else {
        instructions.push(Value::Struct { name: global_name, kv: InsordMap::new(), alignment: 1, size: 1 });
        bindings.global_insert(name, Bind::Type(None));
        return off
    };
    let sv = SharedValue::new(Value::PromisedStruct {
        name: global_name,
        body
    }.into());

    // maybe redo
    bindings.global_insert(name, Bind::Type(Some(sv.clone())));
    bindings.push_type(sv.clone());
    instructions.push(Value::SharedValue(sv));

    off
}

pub fn parse_struct<'a> (name: String, body: &'a [Token], bindings: &mut Bindings<'a>) -> Value<'a> {
    let mut kv = InsordMap::new();
    let mut off = 0;
    while let Some(Token { value: key, .. }) = next_token(&mut off, body, None, Some(TokenKind::Word)) {
        let point = off;
        // quite unefficient. todo: optimize somehow to get this operation (join_by_path) from loop
        if let Some(value) = join_by_path(&name, bindings, |bindings| next_type(&mut off, body, bindings)) {
            if let Type::Struct(l) = &value {
                let Bind::Type(x) = bindings.get(l).unwrap() else { unreachable!() };
                if let Some(x) = x {
                    validate_struct(x.clone(), bindings, &name, l, &body[point]);
                }
            }
            kv.insert(key.clone(), value);
            ignore_separator(&mut off, body);
        }
        else {
            error_range(&body[off-1..=(body.len()-1).min(off)], bindings, &format!("Expected type after field \"{key}\""))
        }
    }

    let alignment = kv.values().fold(1, |acc, x| acc.max(x.alignment(bindings)));
    let size = kv.values().fold(0, |acc, x| acc + (x.sizeof(bindings) + alignment as usize - 1) & !(alignment as usize - 1));
    Value::Struct { name, kv, alignment, size: size.max(1) }
}

fn validate_struct<'a> (x: SharedValue<'a>, bindings: &mut Bindings<'a>, from: &str, to: &str, tk: &'a Token) {
    let mut l = x.replace(Value::Unreachable);
    if let Value::PromisedStruct { name, body } = l {
        l = parse_struct(name, body, bindings);
    }

    if let Value::Struct { name: _stn, kv, .. } = &l {
        for typ in kv.values() {
            if let Type::Struct(name) = typ {
                if let Some(Bind::Type(Some(l))) = bindings.get(name) {
                    validate_struct(l.clone(), bindings, from, to, tk);
                }
            }
        }
    }
    else {
        error(tk, bindings, &format!("Recursive type has infinite size: {from} <-> {to}"));
    }

    // dbg!(&l);
    x.replace(l);
}

pub fn struct_init<'a> (l: SharedValue<'a>, body: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) {
    let x = l.replace(Value::Unreachable);
    if let Value::PromisedStruct { name, body } = x {
        l.replace(parse_struct(name, body, bindings));
    }
    else { l.replace(x); }
    if let Value::Struct { name, kv, .. } = &*l.borrow() {
        let mut off = 0;
        let mut init: HashMap<&'a String, Value> = HashMap::new();
        while let Some(Token { value: key, .. }) = next_token(&mut off, body, None, Some(TokenKind::Word)) {
            let kp = off;
            let Some(value) = next_value(&mut off, body, instructions, bindings) else { bindings.gentle_error(&body[off], "Expected value after key"); return };
            // if kv.get(key).is_some_and(|x| x.check(&value).is_none()) {
            if let Some(x) = kv.get(&key) {
                x.check_strict(&value, body, bindings);
            }
            else { bindings.gentle_error(&body[kp], &format!("There is no such field in type {name}")); return }
            if let Some(_) = init.insert(key, value) { todo!("warn about redefinition") }
            ignore_separator(&mut off, body);
        }
        // if init.keys()
        instructions.push(Value::StructInit(name.clone(), init));
    }
}