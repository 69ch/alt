use std::{collections::HashMap, fmt::Write};

use insordmap::InsordMap;

use crate::{compiler::llvm::{codegen::translate, ir::{IR, ppbind}}, parser::{Value, r#type::Type}};

pub fn translate_type (t: &Type) -> String {
    match t {
        Type::Bool => "i1",
        Type::I(x) | Type::U(x) => return format!("i{x}"),
        Type::Array(t, s) => return format!("[{s} x {}]", translate_type(t)),
        Type::Ptr(Some(t), _) => return format!("{}*", translate_type(t)),
        Type::Ptr(None, _) | Type::Fn(_, _) => "ptr",
        Type::Void | Type::Noret => "void",
        Type::Tuple(v) => return format!("{{{}}}", v.iter().map(|x| translate_type(&x)).collect::<Vec<_>>().join(", ")),
        // Type::Struct(_) => todo!(),
        Type::Struct(name) => return format!("%\"{name}\""),
        Type::Guess => unreachable!(),
        // _ => { dbg!(t); todo!() }
    }.into()
}

pub fn typecast<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, value: Value<'a>, mut from: Type, mut to: Type) -> String {
    // todo smth with that if
    // if let (Value::Int(_) | Value::SInt(_), Type::I(_) | Type::U(_)) = (&value, &to) { return translate(ir, instructions, value).unwrap() }
    let value = translate(ir, instructions, value).unwrap();
    if from == to { return value }
    if let Type::Bool = from { from = Type::I(1) }
    if let Type::Bool = to { to = Type::I(1) }

    macro_rules! typecast_integer {
        ($x:expr, $y:expr, $extflag:literal) => {
            let from = translate_type(&from);
            let to = translate_type(&to);
            if $x < $y {
                let x = $extflag;
                typecast_integer!("%{} = {x}ext {from} {value} to {to}");
            }
            else if $x > $y {
                typecast_integer!("%{} = trunc {from} {value} to {to}");
            }
        };
        ($once:expr) => {
            let tpt = ir.temp();
            writeln!(instructions, $once, tpt).unwrap();
            return format!("%{tpt}")
        }
    }

    match (&from, &to) {
        (Type::I(x), Type::I(y) | Type::U(y)) => {
            typecast_integer!(x, y, 's');
        }
        (Type::U(x), Type::I(y) | Type::U(y)) => {
            typecast_integer!(x, y, 'z');
        }
        _ => {}
    }
    return value
}

// pub fn r#struct<'a> (ir: &mut IR<'a>, name: &str, types: insordmap::IntoValues<&'a String, Type>) {
pub fn r#struct<'a> (ir: &mut IR<'a>, name: String, kv: InsordMap<String, Type>) {
    ir.global_write(&format!("%\"{name}\" = type {{ {} }}", kv.values().map(|x| translate_type(x)).collect::<Vec<String>>().join(", ")));
    ir.bind_type(name, kv);
}

pub fn struct_init<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, name: String, mut kv: HashMap<&'a String, Value<'a>>) -> Option<String> {
    let mut typ = Type::Struct(name.clone());
    let (init, put_in) = ir.seek_local(&typ);
    let init = ppbind(&init);
    let typstr = translate_type(&typ);
    let Some(kt) = ir.get_type(&name) else { panic!() };
    let kt = kt.clone_kv();
    let mut index = 0;
    for (k, t) in kt {
        let Some(v) = kv.remove(k.as_ref()) else { todo!() };
        let gep = ir.temp();
        ir.mark_put_in(gep);
        writeln!(instructions, "%{gep} = getelementptr inbounds {typstr}, ptr %{init}, i64 0, i32 {index}").unwrap();
        if let Some(l) = ir.ref_type_context(&mut typ, |ir| translate(ir, instructions, v)) {
            writeln!(instructions, "store {} {l}, ptr %{gep}", translate_type(&t)).unwrap();
        }
        index += 1;
    }
    ir.unmark_put_in();
    if !put_in {
        let load = ir.temp();
        writeln!(instructions, "%{load} = load {typstr}, ptr %{init}").unwrap();
        return Some(format!("%{load}"))
    }
    None
}

// to future self: now do field access I guess