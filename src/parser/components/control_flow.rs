use crate::{lexer::{Token, TokenKind}, nvalue, parser::{bindings::{Bind, Bindings, Context}, message::error, parse, simpler::{next_body, next_token, next_value}, r#type::Type, Value}};

pub fn get_return (body: &Vec<Value>) -> bool {
    if let Some(Value::Return(_)) | Some(Value::ReturnMark) | Some(Value::Unreachable) = body.last() {
        true
    }
    else {
        false
    }
}

pub fn r#if<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let mut off = 1;
    
    let condition = Box::new(nvalue!(&mut off, tokens, instructions, bindings, "Expected expression"));
    Type::Bool.check_strict(&condition, &tokens[1..off], bindings);
    
    let body = next_body(&mut off, tokens, bindings, ("{", "}"));
    let mut else_then = Box::new(None);

    let mut it_returns = false;
    // if let Some(Token { value, .. }) = tokens.get(off) {
    // if value == "else" {
    //     let (ej, eret) = r#else(&tokens[off..], instructions, bindings);
    //     off += ej;
    //     it_returns = eret;
    //     else_then = Box::new(instructions.pop());
    // }
    // }
    if let Some(_) = next_token(&mut off, tokens, Some("else"), Some(TokenKind::Word)) {
        let (ej, eret) = r#else(&tokens[off-1..], instructions, bindings);
        off += ej;
        it_returns = eret;
        else_then = Box::new(instructions.pop());
    }

    bindings.context_scope(Context::If, |bindings| {
        let body = parse(body, bindings);
        it_returns = it_returns && get_return(&body);
        instructions.push(Value::If { condition, body, else_then });
    });
    
    if it_returns { instructions.push(Value::ReturnMark); return tokens.len() }
    return off
}

pub fn r#else<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> (usize, bool) {
    let mut off = 1;
    if let Some(_) = next_token(&mut off, tokens, Some("if"), Some(TokenKind::Word)) {
        off += r#if(&tokens[1..], instructions, bindings);
        let last = instructions.last();
        if let Some(Value::If { .. }) = last {
            return (off, false)
        }
        else if let Some(Value::ReturnMark) = last {
            instructions.pop();
            return (off, true)
        }
        else { unreachable!() }
    }
    else {
        let body = next_body(&mut off, tokens, bindings, ("{", "}"));
        bindings.join_scope();
        let body = parse(body, bindings);
        let r = (off, get_return(&body));
        instructions.push(Value::Else(body));
        bindings.leave_scope();
        r
    }
}

pub fn r#loop<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let mut off = 1;
    let body = next_body(&mut off, tokens, bindings, ("{", "}"));
    let label = if let Some(_) = next_token(&mut off, tokens, None, Some(TokenKind::LabelSymbol)) {
        let Some(Token { value: label, .. }) = next_token(&mut off, tokens, None, Some(TokenKind::Word)) else { error(&tokens[off], bindings, "Expected name of label") };
        bindings.insert(label, Bind::Label);
        Some(label.as_str())
    } else { None };
    
    bindings.context_scope(Context::Loop, |bindings| {
        let body = parse(body, bindings);
        instructions.push(Value::Loop(body, label));
    });
    
    return off
}


pub fn r#break<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    if let Some(Token { typ: TokenKind::Word, value: name, .. }) = tokens.get(1) {
        if let Some(Bind::Label) = bindings.get(name) {
            instructions.push(Value::Break(Some(name)));
        }
        else {
            error(&tokens[1], bindings, "There's no label with this name")
        }
    }
    else if let Some(_) = bindings.get_context_noval(&Context::Loop) {
        instructions.push(Value::Break(None));
    }
    else {
        error(&tokens[0], bindings, "Using 'break' is allowed only in loops")
    }
    return tokens.len()
}

pub fn r#continue<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    if let Some(Token { typ: TokenKind::Word, value: name, .. }) = tokens.get(1) {
        if let Some(Bind::Label) = bindings.get(name) {
            instructions.push(Value::Continue(Some(name)));
        }
        else {
            error(&tokens[1], bindings, "There's no label with this name")
        }
    }
    else if let Some(_) = bindings.get_context_noval(&Context::Loop) {
        instructions.push(Value::Continue(None));
    }
    else {
        error(&tokens[0], bindings, "Using 'continue' is allowed only in loops")
    }
    return tokens.len()
}