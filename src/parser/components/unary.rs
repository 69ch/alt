use crate::{lexer::Token, novalue, parser::{Value, bindings::Bindings, components::pointer::take_pointer, message::error, simpler::{next_mutable_flag, next_one_value}}};

use super::{expr::get_op};

pub fn unary<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let mut off = 1;
    let token = &tokens[0];
    match tokens[0].value.as_str() {
        "-" => {
            // let val = next_one_value(&mut off, tokens, instructions, bindings, "Expected value for NEG operation");
            // let Some(val) = next_one_value(&mut off, tokens, instructions, bindings) else { bindings.gentle_error(&tokens[0], "Expected value for NEG operation"); return off };
            let val = novalue!(&mut off, tokens, instructions, bindings, "Expected value for NEG operation", 0, off);
            if let Value::Int(x) = val { instructions.push(Value::SInt(-(x as isize))); return off }
            instructions.push(Value::Unary(Box::new((get_op(token).unwrap(), val))));
        }
        "!" => {
            // let val = next_one_value(&mut off, tokens, instructions, bindings, "Expected value for NOT operation");
            let val = novalue!(&mut off, tokens, instructions, bindings, "Expected value for NOT operation", 0, off);
            instructions.push(Value::Unary(Box::new((get_op(token).unwrap(), val))));
        }
        "&" => {
            let mutable = next_mutable_flag(&mut off, tokens);
            // let val = next_one_value(&mut off, tokens, instructions, bindings, "Expected value to get address from");
            let val = novalue!(&mut off, tokens, instructions, bindings, "Expected value to get address from", 0, off);
            instructions.push(take_pointer(&tokens[..off], bindings, mutable, val));
            return off
            // if let Value::Get(name, _) = val {
            //     if mutable {
            //         let Bind::Let(_, true) = bindings.get(name).unwrap() else { error_range(&tokens[..off], bindings, MUT_ADDR_MESSAGE) };
            //         instructions.push(Value::Ptr(Box::new(val), true));
            //         return off
            //     }
            //     instructions.push(Value::Ptr(Box::new(val), false));
            // }
            // else if let Value::LoadFromPtr(x, _) = val {
            //     if mutable { Type::Ptr(None, true).check(&x).unwrap_or_else(|| error_range(&tokens[..off], bindings, MUT_ADDR_MESSAGE)); }
            //     else {
            //         match *x {
            //             Value::LoadAddress(a, b, c, true) => {
            //                 instructions.push(Value::LoadAddress(a, b, c, false));
            //                 return off
            //             }
            //             Value::Ptr(a, true) => {
            //                 instructions.push(Value::Ptr(a, false));
            //                 return off
            //             },
            //             _ => {}
            //         }
            //     }
            //     instructions.push(*x);
            // }
            // else {
            //     instructions.push(if mutable { Value::Ptr(Box::new(val), true) } else { Value::Ptr(Box::new(val), false) });
            // }
        }
        _ => {
            error(&tokens[0], bindings, "This operation is not allowed here")
        }
    }
    return off
}