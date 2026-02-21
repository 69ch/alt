use std::mem;

use crate::{lexer::{Token, TokenKind}, parser::{Operation, Value, bindings::{Bind, Bindings}, message::error, simpler::{ignore_separator, next_body, next_body_optional, next_token, next_type}}, strip_alias_get};

use super::components::expr::{is_branch, is_cmp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    I(u32), U(u32),
    Bool,
    Array(Box<Type>, usize),
    /// (value, mutability)
    Ptr(Option<Box<Type>>, bool),
    Struct(String), // todo maybe eval alias instead of this
    Tuple(Vec<Type>),
    Fn(Vec<Type>, Box<Type>),
    Void, Noret, Guess
}

impl Type {
    pub fn check (&self, x: &Value) -> Option<()> {
        // dbg!(&self, &x);
        if self == &Type::Void || self == &Type::Noret { return None }
        if let Type::Ptr(_, false) = self {
            // dbg!(x);
            return match extract_type(x) {
                Some(Type::Ptr(_, _)) => {
                    // println!("PASSED");
                    Some(())
                },
                _ => {
                    // println!("FAILED");
                    None
                }
            }
        }
        if let Type::Ptr(_, true) = self {
            // dbg!(x);
            return match extract_type(x) {
                Some(Type::Ptr(_, true)) => Some(()),
                _ => None
            }
        }
        if self == &Type::Bool {
            if let Value::Expr(l) = x {
                let (_, _, op) = **l;
                
                if is_cmp(&op) || is_branch(&op) {
                    return Some(())
                }
            }
            else {
                match default_type(x) { Type::I(1) | Type::U(1) | Type::Bool => return Some(()), _ => {} }
            }
        }
        match x {
            Value::Expr(a) => {
                self.check(&a.0)?;
                self.check(&a.1)?;
                return Some(())
            }
            Value::Unary(a) => {
                self.check(&a.1)?;
                return Some(())
            }
            Value::Int(_) | Value::SInt(_) | Value::Bool(_) => return match self {
                Self::I(_) | Self::U(_) | Self::Bool => Some(()),
                _ => None
            },
            Value::Array(x) => {
                if let Type::Array(t, l) = self {
                    if *l != x.len() { return None }
                    for i in x {
                        t.check(i)?;
                    }
                    return Some(())
                }
            }
            Value::Tuple(x) => {
                if let Type::Tuple(y) = self {
                    if x.len() != y.len() { return None }
                    for i in 0..x.len() {
                        y[i].check(&x[i])?;
                    }
                    return Some(())
                }
            }
            _ => {
                if &extract_type(x)? == self { return Some(()) }
            }
        }
        None
    }

    pub fn check_strict (&self, x: &Value, tokens: &[Token], bindings: &mut Bindings) {
        let c = self.check(x);
        if c == None {
            let x = match x {
                Value::Tuple(v) => Type::Tuple(v.iter().map(|x| extract_type(x).unwrap_or(Type::Guess)).collect()),
                x => extract_type(x).unwrap_or(Type::Guess)
            };
            bindings.gentle_error_range(tokens, &format!("Mismatched types: expected '{}', got '{}'", self.display(), x.display()));
        }
    }

    pub fn display (&self) -> String {
        match self {
            Type::I(x) => return format!("i{x}"),
            Type::U(x) => return format!("u{x}"),
            Type::Ptr(Some(x), false) => return format!("&{}", x.display()),
            Type::Ptr(None, false) => "ptr",
            Type::Ptr(Some(x), true) => return format!("&mut {}", x.display()),
            Type::Ptr(None, true) => "ptrmut",
            Type::Bool => "bool",
            Type::Void => "void",
            Type::Array(t, l) => return format!("{}[{l}]", t.display()),
            Type::Tuple(t) => return format!("({})", t.iter().map(|x| x.display()).collect::<Vec<_>>().join(", ")),
            Type::Noret => "!",
            Type::Fn(args, ret) => return format!("fn ({}) {}", args.iter().map(|x| x.display()).collect::<Vec<String>>().join(", "), ret.display()),
            Type::Struct(x) => return format!("{x}"),
            Type::Guess => "_",
        }.into()
    }

