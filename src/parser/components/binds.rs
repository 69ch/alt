use std::mem;

use crate::{lexer::{Token, TokenKind}, nvalue, parser::{Operation, Value, bindings::{Bind, Bindings}, components::{r#fn::r#fn, types::struct_init}, message::{assert, assert_range, error, error_range}, parse_inplace, parse_np, simpler::{ignore_separator, next_access_type_member, next_body, next_body_optional, next_call, next_deref, next_load_address, next_mutable_flag, next_name, next_token, next_type, next_uniform_call, next_value}, r#type::{Type, default_type, extract_type, penetrate_type, pure_type}}};

pub const ASSIGN_NOT_EXPECTED: &str = "Assign operation not expected here";

fn variable_look <'a> (off: &mut usize, tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>, mutable: bool) {
    // fix here
    next_load_address(off, tokens, instructions, bindings, mutable);
    next_access_type_member(off, tokens, instructions, bindings, mutable);
    next_call(off, tokens, instructions, bindings);
    if let Some(_) = next_deref(off, tokens, instructions, bindings) {}
    else if mutable { modify(off, tokens, instructions, bindings); }
}

pub fn external_word<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let mut name = tokens[0].value.clone();
    let mut off = 1;

    loop {
    let Some(x) = bindings.get(&name) else { bindings.gentle_error(&tokens[off-1], "Unknown keyword"); return off };
    match x {
        Bind::Function(argt, typ, l) => {
            let (typ, argt) = (typ.clone(), argt.clone());
            instructions.push(Value::FunctionPointer(name, typ, argt));
            if let Some(l) = l {
                bindings.push_function(l.clone());
            }
            next_call(&mut off, tokens, instructions, bindings);
        }
        Bind::Let(typ, mutable) => {
            instructions.push(Value::Get(&tokens[0].value, typ.clone()));
            variable_look(&mut off, tokens, instructions, bindings, *mutable);
        }
        // Bind::Value(x) => { instructions.push(x.clone()); }
        Bind::Namespace(_) => {
            if let Some(_) = next_token(&mut off, tokens, Some("::"), Some(TokenKind::Special)) {
                let Some(b) = next_token(&mut off, tokens, None, Some(TokenKind::Word)) else {
                    error(&tokens[off-1], bindings, "Invalid path")
                };
                name += "::";
                name += &b.value;
                continue
            }
            else { instructions.push(Value::Namespace(name)); break }
        }
        Bind::Alias(n) => {
            name = n.into();
            continue
        }
        // Bind::Type(_) => todo!("types in code blocks"),
        Bind::Type(Some(l)) => {
            let body = next_body(&mut off, tokens, bindings, ("{", "}"));
            struct_init(l.clone(), body, instructions, bindings);
        },
        Bind::Type(None) => todo!("opaque types"),
        // Bind::Public(x) => { return  }
        Bind::Label => bindings.gentle_error(&tokens[0], "You should use label names for 'break' and 'continue'"),
        Bind::Mark => panic!("Unexpected behaviour in front-end")
    }
    break
    }

    return off
}

// pub fn reserve_local<'a> (bindings: &mut Bindings<'a>) {
//     let Some(Bind::Counter(x)) = bindings.get_mut(RESERVED_LOCALS) else { panic!() };
//     *x += 1;
// }

pub fn var<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let mut off = 1;
    let off_name = off;
    let mutable = next_mutable_flag(&mut off, tokens);
    let name = &next_token(&mut off, tokens, None, Some(TokenKind::Word)).unwrap_or_else(|| error(&tokens[0], bindings, "Expected name of variable")).value;
    let mut typ = next_type(&mut off, tokens, bindings).unwrap_or(Type::Guess);
    // reserve_local(bindings);
    if let Some(_) = next_token(&mut off, tokens, Some("="), Some(TokenKind::Assign)) {
        let val = nvalue!(&mut off, tokens, instructions, bindings, "Expected value of variable", &tokens[off-1], off);
        
        if Type::Guess == typ { typ = default_type(&val); }
        else { typ.check_strict(&val, &tokens[off_name..off], bindings); }
        assert_range(typ != Type::Void, &tokens[1..off], bindings, "Variable with void-type");

        let val = Box::new(val);
        bindings.insert(name, Bind::Let(typ.clone(), mutable));
        // dbg!(typ.alignment(bindings), typ.sizeof(bindings));
        instructions.push(Value::InitVar(&name, typ, Some(val)));
        return off
    }
    assert(typ != Type::Guess, &tokens[off_name], bindings, "Must provide type at this point");
    assert_range(typ != Type::Void, &tokens[1..off], bindings, "Variable with void-type");
    bindings.insert(name, Bind::Let(typ.clone(), mutable));
    instructions.push(Value::InitVar(&name, typ, None));
    return off
}

