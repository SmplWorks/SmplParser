#[derive(Debug, Clone, PartialEq)]
pub enum ScannerAction<T> {
    /// Immediately advance the cursor and return T.
    Return(T),

    /// If next iteration returns None, return T without advancing the cursor.
    Request(T),

    /// If the next iteration returns None, return None without advancing the cursor.
    Require,
}

pub struct Scanner<T> {
    cursor : usize,
    toks : Vec<T>,
}

impl<T> Scanner<T> {
    pub fn new(toks : Vec<T>) -> Self {
        Self { cursor: 0, toks }
    }

    pub fn is_done(&mut self) -> bool {
        self.cursor == self.toks.len()
    }

    pub fn peek(&self) -> Option<&T> {
        self.toks.get(self.cursor)
    }

    pub fn pop(&mut self) -> Option<&T> {
        let tok = self.toks.get(self.cursor)?;
        self.cursor += 1;
        Some(tok)
    }

    pub fn transform<U>(&mut self, cb : impl FnOnce(&T) -> Option<U>) -> Option<U> {
        let tok = self.peek()?;
        let res = cb(tok)?;
        self.cursor += 1;
        Some(res)
    }
    
    pub fn test(&self, cb : impl FnOnce(&T) -> bool) -> bool {
        self.peek().is_some_and(|tok| cb(tok))
    }

    pub fn take(&mut self, cb : impl FnOnce(&T) -> bool) -> Option<&T> {
        self.test(cb).then_some(self.pop().unwrap())
    }

    pub fn take_while(&mut self, cb : impl Fn(&T) -> bool) -> Vec<&T> {
        let mut res = Vec::new();
        while self.test(&cb) {
            let tok = self.toks.get(self.cursor).unwrap();
            self.cursor += 1;
            res.push(tok);
        }

        res
    }

    pub fn scan<U>(&mut self, cb : impl Fn(&[&T]) -> Option<ScannerAction<U>>) -> Result<Option<U>, &'static str> {
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
                ScannerAction::Return(res) => break Ok(Some(res)),
                ScannerAction::Request(res) => {
                    require = false;
                    request = Some(res);
                },
                ScannerAction::Require => require = true,
            }
        }
    }
}
