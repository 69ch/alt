use crate::{lexer::Token, parser::{Value, bindings::{Bind, Bindings}, message::error_range, r#type::Type}};
const MUT_ADDR_MESSAGE: &str = "Cannot take mutable address to immutable data";

pub fn take_pointer<'a> (erange: &'a [Token], bindings: &mut Bindings<'a>, mutable: bool, val: Value<'a>) -> Value<'a> {
    if let Value::Get(name, _) = val {
        if mutable {
            let Bind::Let(_, true) = bindings.get(name).unwrap() else { error_range(&erange, bindings, MUT_ADDR_MESSAGE) };
            return Value::Ptr(Box::new(val), true)
        }
        return Value::Ptr(Box::new(val), false)
    }
    else if let Value::LoadFromPtr(x, _) = val {
        if mutable { Type::Ptr(None, true).check(&x).unwrap_or_else(|| error_range(&erange, bindings, MUT_ADDR_MESSAGE)); }
        else {
            match *x {
                Value::LoadAddress(a, b, c, true) => {
                    return Value::LoadAddress(a, b, c, false)
                }
                Value::Ptr(a, true) => {
                    return Value::Ptr(a, false)
                },
                _ => {}
            }
        }
        return *x
    }
    else {
        return if mutable { Value::Ptr(Box::new(val), true) } else { Value::Ptr(Box::new(val), false) }
    }
}