fn assign<'a> (d: Value<'a>, value: Value<'a>, symbol: &'a str) -> Value<'a> {
    macro_rules! expr {
        ($op:expr) => {
            Value::Expr(Box::new((d.clone(), value, $op)))
        };
    }
    match symbol {
        "=" => value,
        "+=" => expr!(Operation::Add),
        "-=" => expr!(Operation::Sub),
        "*=" => expr!(Operation::Mul),
        "/=" => expr!(Operation::Div),
        "%=" => expr!(Operation::Rem),
        _ => todo!()
    }
}

pub fn modify<'a> (off: &mut usize, tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> bool {
    let s = *off;
    if let Some(Token { value: ass, .. }) = next_token(off, tokens, None, Some(TokenKind::Assign)) {
        let val = instructions.pop().unwrap();
        dbg!(&val);
        if let Value::Get(name, typ) = val {
            let v = assign(Value::Get(name, typ.clone()), nvalue!(off, tokens, instructions, bindings, "Expected value", false), ass);
            typ.check_strict(&v, &tokens[..*off], bindings);
            instructions.push(Value::ModifyVar(name, typ, Box::new(v)));
            return true
        }
        else if let Value::LoadFromPtr(x, typ) = val {
            let v = assign(Value::LoadFromPtr(x.clone(), typ.clone()), nvalue!(off, tokens, instructions, bindings, "Expected value", false), ass);
            typ.check_strict(&v, &tokens[..*off], bindings);
            match *x {
                Value::Get(_, Type::Ptr(_, true)) | Value::LoadAddress(_, _, Type::Ptr(_, true), _) | Value::LoadAddress(_, _, _, true) => {
                    instructions.push(Value::ModifyByPointer(Box::new((*x, typ, v))));
                    return true
                }
                _ => {}
            }
        }
        bindings.gentle_error(&tokens[s], ASSIGN_NOT_EXPECTED);
    }
    false
}

pub fn load_address<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>, mutable: bool) -> usize {
    let mut from = instructions.pop().unwrap();
    let mut off = 0;
    let body = next_body(&mut off, tokens, bindings, ("[", "]"));
    let Some(index) = next_value(&mut 0, body, instructions, bindings) else { bindings.gentle_error_range(&tokens[0..off], "Expected offset"); return off };
    Type::U(bindings.target_ptr_bits() as u32).check_strict(&index, body, bindings);
    let typ = extract_type(&from).unwrap_or_else(|| error(&tokens[0], bindings, "Unindexable value"));
    let mut u = 0;
    match typ {
        Type::Array(_, _) => {}
        Type::Ptr(Some(to), mutable) => {
            instructions.push(Value::LoadFromPtr(Box::new(from), *to.clone()));
            return load_address(tokens, instructions, bindings, mutable)
        }
        Type::Tuple(_) => {
            if let Value::Int(x) = index {
                u = x
            }
            else { bindings.gentle_error_range(&body, "To load address of structure's field, you need to provide clear number"); return off }
        },
        _ => error(&tokens[0], bindings, "Unindexable value")
        // _ => { bindings.gentle_error(&tokens[0], "Unindexable value"); return off },
    }
    let subtyp = penetrate_type(extract_type(&from).unwrap(), u);
    if let Value::LoadFromPtr(x, _) = from { from = *x }
    
    instructions.push(Value::LoadFromPtr(Box::new(Value::LoadAddress(Box::new(from), Box::new(index), typ, mutable)), subtyp));
    // dbg!(instructions.last());

    off
}

