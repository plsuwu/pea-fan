pub struct Lexer<'a> {
    pub input: &'a str,
    pub pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    pub fn next(&mut self) -> Option<char> {
        let ch = self.peek_char();
        if ch.is_some() {
            self.pos += 1;
        }

        ch
    }

    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }

    pub fn next_word(&mut self) -> Option<&'a str> {
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                break;
            }

            self.next();
        }

        if start == self.pos {
            None
        } else {
            Some(&self.input[start..self.pos])
        }
    }

    pub fn next_until(&mut self, delims: &[char]) -> Option<&'a str> {
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if delims.contains(&ch) {
                break;
            }

            self.next();
        }

        if start == self.pos {
            None
        } else {
            Some(&self.input[start..self.pos])
        }
    }

    pub fn rest(&mut self) -> Option<&'a str> {
        if self.is_eof() {
            None
        } else {
            let result = &self.input[self.pos..];
            self.pos = self.input.len();
            Some(result)
        }
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}