    pub fn alignment (&self, bindings: &Bindings) -> u32 {
        match self {
            Type::I(x) | Type::U(x) => {
                let mut x = *x;
                x -= 1;
                x |= x >> 1;
                x |= x >> 2;
                x |= x >> 4;
                x |= x >> 8;
                x |= x >> 16;
                x += 1;
                (x / 8).max(1)
            },
            Type::Bool => 1,
            Type::Ptr(_, _) | Type::Fn(_, _) => (bindings.target_ptr_bits() / 8) as u32,
            Type::Tuple(x) => {
                let mut max = 0;
                for i in x {
                    let x = i.alignment(bindings);
                    if x > max { max = x }
                }
                max
            }
            Type::Array(x, _) => x.alignment(bindings),
            Type::Struct(x) => {
                let Some(Bind::Type(l)) = bindings.get(x) else { todo!() };
                if let Some(l) = l {
                    if let Value::Struct { alignment, .. } = &*l.borrow() {
                        return *alignment
                    }
                }
                0
            }
            x => todo!("{x:?}")
        }
    }
    pub fn sizeof (&self, bindings: &Bindings) -> usize {
        match self {
            Type::I(_) | Type::U(_) | Type::Ptr(_, _) | Type::Fn(_, _) => self.alignment(bindings) as usize,
            Type::Bool => 1,
            Type::Array(x, y) => x.sizeof(bindings) * y,
            Type::Tuple(x) => {
                let align = self.alignment(bindings) as usize;
                let mut y = 0;
                for i in x {
                    // (y + align - 1) & !(align - 1)
                    y += (i.sizeof(bindings) + align - 1) & !(align - 1);
                }
                y
            }
            Type::Struct(x) => {
                let Some(Bind::Type(l)) = bindings.get(x) else { todo!() };
                if let Some(l) = l {
                    if let Value::Struct { size, .. } = &*l.borrow() {
                        return *size
                    }
                    // else { todo!("inf size err") }
                }
                0
            }
            _ => todo!()
        }
    }
}

impl Default for Type {
    fn default () -> Self {
        Type::Void
    }
}