pub fn access_type_member<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>, mutable: bool) -> usize {
    let mut off = 1;
    let from = instructions.pop().unwrap();
    // dbg!(&from);
    // todo
    let typ = extract_type(&from).unwrap_or_else(|| error(&tokens[0], bindings, "No valid value for this operation"));
    let Some(Token { value: field, .. }) = next_token(&mut off, tokens, None, Some(TokenKind::Word)) else { error(&tokens[0], bindings, "Don't you think that this dot here kinda passive-aggressive?") };

    // TODO UNIFORM FUNCTION CALL SYNTAX SOMEHOW
    if let Some(_) = next_body_optional(&mut 0, &tokens[off..], ("(", ")")) {
        // dbg!("bebra");
        if let Type::Struct(name) = pure_type(&typ) {
            let name = format!("{name}::{field}");
            // dbg!(&name);
            if let Some(Bind::Function(args, ret, l)) = bindings.get(&name) {
                instructions.push(Value::FunctionPointer(name, ret.clone(), args.clone()));
                // TODO ORGANIZE
                if let Some(l) = l { bindings.push_function(l.clone()); }
                next_uniform_call(&mut off, from, tokens, instructions, bindings);
                return off
                // next_call(&mut off, tokens, instructions, bindings);
            }
        }
        if let Some(Bind::Function(args, ret, l)) = bindings.get(field) {
            instructions.push(Value::FunctionPointer(field.clone(), ret.clone(), args.clone()));
            // TODO ORGANIZE
            if let Some(l) = l { bindings.push_function(l.clone()); }
            next_uniform_call(&mut off, from, tokens, instructions, bindings);
            return off
            // next_call(&mut off, tokens, instructions, bindings);
        }
    }
    match &typ {
        Type::Struct(name) => {
            if let Some(Bind::Type(Some(l))) = bindings.get(&name) {
                let Value::Struct { kv, .. } = &*l.borrow() else { panic!() };
                let Some((subtyp, index)) = kv.get_w_p(field) else { error(&tokens[off-1], bindings, &format!("There is no '{field}' in type '{}'", typ.display())) };
                // dbg!(mutable);
                instructions.push(Value::LoadFromPtr(Box::new(Value::LoadAddress(Box::new(from), Box::new(Value::Int(index)), typ, mutable)), subtyp.clone()));
            }
            else { error(&tokens[0], bindings, "Unindexable value") }
        }
        Type::Ptr(Some(to), mutable) => {
            instructions.push(Value::LoadFromPtr(Box::new(from), *to.clone()));
            return access_type_member(tokens, instructions, bindings, *mutable)
        }
        _ => error(&tokens[0], bindings, "Unindexable value")
    }
    off
}

pub fn deref<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let mut off = 1;
    assert(Type::Ptr(None, false).check(instructions.last().unwrap()).is_some(), &tokens[0], bindings, "Expected pointer to dereference");
    let val = instructions.pop().unwrap();
    
    let typ = if let Some(x) = extract_type(&val) { penetrate_type(x, 0) }
    else { error_range(&tokens[..off], bindings, "Cannot get value behind address") };
    
    instructions.push(Value::LoadFromPtr(Box::new(val), typ));
    modify(&mut off, tokens, instructions, bindings);
    
    off
}


pub fn namespace<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let mut off = 1;

    let Some(Token { value: name, .. }) = next_token(&mut off, tokens, None, Some(TokenKind::Word)) else { error(&tokens[0], bindings, "Namespace must have name, duh") };
    let body = next_body(&mut off, tokens, bindings, ("{", "}"));

    bindings.join_scope();
    bindings.push_global_prefix(name.to_string());
    parse_np(body, instructions, bindings);
    // let scope = bindings.pop_scope().into_keys();
    // let namespace= Bind::Namespace(scope.filter(|x| {
    //     bindings.get(&bindings.global_name(x)).is_some()
    // }).collect());
    let scope = bindings.pop_scope();
    let namespace= Bind::Namespace(scope);
    bindings.pop_global_prefix();
    bindings.global_insert(name, namespace);

    off
}

fn resolve_alias (mut n: String, bindings: &Bindings) -> String {
    while let Some(Bind::Alias(x)) = bindings.get(&n) {
        n = x.clone();
    }
    n
}

#[derive(Debug)]
enum NSPath { Group(Vec<Vec<NSPath>>), Fragment(String), All }

