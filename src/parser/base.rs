#![allow(dead_code)]

/**
 * "@badge-info=;badges=broadcaster/1,twitch-recap-2023/1;client-nonce=00000000000000000000000000000000;color=#FFBEDF;display-name=plss;emotes=;first-msg=0;flags=;id=77ac96fb-34c4-4494-b4a2-83873aecb333;mod=0;returning-chatter=0;room-id=103033809;subscriber=0;tmi-sent-ts=1749208156695;turbo=0;user-id=103033809;user-type= :plss!plss@plss.tmi.twitch.tv PRIVMSG #plss :eeeeeeeee\r\n"
 *
 * "PING :tmi.twitch.tv\r\n"
 */

pub enum TokenType {

}


#[derive(Debug)]
pub struct Lexer {
    stream: InputStream,
    current: Option<char>,
}

impl Lexer {
    // pub fn new(input: &str) -> Self {
    //
    // }

    pub fn is_digit(&self) -> bool {
        self.current.is_some_and(|curr| curr >= '0' && curr <= '9')
    }

    pub fn is_whitespace(&self) -> bool {
        self.current.is_some_and(|curr| curr == ' ')
    }

    pub fn read_while<F>(&self, f: F) 
    where 
        F: Fn(&Lexer) -> bool,
    {
        while self.current.is_some() && f(self) {
            
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub pos: usize,
    pub line: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            pos: 0,
            line: 1,
            col: 0,
        }
    }
}

#[derive(Debug)]
pub struct InputStream {
    pub cursor: Cursor,
    pub input: Vec<char>,
    // stream: Iter<'a, char>,
}

impl InputStream {
    pub fn new(input: &str) -> Self {
        let cursor = Cursor::new();
        let input = input.chars().collect::<Vec<_>>();

        Self { cursor, input }
    }

    pub fn next(&mut self) -> char {
        self.cursor.pos += 1;
        let ch = self.input[self.cursor.pos];

        if ch == '\n' {
            self.cursor.col = 0;
            self.cursor.line += 1;
        } else {
            self.cursor.col += 1;
        }

        ch
    }

    pub fn peek(&self) -> char {
        self.input[self.cursor.pos]
    }
}
