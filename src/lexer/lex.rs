use super::rules::rules;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    String, Int, Float, Arithmetic, Logical, Bitwise, Brackets, Word,
    Assign, LabelSymbol, Punctuation, Special, In,
    Meta
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub typ: TokenKind,
    pub value: String,
    pub col: usize, pub line: usize
}

pub trait CollectChars { fn collect_chars (&self) -> Vec<char>; }

macro_rules! stdimplchars {
    ($($x:ty),+) => {
        $(
            impl CollectChars for $x {
                fn collect_chars (&self) -> Vec<char> {
                    self.chars().collect()
                }
            }
        )+
    };
}

stdimplchars!(&str, &String, String);

pub fn lex (code: impl CollectChars) -> Vec<Token> {
    let rules = rules();
    let code = code.collect_chars();
    
    let mut tokens: Vec<Token> = vec![];

    let mut index = 0;
    let (mut col, mut line) = (1, 1);
    'x: while index < code.len() {
        if code[index] == '\n' { line += 1; col = 0; }
        else if !code[index].is_whitespace() {
            for i in &rules {
                let val_jmp = i(&code[index..]);
                if let (Some((value, typ)), off) = val_jmp {
                    tokens.push(Token { typ, value, col, line });
                    index += off; col += off;
                    continue 'x
                }
                else if val_jmp.1 > 0 {
                    index += val_jmp.1; col += val_jmp.1;
                    continue 'x
                }
            }
        }
        index += 1; col += 1;
    }

    tokens
}