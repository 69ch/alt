use std::{fmt::Write, mem};

use crate::{compiler::llvm::{codegen::{translate, translate_all}, ir::IR}, parser::{components::args::Arg, r#type::{extract_type, Type}, Value}};

use super::{memory::new_arg, r#type::translate_type};

fn translate_args (ir: &mut IR, args: &Vec<Arg>) -> Vec<String> {
    args.iter().map(|x| {
        ir.temp();
        translate_type(&x.typ)
    }).collect()
}
pub fn declare<'a> (ir: &mut IR<'a>, name: &'a str, args: Vec<Type>, ret: Type) {
    // let args = translate_args(ir, &args).join(", ");
    let args = args.iter().map(|x| translate_type(x)).collect::<Vec<String>>().join(", ");
    let retstr = translate_type(&ret);
    ir.global_write(&format!("declare {retstr} @{name} ({args})"));
    ir.leave();
}
pub fn define<'a, 'b> (ir: &mut IR<'a>, name: String, mut args: Vec<Arg<'a>>, body: Vec<Value<'a>>, ret: Type) {
    ir.join();
    let argss = translate_args(ir, &args);
    let mut body_buf = String::new();
    ir.new_prologue();
    argss.iter().enumerate().for_each(|(index, _)| new_arg(ir, &mut body_buf, args[index].name, mem::replace(&mut args[index].typ, Type::Void), index));
    translate_all(ir, &mut body_buf, body);
    // dbg!(&ir.prologue());
    let body_buf = ir.move_prologue() + &body_buf;
    ir.global_write(&format!("define {} @\"{name}\" ({}) {{\nentry:\n    {}\n}}", translate_type(&ret), argss.join(", "), body_buf.trim().replace("\n", "\n    ")));
    ir.leave();
}

// pub fn call<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, name: &'a str, args: Vec<(Type, Value<'a>)>, ret: Type) -> Option<String> {
pub fn call<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, x: Value<'a>, argv: Vec<Value<'a>>, tail: bool) -> Option<String> {
    let Type::Fn(argt, ret) = extract_type(&x).unwrap() else { unreachable!() };
    let name = translate(ir, instructions, x).unwrap();
    let typstr = translate_type(&ret);
    let mut args = (argt.into_iter(), argv.into_iter());
    let mut args_llvm = vec![];
    while let (Some(typ), Some(value)) = (args.0.next(), args.1.next()) {
        let typstr = translate_type(&typ);
        let value = ir.type_context(Some(typ), |ir| translate(ir, instructions, value)).unwrap();
        args_llvm.push(format!("{typstr} {value}"));
    }
    let mut res = None;
    if *ret != Type::Void && *ret != Type::Noret {
        let c = ir.temp();
        res = Some(format!("%{c}"));
        write!(instructions, "%{c} = ").unwrap();
    }
    // writeln!(instructions, "call {typstr} @\"{name}\" ({})", args_llvm.join(", ")).unwrap(); // todo
    let tail = if tail { "tail " } else { "" };
    writeln!(instructions, "{tail}call {typstr} {name} ({})", args_llvm.join(", ")).unwrap();
    res
}

pub fn ret<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, value: Option<Value<'a>>, typ: Type) {
    if let Some(value) = value {
        let mut typstr = translate_type(&typ);
        let mut retbody = String::new();
        let value = ir.type_context(Some(typ), |ir| {
            if let Value::Call(x, argv) = value { call(ir, &mut retbody, *x, argv, true) }
            else { translate(ir, &mut retbody, value) }
        });
        if let Some(_) = value { typstr += " " }
        write!(instructions, "{retbody}").unwrap();
        writeln!(instructions, "ret {typstr}{}", value.unwrap_or_default()).unwrap();
        return
    }
    writeln!(instructions, "ret {}", translate_type(&typ)).unwrap();
}