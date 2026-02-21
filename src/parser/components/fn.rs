use crate::{lexer::{Token, TokenKind}, parser::{bindings::{Bind, Bindings, Context, MARK_BARRIER}, components::{args::Arg, control_flow::get_return}, message::error, parse_inplace, simpler::{next_args, next_body, next_body_optional, next_token, next_type}, r#type::Type, value_lookaround, SharedValue, Value}};

pub fn r#fn<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>, public: bool) -> usize {    
    let mut off = 1;

    if bindings.get_context_noval(&Context::Fn(Type::Void)).is_some() {
        // error(&tokens[0], bindings, "Function definition not allowed in other functions");
        let args = next_args(&mut off, tokens, bindings);
        let ret = next_type(&mut off, tokens, bindings).unwrap_or(Type::Void);
        let body = next_body(&mut off, tokens, bindings, ("{", "}"));
        let body = parse_fn(&tokens[0], body, bindings, &args, ret.clone());
        instructions.push(Value::AnonFunction {
            args,
            body,
            ret
        });
        return off
    }
    // now owned string for namespaces (maybe, lol. still thinking about implementation)
    let lname = &next_token(&mut off, tokens, None, Some(TokenKind::Word)).unwrap_or_else(|| error(&tokens[0], bindings, "Expected name of function")).value;
    // if bindings.get_at_current(lname).is_some() { error(&tokens[1], bindings, "Function name must be unique in current namespace") }
    let name = bindings.global_name(lname);
    dbg!(&name);
    if bindings.get_at(0, &name).is_some() { error(&tokens[1], bindings, "Function name must be unique in current namespace") }

    let args = next_args(&mut off, tokens, bindings);
    let ret = next_type(&mut off, tokens, bindings).unwrap_or(Type::Void);
    // let body = next_body(&mut off, tokens, bindings, ("{", "}"));
    let Some(body) = next_body_optional(&mut off, tokens, ("{", "}")) else {
        error(&tokens[off], bindings, "Expected function body, argument list enclosed in parentheses, or return type — got none of these")
    };
    // bindings.insert(name, Bind::Function(ret.clone(), args.iter().map(|x| x.typ.clone()).collect(), instructions.len()));
    
    // if name == "main" { bindings.push_function(instructions.len()); }
    // instructions.push(Value::PromisedFunction {
    //     name,
    //     args,
    //     body,
    //     ret,
    //     token: &tokens[0]
    // });
    // let name = format!("{}{name}", bindings.get_global_prefix());
    let argt = args.iter().map(|x| x.typ.clone()).collect::<Vec<Type>>();
    let sv = SharedValue::new(Value::PromisedFunction {
        name: name.clone(),
        args,
        body,
        ret: ret.clone(),
        token: &tokens[0]
    }.into());
    if public { bindings.global_insert(lname, Bind::Function(argt, ret, Some(sv.clone()))); }
    else {
        bindings.insert(&name, Bind::Function(argt, ret, Some(sv.clone())));
        if lname != &name { bindings.insert(lname, Bind::Alias(name.clone())); }
    }
    if name == "main" { bindings.push_function(sv.clone()); }
    // bindings.push_function(sv.clone());

    // bindings.global_insert(lname, Bind::Function(argt, ret));

    instructions.push(Value::SharedValue(sv));

    return off
}

pub fn extrn<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    if bindings.get_context_noval(&Context::Fn(Type::Void)).is_some() {        
        error(&tokens[0], bindings, "'extern' is not allowed in other functions");
    }

    let extrn_name = |off: &mut usize, tokens: &'a [Token], bindings: &mut Bindings<'a>, instructions: &mut Vec<Value<'a>>, argt: Vec<Type>, ret: Type| {
        let name = &next_token(off, tokens, None, Some(TokenKind::Word)).unwrap_or_else(|| error(&tokens[*off], bindings, "Expected function name")).value;
        if let Some(Bind::Function(_, _, _)) = bindings.root_insert(name, Bind::Function(argt.clone(), ret.clone(), None)) {
            error(&tokens[*off-1], bindings, "Function name in 'extern' must be unique in root namespace")
        }
        instructions.push(Value::Extern(name, argt, ret));
    };
    
    let mut off = 1;

    if let Some(body) = next_body_optional(&mut off, tokens, ("{", "}")) {
        let mut soff = 0;
        while let Some(Type::Fn(argt, ret)) = next_type(&mut soff, body, bindings) {
            extrn_name(&mut soff, body, bindings, instructions, argt, *ret);
        }
    }
    else {
        let Some(Type::Fn(argt, ret)) = next_type(&mut off, tokens, bindings) else { error(&tokens[off], bindings, "There must be function type: 'fn (args) ret'") };
        extrn_name(&mut off, tokens, bindings, instructions, argt, *ret);
    }

    return off
}

pub fn r#return<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let j = if tokens[1..].len() > 0 { value_lookaround(&tokens[1..], instructions, bindings) } else { 0 };
    let mut value = if j > 0 { instructions.pop() } else { None };
    
    let Some(Context::Fn(t)) = bindings.get_context_noval(&Context::Fn(Type::Void)) else {
        error(&tokens[0], bindings, "Out-of-function return")
    };
    let t= t.clone();
    
    if let Some(ref mut value) = value {
        let Some(tt) = tokens.get(1..=j) else { bindings.gentle_error(&tokens[1], "No valid value provided"); return 1 };
        t.check_strict(value, tt, bindings);
    }
    else if t != Type::Void { error(&tokens[0], bindings, &format!("Expected `{}`, got nothing", t.display())) }
    
    instructions.push(Value::Return(Box::new((value, t.clone()))));
    return tokens.len()
}

pub fn parse_fn<'a> (token: &Token, body: &'a [Token], bindings: &mut Bindings<'a>, args: &[Arg<'a>], ret: Type) -> Vec<Value<'a>> {
    // let mut fn_instructions = vec![Value::ReservedLocals(0)];
    let mut fn_instructions = vec![];
    bindings.context_scope(Context::Fn(ret.clone()), |bindings| {
        bindings.insert(MARK_BARRIER, Bind::Mark);
        args.iter().for_each(|x| { bindings.insert(x.name, Bind::Let(x.typ.clone(), x.mutable)); });
        // bindings.insert(RESERVED_LOCALS, Bind::Counter(0));
        parse_inplace(body, &mut fn_instructions, bindings);
        // let Bind::Counter(x) = bindings.get(RESERVED_LOCALS).unwrap() else { panic!() };
        // fn_instructions[0] = Value::ReservedLocals(*x);
        // dbg!(&fn_instructions[0]);
        if get_return(&fn_instructions) {}
        else if Type::Void == ret { fn_instructions.push(Value::Return(Box::new((None, Type::Void)))); }
        else if Type::Noret == ret { fn_instructions.push(Value::Unreachable); }
        else { bindings.gentle_error(&token, "Function doesn't have 'return' instruction that will definitely happen") }
    });
    fn_instructions
}