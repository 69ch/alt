use crate::{lexer::{Token, TokenKind}, parser::{bindings::Bindings, message::{assert, error}, r#type::{default_type, extract_types_move, Type}, value, value_loop, Operation, Value}};

// use super::binds::reserve_local;

// use super::{ bindings::Bindings, message::assert, value, Operation, Value };

pub fn get_op (token: &Token) -> Option<Operation> {
    match token.value.as_str() {
        "+" => Some(Operation::Add),
        "-" => Some(Operation::Sub),
        "*" => Some(Operation::Mul),
        "/" => Some(Operation::Div),
        "%" => Some(Operation::Rem),
        "<" => Some(Operation::LT), ">" => Some(Operation::GT),
        "<=" => Some(Operation::LE), ">=" => Some(Operation::GE),
        "==" => Some(Operation::Eq), "!=" => Some(Operation::NE),
        "&&" => Some(Operation::And), "||" => Some(Operation::Or),
        "!" => Some(Operation::Not),
        "&" => Some(Operation::BitAnd), "|" => Some(Operation::BitOr),
        ">>" | "<<" | "^" => todo!(),
        _ => None
    }
}

pub fn precedence (op: &Operation) -> u8 {
    match op {
        Operation::Div | Operation::Mul | Operation::Rem | Operation::BitAnd | Operation::BitOr | Operation::Not => 0,
        Operation::Add | Operation::Sub => 1,
        Operation::Eq | Operation::NE | Operation::GE | Operation::GT | Operation::LE | Operation::LT => 2,
        Operation::And | Operation::Or => 3
    }
}

pub fn is_cmp (op: &Operation) -> bool {
    match op {
        Operation::Add | Operation::Sub | Operation::Div | Operation::Mul |
        Operation::Rem | Operation::And | Operation::Or |
        Operation::BitAnd | Operation::BitOr | Operation::Not => false,
        Operation::Eq | Operation::NE | Operation::GE | Operation::GT | Operation::LE | Operation::LT => true
    }
}

pub fn is_branch (op: &Operation) -> bool {
    match op {
        Operation::And | Operation::Or => true,
        _ => false
    }
}

/// Algorithm for expression parsing \
/// TODO: maybe optimize by excluding excess loops through 'promised' vector and treating 'values' and 'promised' as hashmaps (position, value)
pub fn expr<'a> (tokens: &'a [Token], instructions: &mut Vec<Value<'a>>, bindings: &mut Bindings<'a>) -> usize {
    let mut values = vec![instructions.pop().unwrap()];
    let mut promised: Vec<Operation> = vec![];
    
    // assert(Type::Ptr(None, false).check(&values[0]).is_none(), &tokens[0], bindings, "Clear pointer arithmetic is currently unavailable");
    if let Type::U(_) | Type::I(_) = default_type(&values[0]) {}
    else { bindings.gentle_error(&tokens[0], "Unsupported type"); }

    let mut i = 0;
    while i < tokens.len() {
        let tokens = &tokens[i..];
        let Some(op) = get_op(&tokens[0]) else { break };
        let a = value(&tokens[1..], &mut values, bindings);
        let b = value_loop(&tokens[1..], &mut values, bindings, a, &[(Some(TokenKind::Arithmetic), None), (Some(TokenKind::Bitwise), None), (Some(TokenKind::Logical), None)]);
        let len = b + 1;
        i += len;
        promised.push(op);
    }

    extract_types_move(&values).unwrap_or_else(|| error(&tokens[0], bindings, "Heterogeneous types"));
    assert(values.len() == promised.len() + 1, &tokens[i-1], bindings, "Failed parsing binary expression");
    
    let mut current_prec = 0;
    let mut j = 0;
    loop {
        if promised.is_empty() { break }
        if j >= promised.len() { j = 0; current_prec += 1; }
        
        // idea: optimize remove: do not use remove, take value behind index and replace it with NoOp, so there's no reallocation, just little memory overhead
        if precedence(&promised[j]) == current_prec {
            let lhs = values.remove(j); let rhs = values.remove(j);
            let op = promised.remove(j);
            assert(!is_branch(&op) || (Type::Bool.check(&lhs).is_some() && Type::Bool.check(&rhs).is_some()), &tokens[0], bindings, "'&&' and '||' accepts only boolean-typed values");
            // if is_branch(&op) { reserve_local(bindings); }
            values.insert(j, Value::Expr(Box::new((lhs, rhs, op))));
            continue
        }

        j += 1;
    }


    // instructions.extend(values);
    instructions.push(values.pop().unwrap());

    i
}

#[allow(dead_code)]
fn print_expr (x: &Value) -> String {
    match x {
        Value::Expr(x) => {
            let (ref lhs, ref rhs, op) = **x;
            match op {
                Operation::Add => format!("({} + {})", print_expr(lhs), print_expr(rhs)),
                Operation::Mul => format!("({} * {})", print_expr(lhs), print_expr(rhs)),
                Operation::Sub => format!("({} - {})", print_expr(lhs), print_expr(rhs)),
                Operation::Div => format!("({} / {})", print_expr(lhs), print_expr(rhs)),
                // Operation:: => format!("({} / {})", print_expr(lhs), print_expr(rhs))
                _ => "null".into()
            }
        }
        Value::Int(x) => format!("{x}"),
        Value::Float(x) => format!("{x}"),
        _ => {
            "null".into()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use super::*;

    macro_rules! test_case {
        ($code:expr, $expect:expr) => {
            let code = $code;
            let tokens = lex(code);
            let mut result = parse_program(&tokens, &mut Bindings::new(code, "".into(), None));
            // dbg!(&result);
            assert_eq!(eval_expr(result.remove(0)), $expect);
        };
    }

    #[test]
    fn simple () {
        test_case!("2 + 2 * 2", 6.0);
    }

    #[test]
    fn medium () {
        test_case!("4 * 2 / 5 * 2 + 2 * 4", 11.2);
    }

    #[test]
    fn hard () {
        test_case!("4 * 2 / 5 * 2 + 2 * 4 * 5 / 2 * 4 + 4 * 2 / 5 * 2 - 2 * 4 * 5 / 2 * 4 + 4 * 2 / 5 * 2 + 2 * 4 * 5 / 2 * 4", 89.60000000000001);
    }

    #[test]
    fn medium2 () {
        test_case!("4 / 2 * 2 + 2 - 2 / 2", 5.0);
    }

    #[test]
    fn sub () {
        test_case!("1 - 2", -1.0);
    }
    #[test]
    fn sub2 () {
        test_case!("1 * 1 / 2 - 2", -1.5);
    }
    #[test]
    fn sub3 () {
        test_case!("1 * 1 / 2 - 2 / 1 * 2", -3.5);
    }

    #[test]
    fn nested () {
        test_case!("2 * (1 + 2)", 6.0);
    }

    #[test]
    fn simple2 () {
        test_case!("1 + 2 / 2", 2.0);
    }

    #[test]
    fn logical_simple () {
        test_case!("4 / 2 * 2 + 2 - 2 / 2 < 4 / 2 * 2 + 2 - 2 / 2", 0.0);
    }
    #[test]
    fn logical_simple2 () {
        test_case!("4 / 2 * 2 + 2 - 2 / 2 < 4 / 2 * 2 + 2 + 2 / 2", 1.0);
    }

    fn eval_expr (x: Value) -> f64 {
        match x {
            Value::Expr(x) => {
                let (lhs, rhs, op) = *x;
                match op {
                    Operation::Add => eval_expr(lhs) + eval_expr(rhs),
                    Operation::Mul => eval_expr(lhs) * eval_expr(rhs),
                    Operation::Sub => eval_expr(lhs) - eval_expr(rhs),
                    Operation::Div => eval_expr(lhs) / eval_expr(rhs),
                    Operation::LT => if eval_expr(lhs) < eval_expr(rhs) { 1.0 } else { 0.0 },
                    _ => 0.0
                }
            }
            Value::Int(x) => x as f64,
            Value::Float(x) => x,
            _ => {
                0.0
            }
        }
    }
}