fn resolve_type<'a> (tokens: &'a [Token], bindings: &mut Bindings<'a>) -> Option<(Type, usize)> {
    let tg = tokens.get(0)?.value.as_str();
    if tg.starts_with("i") {
        if tg == "isize" { return Some((Type::I(bindings.target_ptr_bits() as u32), 1)) }
        if let Ok(x) = tg[1..].parse::<u32>() {
            return Some((Type::I(x), 1))
        }
    }
    else if tg.starts_with("u") {
        if tg == "usize" { return Some((Type::U(bindings.target_ptr_bits() as u32), 1)) }
        if let Ok(x) = tg[1..].parse::<u32>() {
            return Some((Type::U(x), 1))
        }
    }
    
    Some((match tokens.get(0)?.value.as_str() {
        "&" => {
            let mut last = 1;
            let mutable = next_token(&mut last, tokens, Some("mut"), Some(TokenKind::Word)).is_some();
            // let (t, j) = parse_type(peek(tokens, bindings, last..), bindings)?;
            let (t, j) = parse_type(tokens.get(last..)?, bindings)?;
            last += j;
            return Some((Type::Ptr(Some(Box::new(t)), mutable), last))
        }
        "&&" => {
            let mut last = 1;
            // let (t, j) = parse_type(peek(tokens, bindings, last..), bindings)?;
            let (t, j) = parse_type(tokens.get(last..)?, bindings)?;
            last += j;
            return Some((Type::Ptr(Some(Box::new(Type::Ptr(Some(Box::new(t)), false))), false), last))
        }
        "ptr" => Type::Ptr(None, false),
        "ptrmut" => Type::Ptr(None, true),
        "fn" => {
            let mut last = 1;
            let mut args = vec![];
            if let Some(body) = next_body_optional(&mut last, tokens, ("(", ")")) {
                let mut boff = 0;
                while let Some(x) = next_type(&mut boff, body, bindings) {
                    args.push(x);
                    ignore_separator(&mut boff, body);
                }
            }
            let ret = next_type(&mut last, tokens, bindings).unwrap_or(Type::Void);
            return Some((Type::Fn(args, Box::new(ret)), last))
        }
        "void" => Type::Void,
        "!" | "noret" => Type::Noret,
        "bool" => Type::Bool,
        "(" => return Some(tuple(tokens, bindings)),
        v => {
            // todo: recusrive parsing, instead of wave-like implemented for functions
            // remove shit from Bindings structure then
            let mut v = String::from(v);
            let mut off = 1;
            while let Some(Bind::Namespace(_)) = strip_alias_get!(v, bindings) {
                if let Some(_) = next_token(&mut off, tokens, Some("::"), Some(TokenKind::Special)) {
                    let Some(Token { value, .. }) = next_token(&mut off, tokens, None, Some(TokenKind::Word))
                    else { error(&tokens[off-1], bindings, "Invalid path") };
                    v += "::";
                    v += value;
                }
            }

            // it'll validate each time. better to look for other ways to implement that
            if let Some(Bind::Type(_l)) = strip_alias_get!(v, bindings) {
                // if let Some(l) = l {
                //     // validate_struct(l.clone(), bindings, &mut Vec::new());
                //     let l = l.clone();
                //     let mut x = l.replace(Value::Unreachable);
                //     if let Value::PromisedStruct { name, body } = x {
                //         x = parse_struct(name, body, bindings);
                //     }
                //     l.replace(x);
                // }

                // todo: need to separate parsing struct itself from parsing struct name

                return Some((Type::Struct(v), off))
            }
            return None
        }
    }, 1))
}

pub fn parse_type<'a> (tokens: &'a [Token], bindings: &mut Bindings<'a>) -> Option<(Type, usize)> {
    let (mut par_typ, j) = resolve_type(tokens, bindings)?;
    let mut last = j;
    if j > 1 { return Some((par_typ, last)) }
    while last < tokens.len() {
        match tokens[last].value.as_str() {
            "[" => {
                par_typ = Type::Array(
                    Box::new(par_typ),
                    tokens[last + 1].value.parse().unwrap_or_else(|_| error(&tokens[last+1], bindings, "Expected length of array"))
                );
                last += 3;

                continue
            }
            "<" => {
                todo!()
            }
            _ => break
        }
    }
    Some((par_typ, last))
}
fn tuple<'a> (tokens: &'a [Token], bindings: &mut Bindings<'a>) -> (Type, usize) {
    let mut off = 0;
    let mut types = vec![];
    let body = next_body(&mut off, tokens, bindings, ("(", ")"));
    let mut boff = 0;
    while boff < body.len() {
        types.push(next_type(&mut boff, body, bindings).unwrap_or_else(|| error(&body[boff], bindings, "Not a type")));
        // let _separator = next_token(&mut boff, body, None, Some(TokenKind::Punctuation));
        ignore_separator(&mut boff, body);
    }
    return (Type::Tuple(types), off)
}

pub fn default_type_expr<'a> (lhs: &Value<'a>, rhs: &Value<'a>, op: Operation) -> Type {
    if is_cmp(&op) { return Type::Bool }
    return extract_types(&[lhs, rhs]).unwrap_or(Type::Void)
}

