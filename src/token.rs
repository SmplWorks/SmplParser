use crate::Scanner;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Literal(i64),
}

fn skip_whitespace(scanner : &mut Scanner<char>) {
    scanner.take_while(|c| c.is_whitespace());
}

fn accept_identifier(c : &char) -> bool {
    c.is_alphabetic() || *c == '_'
}

fn get_identifier(c : &char) -> bool {
    c.is_alphanumeric() || *c == '_'
}

fn get_tok(scanner : &mut Scanner<char>) -> Option<Token> {
    skip_whitespace(scanner);
    if scanner.test(accept_identifier) {
        let ident = scanner.take_while(get_identifier).iter().map(|c| **c).collect();
        Some(Token::Ident(ident))
    } else {
        None
    }
}

pub fn tokenize(code : &str) -> Vec<Token> {
    let mut toks = Vec::new();

    let mut scanner = Scanner::new(code.chars().collect());
    while let Some(tok) = get_tok(&mut scanner) {
        toks.push(tok);
    }

    toks
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ident() {
        let code = "nop _nop no_p nop_ no1p nop1";
        let toks = tokenize(code);
        assert_eq!(toks, vec![
            Token::Ident("nop".to_string()),
            Token::Ident("_nop".to_string()),
            Token::Ident("no_p".to_string()),
            Token::Ident("nop_".to_string()),
            Token::Ident("no1p".to_string()),
            Token::Ident("nop1".to_string()),
        ]);
    }
}
