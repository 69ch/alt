use std::{collections::HashMap, fmt::Write, mem};

use insordmap::InsordMap;

use crate::parser::r#type::Type;

use super::components::r#type::translate_type;

pub fn ppbind (v: &(usize, bool)) -> String {
    format!("{}{}", if v.1 { "_" } else { "" }, v.0)
}

#[derive(Debug, Default)]
pub struct IR<'a> {
    global: String,
    constants: (HashMap<String, String>, usize),
    anon_counter: usize,
    // binds: Vec<HashMap<&'a str, usize>>,
    binds: Vec<HashMap<&'a str, (usize, bool)>>,
    types: HashMap<String, InsordMap<String, Type>>,
    current_type: Option<Type>,
    /// (current, max)
    // reserved_locals: (usize, usize),
    reserved_locals: usize,
    prologue: String,
    temp_counter: usize
}

impl<'a> IR<'a> {
    pub fn global_write (&mut self, line: &str) {
        self.global += line;
        self.global += "\n";
    }
    
    pub fn move_global (self) -> String {
        self.global
    }

    pub fn constant (&mut self, k: String) -> String {
        if let Some(l) = self.constants.0.get(&k) {
            return l.clone()
        }
        else {
            let t = self.constants.1;
            self.constants.1 += 1;
            let key = format!("@.const.{t}");
            self.global_write(&format!("{key} = constant {k}"));
            self.constants.0.insert(k, key.clone());
            key
        }
    }

    pub fn join (&mut self) { self.binds.push(HashMap::new()); }
    pub fn leave (&mut self) {
        self.binds.pop();
        if self.binds.is_empty() {
            self.temp_counter = 0;
            // self.reserved_locals = (0, 0);
            self.reserved_locals = 0;
        }
    }
    pub fn isolated (&mut self, f: impl FnOnce (&mut Self)) {
        let reserved_locals = mem::take(&mut self.reserved_locals);
        let temp_counter = mem::take(&mut self.temp_counter);
        let prologue = mem::take(&mut self.prologue);
        f(self);
        self.temp_counter = temp_counter;
        self.reserved_locals = reserved_locals;
        self.prologue = prologue;
    }
    
    pub fn temp (&mut self) -> usize {
        let s = self.temp_counter;
        self.temp_counter += 1;
        s
    }

    pub fn last_temp (&mut self) -> usize {
        self.temp_counter - 1
    }

    pub fn anon (&mut self) -> usize {
        let s = self.anon_counter;
        self.anon_counter += 1;
        s
    }

    // pub fn bind (&mut self, k: &'a str, v: usize) { self.binds.last_mut().unwrap().insert(k, v); }
    pub fn bind (&mut self, k: &'a str, v: usize, l: bool) { self.binds.last_mut().unwrap().insert(k, (v, l)); }
    pub fn get_bind (&self, name: &'a str) -> Option<&(usize, bool)> {
        let content = &self.binds;
        for i in content.iter().rev() {
            if let Some(x) = i.get(name) {
                // return Some(x)
                return Some(x)
            }
        }
        None
    }
    pub fn remove_bind (&mut self, k: &'a str) { self.binds.last_mut().unwrap().remove(k); }

    pub fn type_context<T> (&mut self, typ: Option<Type>, body: impl FnOnce(&mut Self) -> T) -> T {
        let temp = mem::replace(&mut self.current_type, typ);
        let res = body(self);
        self.current_type = temp;
        res
    }
    pub fn ref_type_context<T> (&mut self, typ: &mut Type, body: impl FnOnce(&mut Self) -> T) -> T {
        let t = mem::take(typ);
        let temp = mem::replace(&mut self.current_type, Some(t));
        let res = body(self);
        let t = mem::replace(&mut self.current_type, temp);
        *typ = t.unwrap();
        res
    }
    pub fn get_current_type (&self) -> Option<&Type> { self.current_type.as_ref() }

    pub fn new_prologue (&mut self) { self.prologue = String::new(); }
    pub fn move_prologue (&mut self) -> String { mem::take(&mut self.prologue) }
    pub fn get_prologue (&mut self) -> &mut String { &mut self.prologue }

    // pub fn count_locals (&mut self, a: usize) {
    //     let curr = self.temp_counter;
    //     self.temp_counter += a;
    //     self.reserved_locals = (curr, self.temp_counter);
    // }

    // pub fn reserve_local (&mut self, typ: &Type) -> usize {
    //     // dbg!(&self.reserved_locals);
    //     let c = self.reserved_locals.0;
    //     self.reserved_locals.0 += 1;
    //     // dbg!(self.reserved_locals);
    //     assert!(self.reserved_locals.0 <= self.reserved_locals.1);

    //     let typstr = translate_type(typ);
    //     writeln!(self.prologue, "%{c} = alloca {typstr}").unwrap();
    //     c
    // }

    pub fn reserve_local (&mut self, typ: &Type) -> (usize, bool) {
        let c = self.reserved_locals;
        self.reserved_locals += 1;

        let typstr = translate_type(typ);
        writeln!(self.prologue, "%_{c} = alloca {typstr}").unwrap();
        (c, true)
    }

    pub fn seek_local (&mut self, typ: &Type) -> ((usize, bool), bool) {
        if let Some(x) = self.get_bind(MARK_PUT_IN).cloned() { (x, true) } else { (self.reserve_local(&typ), false) }
    }
    pub fn mark_put_in (&mut self, l: usize) {
        self.bind(MARK_PUT_IN, l, true);
    }
    pub fn unmark_put_in (&mut self) {
        self.remove_bind(MARK_PUT_IN);
    }
    pub fn get_mark_put_in (&mut self) -> Option<&(usize, bool)> {
        self.get_bind(MARK_PUT_IN)
    }

    pub fn bind_type (&mut self, name: String, kv: InsordMap<String, Type>) {
        self.types.insert(name, kv);
    }
    pub fn get_type (&self, name: &str) -> Option<&InsordMap<String, Type>> {
        self.types.get(name)
    }
    // pub fn get_type_mut (&mut self, name: &str) -> Option<&mut InsordMap<String, Type>> {
    //     self.types.get_mut(name)
    // }
}

const MARK_PUT_IN: &str = "@put_in";