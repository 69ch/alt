use super::TokenKind;

// pub type RuleOutput = Option<(String, TokenKind)>;
pub type RuleOutput = (Option<(String, TokenKind)>, usize);
pub type Rule = fn (&[char]) -> RuleOutput;

fn string (code: &[char]) -> RuleOutput {
    if code[0] != '\"' { return (None, 0) }
    let mut esc = false;
    for (index, i) in code.iter().skip(1).enumerate() {
        if *i == '\"' && !esc {
            let l = code[0..=index+1].iter().collect::<String>();
            return (Some((l, TokenKind::String)), index+2)
        }
        else if *i == '\\' { esc = true; }
        else if esc { esc = false; }
    }
    (None, 0)
}

fn number (code: &[char]) -> RuleOutput {
    let mut typ = TokenKind::Int;
    // && !(code[0] == '-' && code.get(1).is_some_and(|x| x.is_digit(10)))
    if !code[0].is_digit(10) {
        return (None, 0)
    }

    let mut index = 1;
    while index < code.len() {
        if code[index] == '.' && typ != TokenKind::Float && code.get(index+1).is_some_and(|x| x.is_digit(10)) {
            typ = TokenKind::Float;
        }
        else if !code[index].is_numeric() { break }
        index += 1;
    }

    (Some((code[0..index].iter().collect::<String>(), typ)), index)
}

fn word (code: &[char]) -> RuleOutput {
    let mut t = TokenKind::Word;
    if code[0] == '#' { t = TokenKind::Meta; }
    else if !code[0].is_alphabetic() && code[0] != '_' { return (None, 0) }

    let mut index = 1;
    while index < code.len() {
        // if code.get(index..=index+1).is_some_and(|x| x.iter().collect::<String>() == "::") {
        //     index += 2;
        // }
        // else
        if !code[index].is_alphanumeric() && code[index] != '_' {
            break
        }
        index += 1;
    }

    (Some((code[0..index].iter().collect::<String>(), t)), index)
}

fn symbol (code: &[char]) -> RuleOutput {
    if code.len() > 1 {
        let x = code[0..=1].iter().collect::<String>();
        match x.as_str() {
            "&&" | "||" | ">=" | "<=" | "!=" | "==" => return (Some((x, TokenKind::Logical)), 2),
            "//" => { return (None, code.iter().position(|x| *x == '\n').unwrap_or(code.len())) },
            ">>" | "<<" => return (Some((x, TokenKind::Bitwise)), 2),
            "+=" | "-=" | "*=" | "/=" | "%=" => return (Some((x, TokenKind::Assign)), 2),
            ".*" | "::" => return (Some((x, TokenKind::Special)), 2),
            _ => {}
        }
    }

    match code[0] {
        '=' => return (Some((code[0].to_string(), TokenKind::Assign)), 1),
        '+' | '-' | '*' | '/' | '%' => return (Some((code[0].to_string(), TokenKind::Arithmetic)), 1),
        '(' | ')' | '[' | ']' | '{' | '}' => return (Some((code[0].to_string(), TokenKind::Brackets)), 1),
        '>' | '<' | '!' => return (Some((code[0].to_string(), TokenKind::Logical)), 1),
        '&' | '^' | '|' => return (Some((code[0].to_string(), TokenKind::Bitwise)), 1),
        '~' => return (Some((code[0].to_string(), TokenKind::Special)), 1), // TODO REMOVE
        '@' => return (Some((code[0].to_string(), TokenKind::LabelSymbol)), 1),
        ',' | ';' => return (Some((code[0].to_string(), TokenKind::Punctuation)), 1),
        '.' => return (Some((code[0].to_string(), TokenKind::In)), 1),
        _ => {}
    }

    (None, 0)
}

pub fn rules () -> [Rule; 4] {
    [
        string,
        number,
        word,
        symbol
    ]
}