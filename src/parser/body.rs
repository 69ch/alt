use crate::lexer::Token;

pub fn parse_pair_symbols (tokens: &[Token], pair: (&str, &str)) -> Option<usize> {
    let mut joined = 1;
    if tokens.get(0)?.value != pair.0 { return None }

    let mut i = 1;
    while i < tokens.len() {
        let token = &tokens[i];
        i += 1;
        if token.value == pair.0 { joined += 1; }
        else if token.value == pair.1 {
            if joined <= 1 {
                return Some(i)
            }
            else {
                joined -= 1;
            }
        }
    }

    None
}