pub fn default_type (x: &Value) -> Type {
    match x {
        Value::Int(_) => Type::I(32),
        Value::Bool(_) => Type::Bool,
        Value::Expr(l) => default_type_expr(&l.0, &l.1, l.2),
        Value::String(x) => Type::Array(Box::new(Type::I(8)), x.len()),
        Value::Array(t) => {
            let Some(x) = extract_types_move(&t) else { return Type::Void };
            Type::Array(Box::new(x), t.len())
        },
        Value::Ptr(t, m) => Type::Ptr(Some(Box::new(default_type(t))), *m),
        Value::Tuple(x) => Type::Tuple(x.iter().map(|x| default_type(x)).collect()),
        _ => { extract_type(x).unwrap_or_else(|| {dbg!(x); panic!()}) }
    }
}

pub fn pure_type (x: &Type) -> &Type {
    match x {
        Type::Ptr(Some(t), _) => &*t,
        _ => x
    }
}

pub fn penetrate_type (x: Type, u: usize) -> Type {
    match x {
        Type::Array(t, _) => *t,
        Type::Ptr(Some(t), _) => *t,
        Type::Tuple(mut x) => {
            // dbg!(&x, u);
            mem::take(&mut x[u])
        },
        _ => x
    }
}

pub fn extract_type (x: &Value) -> Option<Type> {
    Some(match x {
        Value::Array(x) => {
            let mut m = None;
            let len = x.len();
            for i in x {
                let l = m;
                m = Some(extract_type(i)?);
                if m != l && l != None { return None }
            }
            Type::Array(Box::new(m?), len)
        },
        // Value::Tuple(x) => {
        //     Type::Tuple(x.iter().map(|x| default_type(x)).collect())
        // },
        // Value::FunctionCall(_, t, _) | 
        Value::Get(_, t) | Value::LoadFromPtr(_, t) => t.clone(),
        Value::LoadAddress(_, _, t, mutable) => Type::Ptr(Some(Box::new(penetrate_type(t.clone(), 0))), *mutable),
        Value::Ptr(to, mutable) => {
            let to = if let Some(to) = extract_type(to) { Some(Box::new(to)) } else { None };
            Type::Ptr(to, *mutable)
        },
        Value::InitVar(_, t, _) => t.clone(),
        Value::Typecast(_, _, t) => t.clone(),
        Value::FunctionPointer(_, ret, args) => Type::Fn(args.clone(), Box::new(ret.clone())),
        Value::AnonFunction { args, body: _, ret } => Type::Fn(args.iter().map(|x| x.typ.clone()).collect(), Box::new(ret.clone())),
        Value::Call(v, _) => {
            let Type::Fn(_, ret) = extract_type(v)? else { return None };
            *ret
        },
        Value::StructInit(name, _) => Type::Struct(name.clone()),
        _ => return None
    })
}

macro_rules! extrtyps {
    ($slice:expr) => {
        let slice = $slice;
        let mut strong_type = None;
        for i in slice {
            if None == strong_type {
                strong_type = extract_type(i);
            }
            else {
                break
            }
        }
        let strong_type = strong_type.unwrap_or_else(|| default_type(&slice[0]));
        for i in slice {
            strong_type.check(i)?;
        }
        return Some(strong_type);
    };
}

pub fn extract_types<'a> (slice: &[&Value<'a>]) -> Option<Type> {
    extrtyps!(slice);
}
pub fn extract_types_move<'a> (slice: &[Value<'a>]) -> Option<Type> {
    extrtyps!(slice);
}

// pub fn produces_value (x: &Value) -> bool {
//     match x {
//         Value::String(_) | Value::Int(_) | Value::SInt(_) | Value::Float(_) | Value::Expr(_) | Value::Unary(_) |
//         Value::Array(_) |
//         Value::Get(_, _) | Value::Ptr(_, _) |
//         Value::LoadFromPtr(_, _) |
//         Value::LoadAddress(_, _, _, _) => true,
//         _ => false
//     }
// }