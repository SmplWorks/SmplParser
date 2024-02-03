use crate::{Scanner, ScannerAction};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Literal(i64),
    Punct(char),
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

fn accept_number(c : &char) -> bool {
    c.is_ascii_digit() || *c == '-' // TODO: is_numeric?
}

fn get_number(scanner : &mut Scanner<char>) -> Result<Option<Token>, &'static str> {
    scanner.scan(|chars| match chars {
        ['-'] => Some(ScannerAction::Request(Token::Punct('-'))),
        ['-', ..] if chars.iter().skip(1).all(|c| c.is_digit(10))
            => Some(ScannerAction::Request(Token::Literal(
                -chars.iter().skip(1).map(|c| **c).collect::<String>().parse::<i64>().unwrap()
            ))),
        ['0'] => Some(ScannerAction::Request(Token::Literal(0))),

        ['0', 'x'] => Some(ScannerAction::Require),
        ['0', 'x', ..] if chars.iter().skip(2).all(|c| c.is_digit(16))
            => Some(ScannerAction::Request(Token::Literal(
                i64::from_str_radix(&chars.iter().skip(2).map(|c| **c).collect::<String>(), 16).unwrap()
            ))),

        ['0', 'o'] => Some(ScannerAction::Require),
        ['0', 'o', ..] if chars.iter().skip(2).all(|c| c.is_digit(8))
            => Some(ScannerAction::Request(Token::Literal(
                i64::from_str_radix(&chars.iter().skip(2).map(|c| **c).collect::<String>(), 8).unwrap()
            ))),

        ['0', 'b'] => Some(ScannerAction::Require),
        ['0', 'b', ..] if chars.iter().skip(2).all(|c| c.is_digit(2))
            => Some(ScannerAction::Request(Token::Literal(
                i64::from_str_radix(&chars.iter().skip(2).map(|c| **c).collect::<String>(), 2).unwrap()
            ))),

        _ if chars.iter().all(|c| c.is_digit(10))
            => Some(ScannerAction::Request(Token::Literal(chars.iter().map(|c| **c).collect::<String>().parse().unwrap()))),

        _ => None,
    })
}

fn matches(scanner : &mut Scanner<char>, accept : impl FnOnce(&char) -> bool, get : impl Fn(&char) -> bool) -> Option<String> {
    scanner.test(accept).then(|| scanner.take_while(get).iter().map(|c| **c).collect())
}

fn get_tok(scanner : &mut Scanner<char>) -> Option<Token> {
    skip_whitespace(scanner);

    if let Some(ident) = matches(scanner, accept_identifier, get_identifier) {
        return Some(Token::Ident(ident))
    }

    if scanner.test(accept_number) {
        return get_number(scanner).unwrap() // TODO: Handle error
    }

    None
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

    #[test]
    fn literal() {
        let code = "0 0x0 0o0 0b0 62263 -62263 0xF337 0o171467 0b1111001100110111";
        let toks = tokenize(code);
        assert_eq!(toks, vec![
            Token::Literal(0),
            Token::Literal(0x0),
            Token::Literal(0o0),
            Token::Literal(0b0),
            Token::Literal(62263),
            Token::Literal(-62263),
            Token::Literal(0xF337),
            Token::Literal(0o171467),
            Token::Literal(0b1111001100110111),
        ]);
    }
}
