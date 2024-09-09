use crate::token::*;

pub struct Scanner {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: u32,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            source: Vec::default(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn reset(&mut self, source: &String) {
        self.source = source.chars().collect();
        self.start = 0;
        self.current = 0;
        self.line = 1;
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_white_space();
        self.start = self.current;

        match self.peek() {
            None => self.make_token(TokenType::Eof),
            Some(c) => {
                self.advance();
                match c {
                    '(' => self.make_token(TokenType::LeftParen),
                    ')' => self.make_token(TokenType::RightParen),
                    '{' => self.make_token(TokenType::LeftBrace),
                    '}' => self.make_token(TokenType::RightBrace),
                    ';' => self.make_token(TokenType::Semicolon),
                    ',' => self.make_token(TokenType::Comma),
                    '.' => self.make_token(TokenType::Dot),
                    '+' => self.make_token(TokenType::Plus),
                    '-' => self.make_token(TokenType::Minus),
                    '*' => self.make_token(TokenType::Star),
                    '/' => self.make_token(TokenType::Slash),
                    '!' => match self.r#match('=') {
                        true => self.make_token(TokenType::BangEqual),
                        false => self.make_token(TokenType::Bang),
                    },
                    '=' => match self.r#match('=') {
                        true => self.make_token(TokenType::EqualEqual),
                        false => self.make_token(TokenType::Equal),
                    },
                    '<' => match self.r#match('=') {
                        true => self.make_token(TokenType::LessEqual),
                        false => self.make_token(TokenType::Less),
                    },
                    '>' => match self.r#match('=') {
                        true => self.make_token(TokenType::GreaterEqual),
                        false => self.make_token(TokenType::Greater),
                    },
                    '"' => self.string_token(),
                    '0'..='9' => self.number_token(),
                    'a'..='z' | 'A'..='Z' | '_' => self.identifier_token(),
                    _ => self.error_token("unexpected character"),
                }
            }
        }
    }

    fn skip_white_space(&mut self) {
        loop {
            match self.peek() {
                Some(c) => match c {
                    ' ' | '\t' | '\r' => {
                        self.advance();
                    }
                    '\n' => {
                        self.line += 1;
                        self.advance();
                    }
                    '/' => match self.peek_next() {
                        Some(pn) => match pn {
                            '/' => {
                                while self.peek().is_some() && self.peek().unwrap() != '\n' {
                                    self.advance();
                                }
                            }
                            _ => break,
                        },
                        None => break,
                    },
                    _ => break,
                },
                None => break,
            }
        }
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn peek(&self) -> Option<char> {
        match self.current < self.source.len() {
            true => Some(self.source[self.current]),
            false => None,
        }
    }

    fn peek_next(&self) -> Option<char> {
        match self.current + 1 < self.source.len() {
            true => Some(self.source[self.current + 1]),
            false => None,
        }
    }

    fn r#match(&mut self, match_char: char) -> bool {
        match self.peek() {
            Some(c) => {
                if c == match_char {
                    self.advance();
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    fn make_token(&mut self, token_type: TokenType) -> Token {
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        Token {
            r#type: token_type,
            lexeme: lexeme,
            line: self.line,
        }
    }

    fn error_token(&mut self, error_info: &str) -> Token {
        Token {
            r#type: TokenType::Error,
            lexeme: error_info.to_string(),
            line: self.line,
        }
    }

    fn string_token(&mut self) -> Token {
        loop {
            let peek_char = self.peek();
            match peek_char {
                Some(c) => match c {
                    '"' => break,
                    '\n' => {
                        self.line += 1;
                        self.advance();
                    }
                    _ => self.advance(),
                },
                None => break,
            }
        }
        match self.peek().is_none() {
            true => self.error_token("unterminated string"),
            false => {
                self.advance();
                self.make_token(TokenType::String)
            }
        }
    }

    fn number_token(&mut self) -> Token {
        while self.peek().is_some() && self.peek().unwrap().is_ascii_digit() {
            self.advance();
        }
        if self.peek().is_some()
            && self.peek().unwrap() == '.'
            && self.peek_next().is_some()
            && self.peek_next().unwrap().is_ascii_digit()
        {
            self.current += 2;
            while self.peek().is_some() && self.peek().unwrap().is_ascii_digit() {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier_token(&mut self) -> Token {
        loop {
            let peek_char = self.peek();
            match peek_char {
                Some(c) => {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        self.advance();
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }
        let keyword: String = self.source[self.start..self.current].iter().collect();
        self.make_token(match keyword.as_str() {
            "var" => TokenType::Var,
            "nil" => TokenType::Nil,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "class" => TokenType::Class,
            "this" => TokenType::This,
            "super" => TokenType::Super,
            _ => TokenType::Identifier,
        })
    }
}
