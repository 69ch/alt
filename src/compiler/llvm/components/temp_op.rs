use std::fmt::Write;

use crate::{compiler::llvm::{codegen::translate, ir::{IR, ppbind}}, parser::{Operation, Value, components::expr::{is_branch, is_cmp}, r#type::{Type, extract_types}}};

use super::r#type::translate_type;

fn is_unsigned (t: &Type) -> bool {
    match t {
        Type::U(_) | Type::Bool => true,
        _ => false
    }
}

fn cmp<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, lhs: Value<'a>, rhs: Value<'a>, op: Operation) -> String {
    let t = extract_types(&[&lhs, &rhs]).unwrap();
    let typ = translate_type(&t);

    let (lhs, rhs) = ir.type_context(Some(t), |ir| (translate(ir, instructions, lhs).unwrap(), translate(ir, instructions, rhs).unwrap()));

    let prefix = "i"; // todo
    let for_unsigned = is_unsigned(ir.get_current_type().unwrap());

    let c = ir.temp();
    write!(instructions, "%{c} = {prefix}cmp ").unwrap();

    let op = match op {
        Operation::Eq => "eq",
        Operation::NE => "ne",
        op => {
            write!(instructions, "{}", if for_unsigned { "u" } else { "s" }).unwrap();
            match op {
                Operation::GT => "gt",
                Operation::GE => "ge",
                Operation::LT => "lt",
                Operation::LE => "le",
                _ => unreachable!()
            }
        }
    };
    writeln!(instructions, "{op} {typ} {lhs}, {rhs}").unwrap();
    format!("%{c}")
}

fn branch_op<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, lhs: Value<'a>, rhs: Value<'a>, op: Operation) -> String {
    let lhs = ir.type_context(Some(Type::Bool), |ir| translate(ir, instructions, lhs).unwrap());

    let l = ppbind(&ir.seek_local(&Type::Bool).0);
    let next = ir.temp();

    writeln!(instructions, "store i1 {lhs}, ptr %{l}").unwrap();
    match op {
        Operation::And => writeln!(instructions, "br i1 {lhs}, label %{next}, label %end.{next}").unwrap(),
        Operation::Or => writeln!(instructions, "br i1 {lhs}, label %end.{next}, label %{next}").unwrap(),
        _ => todo!()
    }
    let rhs = ir.type_context(Some(Type::Bool), |ir| translate(ir, instructions, rhs).unwrap());
    writeln!(instructions, "store i1 {rhs}, ptr %{l}").unwrap();
    writeln!(instructions, "br label %end.{next}").unwrap();
    writeln!(instructions, "end.{next}:").unwrap();

    let res = ir.temp();
    writeln!(instructions, "%{res} = load i1, ptr %{l}").unwrap();
    format!("%{res}")
}

pub fn expr<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, lhs: Value<'a>, rhs: Value<'a>, op: Operation) -> String {
    // let is_cmp = is_cmp(&op);
    let optype = translate_type(ir.get_current_type().unwrap());
    if is_cmp(&op) {
        return cmp(ir, instructions, lhs, rhs, op)
    }
    if is_branch(&op) {
        return branch_op(ir, instructions, lhs, rhs, op)
    }
    
    let lhs = translate(ir, instructions, lhs).unwrap();
    let rhs = translate(ir, instructions, rhs).unwrap();
    
    // TODO: also f flag for float support

    let for_unsigned = is_unsigned(ir.get_current_type().unwrap());
    
    let c = ir.temp();
    // op
    writeln!(instructions, "%{c} = {} {optype} {lhs}, {rhs}", match op {
        Operation::Add => "add",
        Operation::Sub => "sub",
        Operation::Mul => "mul",
        Operation::Div => {
            if for_unsigned { "udiv" }
            else { "sdiv" }
        },
        Operation::Rem => {
            if for_unsigned { "urem" }
            else { "srem" }
        },
        Operation::BitAnd => "and",
        Operation::And => todo!(),
        _ => { dbg!(op); todo!() }
    }).unwrap();

    return format!("%{c}")
}

pub fn unary<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, op: Operation, value: Value<'a>) -> String {
    let typstr = translate_type(ir.get_current_type().unwrap());
    let value = translate(ir, instructions, value).unwrap();
    let c = ir.temp();
    write!(instructions, "%{c} = ").unwrap();
    match op {
        Operation::Not => {
            writeln!(instructions, "xor {typstr} {value}, -1").unwrap();
        }
        Operation::Sub => {
            writeln!(instructions, "sub {typstr} 0, {value}").unwrap();
        }
        _ => todo!()
    }
    format!("%{c}")
}