use std::fmt::Write;

use crate::{compiler::llvm::{codegen::translate, ir::{IR, ppbind}}, parser::{Value, r#type::{Type, default_type, penetrate_type}}};

use super::r#type::translate_type;

macro_rules! store {
    ($s:expr, $t:expr, $v:expr, $dest:expr) => {
        writeln!($s, "store {} {}, ptr %{}", $t, $v, $dest).unwrap();
    };
}
macro_rules! load {
    ($s:expr, $t:expr, $from:expr) => {
        writeln!($s, "load {}, ptr %{}", $t, $from).unwrap();
    };
}

macro_rules! gct {
    ($b:expr, $r:expr) => {
        {
            let Some(t) = $b.get_current_type() else { return $r }; // if there is no type, then value probably just hanging around somewhere in code, not used
            t
        }
    };
}

pub fn new_var<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, name: &'a str, typ: Type, value: Option<Box<Value<'a>>>) {
    let typstr = translate_type(&typ);
    // let new_t = ir.reserve_local(&typ);
    let new_t = ir.seek_local(&typ).0;
    dbg!(new_t, &typstr);
    if let Some(value) = value {
        // ir.bind(MARK_PUT_IN, new_t);
        ir.mark_put_in(new_t.0);
        // dbg!(&typ);
        let Some(value) = ir.type_context(Some(typ), |ir| translate(ir, instructions, *value)) else {
            // ir.remove_bind(MARK_PUT_IN);
            ir.unmark_put_in();
            ir.bind(name, new_t.0, true);
            return
        };
        // ir.remove_bind(MARK_PUT_IN);
        ir.unmark_put_in();
        ir.bind(name, new_t.0, true);
        store!(instructions, typstr, value, ppbind(&new_t));
    }
    else {
        ir.bind(name, new_t.0, true);
    }
}

pub fn new_arg<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, name: &'a str, typ: Type, initial: usize) {
    let typstr = translate_type(&typ);
    let new_t = ir.temp();
    ir.bind(name, new_t, false);
    let x = ir.get_prologue();
    writeln!(x, "%{new_t} = alloca {typstr}").unwrap();
    let value = format!("%{initial}");
    store!(instructions, typstr, value, new_t);
}

pub fn get_var<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, name: &'a str, typ: Type) -> String {
    let typstr = translate_type(&typ);
    let c = ir.temp();
    let from = ir.get_bind(name).unwrap();
    dbg!(name, from);
    write!(instructions, "%{c} = ").unwrap();
    load!(instructions, typstr, ppbind(from));
    format!("%{c}")
}

pub fn modify_var<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, name: &'a str, typ: Type, value: Value<'a>) {
    let typstr = translate_type(&typ);
    let p = *ir.get_bind(name).unwrap();
    // ir.bind(MARK_PUT_IN, p);
    ir.mark_put_in(p.0);
    let value = ir.type_context(Some(typ), |ir| translate(ir, instructions, value)).unwrap();
    // ir.remove_bind(MARK_PUT_IN);
    ir.unmark_put_in();
    // writeln!(instructions, "store {typstr} {value}, ptr %{p}").unwrap();
    writeln!(instructions, "store {typstr} {value}, ptr %{}", ppbind(&p)).unwrap();
}

pub fn deref<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, value: Value<'a>, typ: Type) -> String {
    let typstr = translate_type(&typ);
    // let d = get_var(ir, instructions, name, Type::Ptr(None));
    let d = translate(ir, instructions, value).unwrap();
    let c = ir.temp();
    write!(instructions, "%{c} = ").unwrap();
    writeln!(instructions, "load {typstr}, ptr {d}").unwrap();
    format!("%{c}")
}

pub fn modify_pointer<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, f: Value<'a>, typ: Type, value: Value<'a>) {
    let typstr = translate_type(&typ);
    // let f = deref(ir, instructions, f, typ.clone());
    let value = ir.type_context(Some(typ), |ir| translate(ir, instructions, value)).unwrap();
    let f = translate(ir, instructions, f).unwrap();
    // let deref = get_var(ir, instructions, name, Type::Ptr(None));
    writeln!(instructions, "store {typstr} {value}, ptr {f}").unwrap();
}

