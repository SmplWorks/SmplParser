#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Literal(u8),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action<T> {
    /// Immediately advance the cursor and return T.
    Return(T),

    /// If next iteration returns None, return T without advancing the cursor.
    Request(T),

    /// If the next iteration returns None, return None without advancing the cursor.
    Require,
}

pub struct TokenScanner {
    cursor : usize,
    toks : Vec<Token>,
}

impl TokenScanner {
    pub fn new(toks : Vec<Token>) -> Self {
        Self { cursor: 0, toks }
    }

    pub fn is_done(&mut self) -> bool {
        self.cursor == self.toks.len()
    }

    pub fn peek(&mut self) -> Option<&Token> {
        self.toks.get(self.cursor)
    }

    pub fn pop(&mut self) -> Option<&Token> {
        let tok = self.toks.get(self.cursor)?;
        self.cursor += 1;
        Some(tok)
    }

    pub fn scan<T>(&mut self, cb : impl Fn(&[&Token]) -> Option<Action<T>>) -> Result<Option<T>, &'static str> {
        let mut sequence = Vec::new();
        let mut request = None;
        let mut require = false;

        loop {
            let Some(target) = self.toks.get(self.cursor) else {
                break if require { Err("EOF") } else { Ok(request) }
            };

            sequence.push(target);
            let Some(action) = cb(&sequence[..]) else {
                break if require { Err("TODO: Error") } else { Ok(request) }
            };
            self.cursor += 1;

            match action {
                Action::Return(res) => break Ok(Some(res)),
                Action::Request(res) => {
                    require = false;
                    request = Some(res);
                },
                Action::Require => require = true,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum Expr {
        Nop,
        MovC2R(u8, String),
    }

    #[test]
    fn full() {
        let mut scanner = TokenScanner::new(vec![
            Token::Ident("nop".to_string()),
            Token::Ident("mov".to_string()),
            Token::Ident("mov".to_string()),
            Token::Literal(3),
            Token::Ident("r0".to_string()),
        ]);

        let expr = scanner.scan(|toks| match toks {
            [Token::Ident(op)] if op == "nop" => Some(Action::Return(Expr::Nop)),

            [Token::Ident(op)] if op == "mov" => Some(Action::Require),
            [Token::Ident(op), Token::Literal(_)] if op == "mov" => Some(Action::Require),
            [Token::Ident(op), Token::Literal(value), Token::Ident(reg)] if op == "mov" => Some(Action::Return(Expr::MovC2R(*value, reg.clone()))),

            _ => None,
        });

        todo!("{expr:?}")
    }
}
