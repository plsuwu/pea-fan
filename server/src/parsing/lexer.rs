/// Lexer state structure for tokenizing an IRC message input stream
pub struct Lexer<'a> {
    pub input: &'a str,
    pub byte_pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            byte_pos: 0,
        }
    }

    /// Checks current value of the input stream under the cursor without advancing its position
    pub fn peek_char(&self) -> Option<char> {
        self.input[self.byte_pos..].chars().next()
    }

    /// Advances the cursor position forward one element (if the next element is not the EOF),
    /// returning the value of the previous element
    ///
    /// In practice, this method is kind of functionally equivalent to popping and returning an
    /// element from the top of a queue
    pub fn next(&mut self) -> Option<char> {
        let ch = self.peek_char();
        if let Some(utf) = ch {
            self.byte_pos += utf.len_utf8();
            Some(utf)
        } else {
            None
        }
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
        let start = self.byte_pos;
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                break;
            }

            self.next();
        }

        if start == self.byte_pos {
            None
        } else {
            Some(&self.input[start..self.byte_pos])
        }
    }

    /// Consumes all consecutive characters until the character under the cursor is equal to a
    /// member in the `delims` array, returning the consumed characters to the caller
    pub fn next_until(&mut self, delims: &[char]) -> Option<&'a str> {
        let start = self.byte_pos;
        while let Some(ch) = self.peek_char() {
            if delims.contains(&ch) {
                break;
            }

            self.next();
        }

        if start == self.byte_pos {
            None
        } else {
            Some(&self.input[start..self.byte_pos])
        }
    }

    /// Consume the remaining input stream and return it as an array
    pub fn rest(&mut self) -> Option<&'a str> {
        if self.is_eof() {
            None
        } else {
            let result = &self.input[self.byte_pos..];
            self.byte_pos = self.input.len();
            Some(result)
        }
    }

    /// Determine if the cursor's position is the end of the input stream
    pub fn is_eof(&self) -> bool {
        self.byte_pos >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_utf8() {
        let test_input = "hello 🗣️ 123 🪱 world";
        let mut lexer = Lexer::new(test_input);

        assert_eq!(lexer.next(), Some('h'));
        assert_eq!(lexer.next(), Some('e'));
        assert_eq!(lexer.next(), Some('l'));
        assert_eq!(lexer.next(), Some('l'));
        assert_eq!(lexer.next(), Some('o'));
        assert_eq!(lexer.next(), Some(' '));
        assert_eq!(lexer.next(), Some('🗣'));
        assert_eq!(lexer.next(), Some('\u{fe0f}')); // variation selector

        let remaining = lexer.rest().unwrap();
        assert_eq!(remaining, " 123 🪱 world");
    }

    /// This tests for the error we get when we try to tokenize e.g. emoji:
    ///
    /// ```ignore
    /// thread 'tokio-runtime-worker' panicked at src/parser/lexer.rs:78:29:
    /// byte index 328 is not a char boundary; it is inside '🪱' (bytes 327..331)
    /// ```
    #[test]
    fn test_rest_utf8_at_boundary() {
        let test_input = "abc🪱def";
        let mut lexer = Lexer::new(test_input);

        assert_eq!(lexer.next(), Some('a'));
        assert_eq!(lexer.next(), Some('b'));
        assert_eq!(lexer.next(), Some('c'));

        let remaining = lexer.rest().unwrap();
        assert_eq!(remaining, "🪱def");
    }

    #[test]
    fn test_utf8_parsing() {
        let test_input = "hello 🗣️ 123 🪱";
        let expected_chars: Vec<char> = test_input.chars().collect();

        let mut lexer = Lexer::new(test_input);
        let mut actual_chars = Vec::new();
        while let Some(ch) = lexer.next() {
            actual_chars.push(ch);
        }

        assert_eq!(actual_chars, expected_chars); // correct characters have been read

        let reconstructed: String = actual_chars.into_iter().collect();
        assert_eq!(&reconstructed, test_input); // correct overall string has been read
    }
}

