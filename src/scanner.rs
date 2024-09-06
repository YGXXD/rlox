#[derive(PartialEq, Clone)]
pub enum TokenType {
    // 单字符词法
    LeftParen = 0,
    RightParen = 1,
    LeftBrace = 2,
    RightBrace = 3,
    Comma = 4,
    Dot = 5,
    Minus = 6,
    Plus = 7,
    Semicolon = 8,
    Slash = 9,
    Star = 10,
    // 一或两字符词法
    Bang = 20,
    BangEqual = 21,
    Equal = 22,
    EqualEqual = 23,
    Greater = 24,
    GreaterEqual = 25,
    Less = 26,
    LessEqual = 27,
    // 字面量
    Identifier = 40,
    String = 41,
    Number = 42,
    // 关键字
    And = 60,
    Class = 61,
    Else = 62,
    False = 63,
    For = 64,
    Fun = 65,
    If = 66,
    Nil = 67,
    Or = 68,
    Print = 69,
    Return = 70,
    Super = 71,
    This = 72,
    True = 73,
    Var = 74,
    While = 75,
    // 特殊词
    Eof = 254,
    Error = 255,
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Eof
    }
}

impl ToString for TokenType {
    fn to_string(&self) -> String {
        match self {
            TokenType::LeftParen => "LeftParen".to_string(),
            TokenType::RightParen => "RightParen".to_string(),
            TokenType::LeftBrace => "LeftBrace".to_string(),
            TokenType::RightBrace => "RightBrace".to_string(),
            TokenType::Comma => "Comma".to_string(),
            TokenType::Dot => "Dot".to_string(),
            TokenType::Minus => "Minus".to_string(),
            TokenType::Plus => "Plus".to_string(),
            TokenType::Semicolon => "Semicolon".to_string(),
            TokenType::Slash => "Slash".to_string(),
            TokenType::Star => "Star".to_string(),
            TokenType::Bang => "Bang".to_string(),
            TokenType::BangEqual => "BangEqual".to_string(),
            TokenType::Equal => "Equal".to_string(),
            TokenType::EqualEqual => "EqualEqual".to_string(),
            TokenType::Greater => "Greater".to_string(),
            TokenType::GreaterEqual => "GreaterEqual".to_string(),
            TokenType::Less => "Less".to_string(),
            TokenType::LessEqual => "LessEqual".to_string(),
            TokenType::Identifier => "Identifier".to_string(),
            TokenType::String => "String".to_string(),
            TokenType::Number => "Number".to_string(),
            TokenType::And => "And".to_string(),
            TokenType::Class => "Class".to_string(),
            TokenType::Else => "Else".to_string(),
            TokenType::False => "False".to_string(),
            TokenType::For => "For".to_string(),
            TokenType::Fun => "Fun".to_string(),
            TokenType::If => "If".to_string(),
            TokenType::Nil => "Nil".to_string(),
            TokenType::Or => "Or".to_string(),
            TokenType::Print => "Print".to_string(),
            TokenType::Return => "Return".to_string(),
            TokenType::Super => "Super".to_string(),
            TokenType::This => "This".to_string(),
            TokenType::True => "True".to_string(),
            TokenType::Var => "Var".to_string(),
            TokenType::While => "While".to_string(),
            TokenType::Eof => "Eof".to_string(),
            TokenType::Error => "Error".to_string(),
        }
    }
}

#[derive(Default, Clone)]
pub struct Token {
    pub r#type: TokenType,
    pub lexeme: String,
    pub line: u32,
}

impl Token {
    pub fn send_error(&self, message: &str) {
        eprint!("[line {}] Error ", self.line);
        match self.r#type {
            TokenType::Eof => eprint!("at end"),
            TokenType::Error => eprint!("{}", self.lexeme),
            _ => eprint!("at {}", self.lexeme),
        }
        eprintln!(" : {}", message);
    }

    pub fn send_info(&self, message: &str) {
        print!("[line {}] Info ", self.line);
        match self.r#type {
            TokenType::Eof => print!("at end"),
            TokenType::Error => print!("{}", self.lexeme),
            _ => eprint!("at {}", self.lexeme),
        }
        println!(" : {}", message);
    }
}

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: u32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_white_space();
        self.start = self.current;

        match self.is_at_end() {
            true => self.make_token(TokenType::Eof),
            false => {
                let c: char = self.advance();
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
                ' ' | '\t' | '\r' => {
                    self.current = self.current + 1;
                }
                '\n' => {
                    self.line = self.line + 1;
                    self.current = self.current + 1;
                }
                '/' => match self.peek_next() == '/' {
                    true => {
                        while !self.is_at_end() && self.peek() != '\n' {
                            self.current = self.current + 1;
                        }
                    }
                    false => break,
                },
                _ => break,
            }
        }
    }

    fn advance(&mut self) -> char {
        self.current = self.current + 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn r#match(&mut self, match_char: char) -> bool {
        match self.is_at_end() {
            true => false,
            false => match self.source.chars().nth(self.current).unwrap() == match_char {
                true => {
                    self.current = self.current + 1;
                    true
                }
                false => false,
            },
        }
    }

    fn peek(&self) -> char {
        match self.source.chars().nth(self.current) {
            Some(c) => c,
            None => '\0',
        }
    }

    fn peek_next(&self) -> char {
        match self.source.chars().nth(self.current + 1) {
            Some(c) => c,
            None => '\0',
        }
    }

    fn make_token(&mut self, token_type: TokenType) -> Token {
        let lexeme: &str = &self.source[self.start..self.current];
        Token {
            r#type: token_type,
            lexeme: lexeme.to_string(),
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
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line = self.line + 1;
            }
            self.current = self.current + 1;
        }
        match self.is_at_end() {
            true => self.error_token("unterminated string"),
            false => {
                self.current = self.current + 1;
                self.make_token(TokenType::String)
            }
        }
    }

    fn number_token(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.current = self.current + 1;
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.current = self.current + 2;
        }
        while self.peek().is_ascii_digit() {
            self.current = self.current + 1;
        }
        self.make_token(TokenType::Number)
    }

    fn identifier_token(&mut self) -> Token {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.current = self.current + 1;
        }
        let keyword: &str = &self.source[self.start..self.current];
        self.make_token(match keyword {
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

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