// todo check for bugs
pub fn array<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, arr: Vec<Value<'a>>) -> Option<String> {
    // let typ = ir.get_current_type().unwrap().clone();
    let typ = gct!(ir, None).clone();
    let (current, put_in) = ir.seek_local(&typ);
    let current = ppbind(&current);
    let rtypstr = translate_type(&typ);
    let typstr = translate_type(&penetrate_type(typ.clone(), 0));
    let Type::Array(mut subtyp, _) = typ else { panic!() };
    let subtypstr = translate_type(&subtyp);
    
    let mut index = 0;

    for i in arr {
        let gep = ir.temp();
        writeln!(instructions, "%{gep} = getelementptr inbounds {typstr}, ptr %{current}, i64 {index}").unwrap();
        ir.mark_put_in(gep);

        if let Some(item) = ir.ref_type_context(&mut subtyp, |ir| translate(ir, instructions, i)) {
            writeln!(instructions, "store {subtypstr} {item}, ptr %{gep}").unwrap();
        }
        index += 1;
    }

    ir.unmark_put_in();

    if !put_in {
        let load = ir.temp();
        writeln!(instructions, "%{load} = load {rtypstr}, ptr %{current}").unwrap();
        return Some(format!("%{load}"))
    }
    None
}

pub fn load_address<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, from: Value<'a>, index: Value<'a>, typ: Type) -> String {
    let bits = match &typ {
        Type::Tuple(_) | Type::Struct(_) => 32,
        // Type::Struct(_) => todo!(),
        _ => 64
    };
    let f = match &typ { Type::Ptr(_, _) => "", _ => " i64 0," };
    let from = translate(ir, instructions, if Type::Ptr(None, false).check(&from).is_some() { from } else { Value::Ptr(Box::new(from), false) }).unwrap(); //  || Type::PtrMut(None).check(&from).is_some()
    let typstr = translate_type(&typ);
    
    let index = translate(ir, instructions, index).unwrap();

    let gep = ir.temp();
    writeln!(instructions, "%{gep} = getelementptr inbounds {typstr}, ptr {from},{f} i{bits} {index}").unwrap();

    return format!("%{gep}")
}

pub fn ptrinit<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, to: Value<'a>) -> String {
    // dbg!(ir.get_current_type());
    let x = if let Type::Ptr(Some(x), _) = ir.get_current_type().unwrap() { x } else { &default_type(&to) };
    let typstr = translate_type(&x);
    let x = x.clone();
    let c = ir.reserve_local(&x);
    let val = ir.type_context(Some(x.clone()), |ir| translate(ir, instructions, to).unwrap());
    let c = ppbind(&c);
    store!(instructions, typstr, val, c);
    format!("%{c}")
}

pub fn tuple<'a> (ir: &mut IR<'a>, instructions: &mut impl Write, tuple: Vec<Value<'a>>) -> Option<String> {
    // let typ = ir.get_current_type().unwrap().clone();
    // let Some(typ) = ir.get_current_type() else { return None }; // if there is no type provided, then value just somewhere in the code, not used
    // let typ = typ.clone();
    let typ = gct!(ir, None).clone();
    let typstr = translate_type(&typ);
    // let init = ir.reserve_local(&typ);
    let (init, put_in) = ir.seek_local(&typ);
    let init = ppbind(&init);
    // ir.unmark_put_in();
    let Type::Tuple(mut subtyp) = typ else { panic!() };
    for (index, value) in tuple.into_iter().enumerate() {
        let gep = ir.temp();
        ir.mark_put_in(gep);
        writeln!(instructions, "%{gep} = getelementptr inbounds {typstr}, ptr %{init}, i64 0, i32 {index}").unwrap();
        if let Some(x) = ir.ref_type_context(&mut subtyp[index], |ir| translate(ir, instructions, value)) {
            writeln!(instructions, "store {} {x}, ptr %{gep}", translate_type(&subtyp[index])).unwrap();
        }
    }
    ir.unmark_put_in();
    if !put_in {
        let load = ir.temp();
        writeln!(instructions, "%{load} = load {typstr}, ptr %{init}").unwrap();
        return Some(format!("%{load}"))
    }
    None
}