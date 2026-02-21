use std::fmt::Write;

use crate::{compiler::llvm::{codegen::{translate, translate_all}, ir::IR}, parser::{r#type::Type, Value}};

fn is_terminator (v: &Value) -> bool {
    match v {
        Value::Return(_) | Value::ReturnMark | Value::Break(_) | Value::Continue(_) | Value::Unreachable => true,
        _ => false
    }
}

fn find_terminator (values: &[Value]) -> bool {
    if let Some(v) = values.last() {
        is_terminator(v)
    }
    else {
        false
    }
}

pub fn r#if<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, condition: Value<'a>, body: Vec<Value<'a>>, next: Option<Value<'a>>) {
    // let typ = if let Value::Expr(ref x) = condition { default_type_expr(&x.0, &x.1, &x.2) } else { Type::Bool };
    let typ = Type::Bool;
    let condition = ir.type_context(Some(typ), |ir| translate(ir, instructions, condition)).unwrap();
    let mut bodybuf = String::new();
    let true_label = ir.temp();
    let terminates = find_terminator(&body);
    ir.join();
    translate_all(ir, &mut bodybuf, body);
    ir.leave();
    let false_label = ir.temp();
    let mut end_label = false_label;
    writeln!(instructions, "br i1 {condition}, label %{true_label}, label %{false_label}").unwrap();

    let mut next_buf = String::new();
    if let Some(next) = next {
        translate(ir, &mut next_buf, next);
        end_label = ir.last_temp();
    }

    if !terminates { writeln!(bodybuf, "br label %{end_label}").unwrap(); }
    writeln!(instructions, "{true_label}:\n    {}", bodybuf.trim().replace("\n", "\n    ")).unwrap();
    if false_label != end_label {
        writeln!(instructions, "{false_label}:\n    {}", next_buf.trim().replace("\n", "\n    ")).unwrap();
    }
}

pub fn r#else<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, body: Vec<Value<'a>>) {
    let terminates = find_terminator(&body);
    ir.join();
    translate_all(ir, instructions, body);
    ir.leave();
    let end = ir.temp();
    if !terminates { writeln!(instructions, "br label %{end}").unwrap(); }
}


pub fn r#loop<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, body: Vec<Value<'a>>, label: Option<&'a str>) {
    let (start_label, end_label) = if let Some(label) = label {
        let l = ir.temp();
        ir.bind(label, l, false);
        ir.bind(CURR_LABEL, l, false);
        (format!("{l}"), format!("end.{l}"))
    } else {
        let l = ir.temp();
        ir.bind(CURR_LABEL, l, false);
        (format!("{l}"), format!("end.{l}"))
    };
    writeln!(instructions, "br label %{start_label}").unwrap();
    writeln!(instructions, "{start_label}:").unwrap();
    let terminates = find_terminator(&body);
    let mut bodystr = String::new();
    ir.join();
    translate_all(ir, &mut bodystr, body);
    ir.leave();
    writeln!(instructions, "    {}", bodystr.trim().replace("\n", "\n    ")).unwrap();
    if !terminates { writeln!(instructions, "    br label %{start_label}").unwrap() }
    writeln!(instructions, "{end_label}:").unwrap();
}

pub fn r#break<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, label: Option<&'a str>) {
    if let Some(label) = label {
        let label = ir.get_bind(label).unwrap().0;
        writeln!(instructions, "br label %end.{label}").unwrap();
    }
    else {
        let label = ir.get_bind(CURR_LABEL).unwrap().0;
        writeln!(instructions, "br label %end.{label}").unwrap();
    }
}
pub fn r#continue<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, label: Option<&'a str>) {
    if let Some(label) = label {
        let label = ir.get_bind(label).unwrap().0;
        writeln!(instructions, "br label %{label}").unwrap();
    }
    else {
        let label = ir.get_bind(CURR_LABEL).unwrap().0;
        writeln!(instructions, "br label %{label}").unwrap();
    }
}

pub fn unreachable (instructions: &mut impl Write) {
    writeln!(instructions, "unreachable").unwrap();
}

pub const CURR_LABEL: &str = "@current_label";