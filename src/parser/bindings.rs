use std::{collections::{HashMap, HashSet}, mem, path::PathBuf};

use crate::{compiler::Target, lexer::Token, parser::{message::{parsing_error_message, point, point_range}, SharedValue}};

use super::r#type::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Context { Fn(Type), If, Loop, Array }

#[derive(Debug, Clone)]
pub enum Bind<'a> {
    /** (type, mutable) */
    Let(Type, bool),
    /** (args, return, shared pointer to raw function in AST) */
    Function(Vec<Type>, Type, Option<SharedValue<'a>>),
    Label,
    Mark,
    // Counter(usize),
    // Type(Type)
    // Namespace(Vec<String>),
    Namespace(HashMap<String, Bind<'a>>),
    Type(Option<SharedValue<'a>>),
    Alias(String),
    // Public(Box<Bind<'a>>),
    // Value(Value<'a>)
}

macro_rules! comptime_if {
    (true $x:tt $y:tt) => { $x };
    (false $x:tt $y:tt) => { $y }
}

macro_rules! bget {
    ($self:expr, $name:expr, $mut:tt) => {
        return {
            let mut barrier_passed = false;
            for i in {comptime_if!($mut { $self.content.iter_mut() } { $self.content.iter() })}.rev() {
                let bp = if let Some(Bind::Mark) = i.get(MARK_BARRIER) { true } else { false };
                if let Some(x) = {comptime_if!($mut { i.get_mut($name) } { i.get($name) })} {
                    if let Bind::Let(_, _) | Bind::Mark | Bind::Label = x {
                        if barrier_passed {
                            break
                        }
                    }
                    return Some(x)
                }
                if bp { barrier_passed = bp; }
            }
            None
        }
    };
}

#[macro_export]
macro_rules! strip_alias_get {
    ($l:expr, $b:expr) => {
        {
            let mut x = $b.get(&$l);
            while let Some(Bind::Alias(l)) = x {
                x = $b.get(l);
                $l = l.clone();
            }
            x
        }
    };
}

#[derive(Debug, Default)]
struct PromisedValues<'a> {
    types: Vec<SharedValue<'a>>,
    functions: Vec<SharedValue<'a>>
}

macro_rules! gen_bindings_promised_operations {
    ($push:ident, $get:ident, $move:ident, $x:tt) => {
        pub fn $push (&mut self, init: SharedValue<'a>) { self.promised_values.$x.push(init); }
        pub fn $get (&self) -> &Vec<SharedValue<'a>> { &self.promised_values.$x }
        pub fn $move (&mut self) -> Vec<SharedValue<'a>> { mem::take(&mut self.promised_values.$x) }
    };
}

#[derive(Debug, Default)]
pub struct Bindings<'a> {
    content: Vec<HashMap<String, Bind<'a>>>,
    /// Scope context
    context: Vec<Context>,
    global_prefix: Vec<String>,
    current_file: PathBuf,
    initial_code: &'a str,
    // function_pointers: Vec<SharedValue<'a>>,
    promised_values: PromisedValues<'a>,
    link: HashSet<&'a str>,
    target: Target,
    compileable: bool,
    // type_pointers: Vec<usize>
}
impl<'a> Bindings<'a> {
    pub fn new (initial_code: &'a str, path: PathBuf, target: Option<Target>) -> Self {
        let mut s = Self::default();
        s.content.push(HashMap::new());
        s.initial_code = initial_code;
        s.current_file = path;
        s.compileable = true;
        if let Some(target) = target { s.target = target; }
        s
    }

    pub fn get_at_current (&self, name: &str) -> Option<&Bind<'a>> { self.content.last().unwrap().get(name) }
    
    pub fn get_at (&self, at: usize, name: &str) -> Option<&Bind<'a>> { self.content[at].get(name) }
    pub fn get_mut_at (&mut self, at: usize, name: &str) -> Option<&mut Bind<'a>> { self.content[at].get_mut(name) }

    // pub fn get_noalias (&self, mut name: String) -> Option<&Bind<'a>> {
    //     while let Some(Bind::Alias(x)) = self.get(&name) {
    //         name = x.clone();
    //     }
    //     self.get(&name)
    // }
    // pub fn get_noalias_mut (&mut self, mut name: String) -> Option<&mut Bind<'a>> {
    //     while let Some(Bind::Alias(x)) = self.get_mut(&name) {
    //         name = x.clone();
    //     }
    //     self.get_mut(&name)
    // }

    pub fn get (&self, name: &str) -> Option<&Bind<'a>> {
        bget!(self, name, false)
    }
    pub fn get_mut (&mut self, name: &str) -> Option<&mut Bind<'a>> {
        bget!(self, name, true)
    }

    pub fn join_scope (&mut self) {
        self.content.push(HashMap::new());
    }
    pub fn push_scope (&mut self, scope: HashMap<String, Bind<'a>>) {
        self.content.push(scope);
    }

    pub fn leave_scope (&mut self) {
        self.content.pop();
    }
    pub fn pop_scope (&mut self) -> HashMap<String, Bind<'a>> {
        self.content.pop().unwrap()
    }

    // pub fn insert (&mut self, name: &'a str, value: Bind<'a>) { self.content.last_mut().unwrap().insert(name, value); }
    pub fn insert (&mut self, name: &str, value: Bind<'a>) -> Option<Bind<'a>> { self.content.last_mut().unwrap().insert(name.into(), value) }
    pub fn global_insert (&mut self, name: &str, value: Bind<'a>) -> Option<Bind<'a>> {
        let gname = self.global_name(name);
        self.content.last_mut().unwrap().insert(name.into(), Bind::Alias(gname.clone()));
        self.content[0].insert(gname, value)
    }
    pub fn root_insert (&mut self, name: &str, value: Bind<'a>) -> Option<Bind<'a>> { self.content.get_mut(0).unwrap().insert(name.into(), value) }

    // pub fn insert_in (&mut self, scope: usize, name: &'a str, value: Bind<'a>) { self.content[scope].insert(name, value); }
    // pub fn insert_in_parent (&mut self, name: &str, value: Bind<'a>) { let last = self.content.len()-1; self.content[last-1].insert(name.into(), value); }

    // context

    pub fn push_context (&mut self, context: Context) { self.context.push(context); }
    // pub fn get_context (&self, x: Context) -> Option<&Context> { self.context.iter().rev().find(|y| **y == x) }
    pub fn get_context_noval (&self, x: &Context) -> Option<&Context> {
        self.context.iter().rev().find(
            |y| std::mem::discriminant(*y) == std::mem::discriminant(x)
        )
    }
    // pub fn get_last_context (&self) -> Option<&Context> { self.context.last() }
    pub fn pop_context (&mut self) -> Option<Context> { self.context.pop() }
    pub fn move_context (&mut self) -> Vec<Context> { mem::take(&mut self.context) }
    pub fn switch_context (&mut self, x: Vec<Context>) { let _ = mem::replace(&mut self.context, x); }

    pub fn context_scope (&mut self, context: Context, scope: impl FnOnce(&mut Self) -> ()) {
        self.join_scope();
        let upper_context = if let Context::Fn(_) = context { Some(self.move_context()) } else { None };
        self.push_context(context);
        // self.insert(MARK_BARRIER, Bind::Mark);

        scope(self);

        self.pop_context();
        if let Some(x) = upper_context { self.switch_context(x); }
        self.leave_scope();
    }

    pub fn push_global_prefix (&mut self, prefix: String) {
        self.global_prefix.push(prefix);
    }
    pub fn pop_global_prefix (&mut self) -> String {
        self.global_prefix.pop().unwrap()
    }
    pub fn get_global_prefix (&self) -> String {
        let x = self.global_prefix.join("::");
        if x.len() > 0 { format!("{x}::") }
        else { x }
    }
    pub fn global_name (&self, s: &str) -> String {
        format!("{}{s}", self.get_global_prefix())
    }

    // pub fn push_function (&mut self, init: SharedValue<'a>) { self.function_pointers.push(init); }
    // pub fn get_functions (&self) -> &Vec<SharedValue<'a>> { &self.function_pointers }
    // pub fn move_functions (&mut self) -> Vec<SharedValue<'a>> { mem::take(&mut self.function_pointers) }

    gen_bindings_promised_operations!(push_function, get_functions, move_functions, functions);
    gen_bindings_promised_operations!(push_type, get_types, move_types, types);

    pub fn link (&mut self, k: &'a str) { self.link.insert(k); }
    pub fn move_links (self) -> HashSet<&'a str> { self.link }

    pub fn target_ptr_bits (&self) -> u8 { self.target.ptr_bits }

    pub fn gentle_error (&mut self, token: &Token, message: &str) {
        let Token { col, line, value, .. } = token;
        parsing_error_message(self, message, *line, *col);
        let lines: Vec<&str> = self.get_initial_code().lines().collect();
        point(lines, *line, *col, value.chars().count(), "[91m");
        if self.compileable { self.compileable = false; }
    }
    pub fn gentle_error_range (&mut self, tokens: &[Token], message: &str) {
        let f = &tokens[0];
        parsing_error_message(self, message, f.line, f.col);
        let lines: Vec<&str> = self.get_initial_code().lines().collect();
        point_range(tokens, lines, "[91m");
        if self.compileable { self.compileable = false; }
    }

    pub fn is_compileable (&self) -> bool { self.compileable }
    pub fn get_initial_code (&self) -> &'a str { self.initial_code }
    pub fn get_current_file_path (&self) -> &PathBuf { &self.current_file }
}

pub const MARK_BARRIER: &str = "@barrier";
// pub const RESERVED_LOCALS: &str = "@reserved_locals";