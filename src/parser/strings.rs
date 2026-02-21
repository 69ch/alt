pub trait EscapeGen {
    fn escape_escaping (self) -> String;
}


// todo: maybe don't do this actually
impl EscapeGen for String {
    fn escape_escaping (self) -> String {
        self[1..self.len()-1].replace("\\n", "\n")
    }
}