use std::panic;

use crate::lexer::Token;

use super::bindings::Bindings;

pub fn point (lines: Vec<&str>, line: usize, col: usize, amount: usize, color: &str) {
    println!("{}\x1b{color}", lines[line-1]);
    println!("{}{}\x1b[0m", " ".repeat(col-1), "^".repeat(amount));
}

pub fn point_range (tokens: &[Token], lines: Vec<&str>, color: &str) {
    let mut i = 0;
    let mut line = 0;
    let mut il = 0;
    let mut joined = false;
    while i < tokens.len() {
        let t = &tokens[i];
        if line != t.line {
            il = i;
            line = t.line;
            if joined == true { joined = false; print!("\x1b[0m\n"); }
            println!("{}\x1b{color}", lines[line-1]);
        }
        let s = &tokens[il];
        let e = &tokens[i];
        if joined == false { print!("{}", " ".repeat(s.col-1)); joined = true; }
        let nearest = if i > 0 { tokens.get(i-1) } else { None };
        let space = if let Some(nearest) = nearest {
            if nearest.line == e.line { e.col - nearest.col - nearest.value.chars().count() }
            else { 0 }
        }
        else { 0 };
        print!("{}", "^".repeat(e.value.chars().count() + space));
        i += 1;
    }

    print!("\x1b[0m\n");
}



pub fn parsing_error_message (bindings: &Bindings, message: &str, line: usize, col: usize) {
    let path = bindings.get_current_file_path();
    let link = format!("\x1B]8;;{}\x1B\\{}:{line}:{col}\x1B]8;;\x1B\\", path.to_str().unwrap(), path.as_path().file_name().unwrap().to_str().unwrap());
    println!("\x1b[91mError\x1b[0m at {link}: \x1b[1m{message}\x1b[0m");
}

pub fn error (token: &Token, bindings: &Bindings, message: &str) -> ! {
    // panic::set_hook(Box::new(|_| {}));
    let Token { col, line, value, .. } = token;
    parsing_error_message(bindings, message, *line, *col);
    let lines: Vec<&str> = bindings.get_initial_code().lines().collect();
    point(lines, *line, *col, value.chars().count(), "[91m");
    // exit(1)
    panic!()
}

// pub fn error_eots () -> ! {
//     parsing_error_message(&Bindings::default(), "Unexpected end of token stream somewhere in program", 0, 0);
//     panic!()
// }

pub fn error_range (tokens: &[Token], bindings: &Bindings, message: &str) -> ! {
    let f = &tokens[0];
    parsing_error_message(bindings, message, f.line, f.col);

    let lines: Vec<&str> = bindings.get_initial_code().lines().collect();
    point_range(tokens, lines, "[91m");

    // exit(1)
    panic!()
}

pub fn assert (a: bool, token: &Token, bindings: &Bindings, message: &str) {
    if !a { error(token, bindings, message) }
}

pub fn assert_range (a: bool, tokens: &[Token], bindings: &Bindings, message: &str) {
    if !a { error_range(tokens, bindings, message) }
}

pub fn err_expected_body (token: &Token, bindings: &Bindings, pair: (&str, &str)) -> ! {
    error(token, bindings, &format!("Expected body of shape '{}...{}'", pair.0, pair.1))
}