fn nspath<'a> (off: &mut usize, tokens: &'a [Token], bindings: &Bindings<'a>) -> Option<Vec<NSPath>> {
    let mut path = vec![NSPath::Fragment(next_token(off, tokens, None, Some(TokenKind::Word))?.value.to_string())];
    while let Some(_) = next_token(off, tokens, Some("::"), Some(TokenKind::Special)) {
        if let Some(sub) = next_token(off, tokens, None, Some(TokenKind::Word)) {
            path.push(NSPath::Fragment(sub.value.to_string()));
        }
        else if let Some(_) = next_token(off, tokens, Some("*"), Some(TokenKind::Arithmetic)) {
            path.push(NSPath::All);
            break
        }
        else if let Some(body) = next_body_optional(off, tokens, ("{", "}")) {
            let mut soff = 0;
            let mut v = vec![];
            while let Some(b) = nspath(&mut soff, body, bindings) {
                v.push(b);
                ignore_separator(&mut soff, body);
            }
            path.push(NSPath::Group(v));
            break
        }
    }
    Some(path)
}

fn sign_path (bindings: &mut Bindings, mut path: Vec<NSPath>, mut root: String) {
    let access = path.pop().unwrap();
    let mut path = path.into_iter();
    while let Some(NSPath::Fragment(x)) = path.next() {
        // may be problem in future. there might be problems with public "use"
        if root.is_empty() { root += &resolve_alias(x, bindings) }
        else { root += &x; }
        root += "::";
    }
    match access {
        NSPath::Fragment(x) => {
            // dbg!(&x, format!("{root}{x}"));
            bindings.insert(&x, Bind::Alias(format!("{root}{x}")));
        },
        NSPath::Group(g) => {
            for i in g {
                sign_path(bindings, i, root.clone());
            }
        }
        NSPath::All => {
            let Some(Bind::Namespace(n)) = bindings.get(&root[..root.len()-2]) else { todo!() };
            // let n = n.keys().cloned().filter(|x| bindings.get(&bindings.global_name(x)).is_some()).collect::<Vec<String>>();
            // dbg!(&root);
            let n = n.keys().cloned().filter(|x| bindings.get(&format!("{root}{x}")).is_some()).collect::<Vec<String>>();
            for i in n {
                bindings.insert(&i, Bind::Alias(format!("{root}{i}")));
            }
        }
    }
}

/// WARN: for now, this is implemented for namespaces only. Need to think about structures, especially cases with generics involved
pub fn r#use<'a> (tokens: &'a [Token], bindings: &mut Bindings<'a>) -> usize {
    let mut off = 1;

    let Some(path) = nspath(&mut off, tokens, bindings) else { todo!() };
    sign_path(bindings, path, String::new());

    off
}


pub fn r#pub<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    if let Some(Token { typ: TokenKind::Word, value, .. }) = tokens.get(1) {
        match value.as_str() {
            "fn" => return r#fn(&tokens[1..], instructions, bindings, true) + 1,
            _ => {}
        }
    }
    error(&tokens[0], bindings, "Expected valid instruction after 'pub' (e.g. function declaration)");
}

// somehow fix (idk i fixed it or not, i guess i am because it works fine, but comment is not deleted soooo)
pub fn join_by_path<'a, T> (name: &str, bindings: &mut Bindings<'a>, f: impl FnOnce (&mut Bindings<'a>) -> T) -> T {
    let path = name.split("::").collect::<Vec<&str>>();
    if path.len() > 1 {
        let mut j = 0;
        while let Some(Bind::Namespace(scope)) = bindings.get_mut(path[j]) {
            // dbg!(&path[j]);
            let scope = mem::take(scope);
            bindings.push_scope(scope);
            // if j >= path.len() - 2 { break }
            if j == path.len() - 1 { break }
            j += 1;
        }
        let rv = f(bindings);
        while j != 0 {
            let scope = bindings.pop_scope();
            // dbg!(&path[0..j]);
            *bindings.get_mut_at(0, &path[0..j].join("::")).unwrap() = Bind::Namespace(scope);
            j -= 1;
        }
        rv
    }
    else { f(bindings) }
}

pub fn r#impl<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let mut off = 1;
    let name = next_name(&mut off, tokens);
    // dbg!(&name);
    let body = next_body(&mut off, tokens, bindings, ("{", "}"));

    bindings.join_scope();
    bindings.push_global_prefix(name);
    parse_inplace(body, instructions, bindings);
    bindings.pop_global_prefix();
    bindings.leave_scope();
    // dbg!(instructions);

    // todo!();

    off
}