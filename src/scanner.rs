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

pub struct Scanner {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: u32,
}

impl Scanner {
    pub fn new(source: &String) -> Self {
        Self {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
        }
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
                },
                None => break
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
