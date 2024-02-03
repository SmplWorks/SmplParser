use std::collections::VecDeque;

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
    toks : VecDeque<T>,
}

impl<T> Scanner<T> {
    pub fn new(toks : VecDeque<T>) -> Self {
        Self { toks }
    }

    pub fn is_done(&mut self) -> bool {
        self.toks.is_empty()
    }

    pub fn peek(&self) -> Option<&T> {
        self.toks.get(0)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.toks.pop_front()
    }

    pub fn transform<U>(&mut self, cb : impl FnOnce(&T) -> Option<U>) -> Option<U> {
        let tok = self.peek()?;
        let res = cb(tok)?;
        self.pop().unwrap();
        Some(res)
    }
    
    pub fn test(&self, cb : impl FnOnce(&T) -> bool) -> bool {
        self.peek().is_some_and(|tok| cb(tok))
    }

    pub fn take(&mut self, cb : impl FnOnce(&T) -> bool) -> Option<T> {
        self.test(cb).then(|| self.pop().unwrap())
    }

    pub fn take_while(&mut self, cb : impl Fn(&T) -> bool) -> Vec<T> {
        let mut res = Vec::new();
        while self.test(&cb) {
            res.push(self.pop().unwrap())
        }
        res
    }

    pub fn scan<U>(&mut self, cb : impl Fn(&[T]) -> Option<ScannerAction<U>>) -> Result<Option<U>, &'static str> {
        let mut sequence = Vec::new();
        let mut request = None;
        let mut require = false;

        loop {
            let Some(tok) = self.pop() else {
                break if require { Err("EOF") } else { Ok(request) }
            };

            sequence.push(tok);
            let Some(action) = cb(&sequence[..]) else {
                self.toks.push_front(sequence.pop().unwrap()); // Put it back
                break if require { Err("TODO: Error") } else { Ok(request) }
            };

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
