
/// Lexer state structure for tokenizing an IRC message input stream
pub struct Lexer<'a> {
    pub input: &'a str,
    pub pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }
    
    /// Checks current value of the input stream under the cursor without advancing its position
    pub fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }
    
    /// Advances the cursor position forward one element (if the next element is not the EOF),
    /// returning the value of the previous element
    ///
    /// In practice, this method is kind of functionally equivalent to popping and returning an
    /// element from the top of a queue
    pub fn next(&mut self) -> Option<char> {
        let ch = self.peek_char();
        if ch.is_some() {
            self.pos += 1;
        }

        ch
    }
    
    /// Consumes all consecutive whitespace characters, returning execution to the caller when a
    /// non-whitespace character is found under the cursor
    pub fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.next();
            } else {
                break;
            }
        }
    }
    
    /// Consumes all consecutive non-whitespace characters, returning the the consumed characters
    /// to the caller
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
    
    /// Consumes all consecutive characters until the character under the cursor is equal to a
    /// member in the `delims` array, returning the consumed characters to the caller
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
    
    /// Consume the remaining input stream and return it as an array
    pub fn rest(&mut self) -> Option<&'a str> {
        if self.is_eof() {
            None
        } else {
            let result = &self.input[self.pos..];
            self.pos = self.input.len();
            Some(result)
        }
    }
    
    /// Determine if the cursor's position is the end of the input stream
    pub fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}
