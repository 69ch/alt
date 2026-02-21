use std::fmt::Write;

use crate::{compiler::llvm::{components::{control_flow::unreachable, r#type::{r#struct, struct_init, typecast}}, ir::ppbind}, parser::Value};

use super::{components::{control_flow::{r#break, r#continue, r#else, r#if, r#loop}, global::{call, declare, define, ret}, memory::{array, deref, get_var, load_address, modify_pointer, modify_var, new_var, ptrinit, tuple}, temp_op::{expr, unary}}, ir::IR};

pub fn translate<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, value: Value<'a>) -> Option<String> {
    match value {
        Value::Int(x)  => return Some(x.to_string()),
        Value::SInt(x) => return Some(x.to_string()),
        Value::Bool(x) => return Some((x as u8).to_string()),
        Value::Expr(x) => {
            let (lhs, rhs, op) = *x;
            return Some(expr(ir, instructions, lhs, rhs, op))
        },
        Value::Unary(x) => return Some(unary(ir, instructions, x.0, x.1)),
        Value::Extern(name, args, ret) => declare(ir, name, args, ret),
        Value::Function { name, args, body, ret } => define(ir, name, args, body, ret),
        // Value::FunctionCall(name, ret, args) => return call(ir, instructions, name, args, ret),
        Value::AnonFunction { args, body, ret } => {
            let anon = ir.anon();
            let name = format!(".anon.{anon}");
            ir.isolated(|ir| {
                define(ir, name.clone(), args, body, ret);
            });
            return Some(format!("@{name}"))
        },
        Value::Call(x, argv) => return call(ir, instructions, *x, argv, false),
        Value::Return(x) => {
            let (value, typ) = *x;
            ret(ir, instructions, value, typ)
        },
        Value::InitVar(name, typ, value) => new_var(ir, instructions, name, typ, value),
        Value::ModifyVar(name, typ, value) => modify_var(ir, instructions, name, typ, *value),
        Value::ModifyByPointer(x) => modify_pointer(ir, instructions, x.0, x.1, x.2),
        Value::Get(name, typ) => return Some(get_var(ir, instructions, name, typ)),
        Value::LoadFromPtr(value, typ) => return Some(deref(ir, instructions, *value, typ)),
        Value::Ptr(to, _) => {
            dbg!(&to);
            match *to {
                Value::Get(name, _) => {
                    return Some(format!("%{}", ppbind(ir.get_bind(name).unwrap())))
                },
                Value::String(x) => {
                    let len = x.len();
                    let x = x.replace("\n", "\\0A");
                    let s = ir.constant(format!("[{len} x i8] c\"{x}\""));
                    return Some(format!("{s}"))
                },
                to => return Some(ptrinit(ir, instructions, to))
            }
        }
        Value::FunctionPointer(n, _, _) => return Some(format!("@\"{n}\"")),
        Value::String(x) => {
            if let Some(t) = ir.get_mark_put_in().cloned() {
                let len = x.len();
                let x = x.replace("\n", "\\0A");
                let s = ir.constant(format!("[{len} x i8] c\"{x}\""));
                let t = ppbind(&t);
                writeln!(instructions, "call void @llvm.memcpy.inline.p0.p0.i64(ptr %{t}, ptr {s}, i64 {len}, i1 false)").unwrap();
            }
        },
        Value::Array(arr) => return array(ir, instructions, arr),
        // Value::LoadAddress(from, i, t) => return Some(load_address(ir, instructions, *from, i, t)),
        Value::LoadAddress(from, index, typ, _) => return Some(load_address(ir, instructions, *from, *index, typ)),

        Value::Tuple(tup) => return tuple(ir, instructions, tup),
        Value::Struct { name, kv, .. } => r#struct(ir, name, kv),
        Value::StructInit(name, kv) => return struct_init(ir, instructions, name, kv),

        Value::If { condition, body, else_then } => r#if(ir, instructions, *condition, body, *else_then),
        Value::Else(body) => r#else(ir, instructions, body),
        Value::Loop(body, label) => r#loop(ir, instructions, body, label),
        Value::Break(label) => r#break(ir, instructions, label),
        Value::Continue(label) => r#continue(ir, instructions, label),
        Value::Unreachable => unreachable(instructions),

        Value::Typecast(value, from, to) => return Some(typecast(ir, instructions, *value, from, to)),

        // Value::ReservedLocals(a) => ir.count_locals(a),
        Value::SharedValue(x) => return translate(ir, instructions, x.replace(Value::Unreachable)),
        Value::PromisedFunction { .. } | Value::PromisedStruct { .. } | Value::ReturnMark | Value::Namespace(_) => (),
        _ => { dbg!(value); todo!() }
    }
    None
}

pub fn translate_all<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, program: Vec<Value<'a>>) {
    for value in program {
        translate(ir, instructions, value);
    }
}

pub fn emit_llvm (program: Vec<Value>) -> String {
    let mut ir = IR::default();
    let mut w = String::new();
    translate_all(&mut ir, &mut w, program);
    ir.move_global()
}