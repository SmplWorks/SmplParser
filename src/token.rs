use crate::{Scanner, ScannerAction};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Punct(char),
    Comment(String),

    String(String),
    Char(char),
    Number(i64),
}

fn skip_whitespace(scanner : &mut Scanner<char>) {
    scanner.take_while(|c| c.is_whitespace());
}

fn match_comment(scanner : &mut Scanner<char>) -> Option<Token> {
    scanner.take(|c| *c == '/')?;

    if scanner.take(|c| *c == '/').is_some() {
        let comment = scanner.take_while(|c| !(*c == '\n' || *c == '\r'));
        Some(Token::Comment(comment.into_iter().collect()))
    } else if scanner.take(|c| *c == '*').is_some() {
        scanner.scan(|chars| match chars {
            ['*', '/'] => ScannerAction::Return(Token::Comment("".to_string())),
            [comment @ .., '*', '/'] => ScannerAction::Return(Token::Comment(comment.iter().collect())),
            _ => ScannerAction::Require,
        }).unwrap() // TODO: Handle
    } else {
        Some(Token::Punct('/'))
    }
}

fn match_identifier(scanner : &mut Scanner<char>) -> Option<Token> {
    scanner.test(|c| c.is_alphabetic() || *c == '_')
        .then(|| scanner.take_while(|c| c.is_alphanumeric() || *c == '_').iter().collect())
        .map(|ident| Token::Ident(ident))
}

fn match_number(scanner : &mut Scanner<char>) -> Option<Token> {
    if scanner.test(|c| c.is_ascii_digit() || *c == '-') { // TODO: is_numeric?
        scanner.scan(|chars| match chars {
            ['-'] => ScannerAction::Request(Token::Punct('-')),
            ['-', ..] if chars.iter().skip(1).all(|c| c.is_digit(10))
                => ScannerAction::Request(Token::Number(
                    -chars.iter().skip(1).collect::<String>().parse::<i64>().unwrap()
                )),
            ['0'] => ScannerAction::Request(Token::Number(0)),

            ['0', 'x'] => ScannerAction::Require,
            ['0', 'x', ..] if chars.iter().skip(2).all(|c| c.is_digit(16))
                => ScannerAction::Request(Token::Number(
                    i64::from_str_radix(&chars.iter().skip(2).collect::<String>(), 16).unwrap()
                )),

            ['0', 'o'] => ScannerAction::Require,
            ['0', 'o', ..] if chars.iter().skip(2).all(|c| c.is_digit(8))
                => ScannerAction::Request(Token::Number(
                    i64::from_str_radix(&chars.iter().skip(2).collect::<String>(), 8).unwrap()
                )),

            ['0', 'b'] => ScannerAction::Require,
            ['0', 'b', ..] if chars.iter().skip(2).all(|c| c.is_digit(2))
                => ScannerAction::Request(Token::Number(
                    i64::from_str_radix(&chars.iter().skip(2).collect::<String>(), 2).unwrap()
                )),

            _ if chars.iter().all(|c| c.is_digit(10))
                => ScannerAction::Request(Token::Number(chars.iter().collect::<String>().parse().unwrap())),

            _ => ScannerAction::None,
        }).unwrap() // TODO: Handle
    } else { None }
}


fn match_string(scanner : &mut Scanner<char>) -> Option<Token> {
    if scanner.take(|c| *c == '"').is_some() {
        let mut s = String::new();
        while let Some(c) = scanner.pop() {
            if c == '"' {
                break;
            } else if c == '\\' {
                let Some(c) = scanner.pop() else { panic!() }; // TODO: Handle
                match c {
                    '"' => s.push(c),
                    _ => panic!("Unknown control sequence: '{c}'"),
                }
            } else {
                s.push(c)
            }
        }

        Some(Token::String(s))
    } else { None }
}

fn match_char(scanner : &mut Scanner<char>) -> Option<Token> {
    if scanner.take(|c| *c == '\'').is_some() {
        let mut c = scanner.pop().unwrap(); // TODO: Handle
        if c == '\\' {
            c = scanner.pop().unwrap();
            match c {
                '\'' => (),
                _ => panic!("Unknown control sequence: '{c}'"),
            }
        }

        if scanner.take(|c| *c == '\'').is_none() {
            panic!("Unclosed character")
        }
        
        Some(Token::Char(c))
    } else { None }
}

fn match_punct(scanner : &mut Scanner<char>) -> Option<Token> {
    Some(Token::Punct(scanner.pop()?))
}

fn get_tok(scanner : &mut Scanner<char>) -> Option<Token> {
    skip_whitespace(scanner);

    match_identifier(scanner)
    .or_else(|| match_comment(scanner))
    .or_else(|| match_string(scanner))
    .or_else(|| match_number(scanner))
    .or_else(|| match_char(scanner))
    .or_else(|| match_punct(scanner))
}

fn tokenize_scanner(scanner : &mut Scanner<char>) -> Vec<Token> {
    let mut toks = Vec::new();
    while let Some(tok) = get_tok(scanner) {
        toks.push(tok);
    }
    toks
}

pub fn tokenize(code : &str) -> Vec<Token> {
    let mut scanner = Scanner::new(code.chars().collect());
    tokenize_scanner(&mut scanner)
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
    fn string() {
        let code = r#""A string" "\"A quoted string\"""#;
        let toks = tokenize(code);
        assert_eq!(toks, vec![
            Token::String("A string".to_string()),
            Token::String("\"A quoted string\"".to_string()),
        ]);
    }

    #[test]
    fn number() {
        let code = "0 0x0 0o0 0b0 62263 -62263 0xF337 0o171467 0b1111001100110111";
        let toks = tokenize(code);
        assert_eq!(toks, vec![
            Token::Number(0),
            Token::Number(0x0),
            Token::Number(0o0),
            Token::Number(0b0),
            Token::Number(62263),
            Token::Number(-62263),
            Token::Number(0xF337),
            Token::Number(0o171467),
            Token::Number(0b1111001100110111),
        ]);
    }

    #[test]
    fn character() {
        let code = "'0' '\''";
        let toks = tokenize(code);
        assert_eq!(toks, vec![
            Token::Char('0'),
            Token::Char('\''),
        ]);
    }

    #[test]
    fn comment() {
        let code = "// 0 1 asd\n/* 0 1 *\n * / asd */";
        let toks = tokenize(code);
        assert_eq!(toks, vec![
            Token::Comment(" 0 1 asd".to_string()),
            Token::Comment(" 0 1 *\n * / asd ".to_string()),
        ]);
    }
}
