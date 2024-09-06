use crate::chunk::*;
use crate::scanner::*;
use crate::value::Value;

#[derive(Clone)]
enum Precedence {
    None = 0,
    Assignment = 1, // =
    Or = 2,         // or
    And = 3,        // and
    Equality = 4,   // == !=
    Comparison = 5, // < > <= >=
    Team = 6,       // + -
    Factor = 7,     // * /
    Unary = 8,      // ! -
    Call = 9,       // . ()
    Primary = 10,
}

impl From<Precedence> for u8 {
    fn from(value: Precedence) -> Self {
        value as u8
    }
}

impl From<u8> for Precedence {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Assignment,
            2 => Self::Or,
            3 => Self::And,
            4 => Self::Equality,
            5 => Self::Comparison,
            6 => Self::Team,
            7 => Self::Factor,
            8 => Self::Unary,
            9 => Self::Call,
            10 => Self::Primary,
            _ => unimplemented!("Invalid Precedence"),
        }
    }
}

impl From<TokenType> for Precedence {
    fn from(value: TokenType) -> Self {
        match value {
            TokenType::Plus | TokenType::Minus => Self::Team,
            TokenType::Star | TokenType::Slash => Self::Factor,
            TokenType::Number => Self::Assignment,
            _ => Self::None
        }
    }
}

pub struct Parser<'a, 'b> {
    pub had_error: bool,
    pub panic_mode: bool,
    pub current: Token,
    pub previous: Token,
    pub scanner: &'a mut Scanner<'b>,
    pub chunk: &'a mut Chunk
}

impl<'a, 'b> Parser<'a, 'b> {
    pub fn new(scanner: &'a mut Scanner<'b>, chunk: &'a mut Chunk) -> Self {
        Self {
            had_error: false,
            panic_mode: false,
            current: Token::default(),
            previous: Token::default(),
            scanner: scanner,
            chunk: chunk
        }
    }

    pub fn advance(&mut self) {
        self.previous = self.current.clone();
        loop {
            self.current = self.scanner.scan_token();
            match self.current.r#type {
                TokenType::Error => self.error_at_current("Paser error"),
                _ => break,
            }
        }
    }

    pub fn consume(&mut self, token_type: TokenType, message: &str) {
        match self.current.r#type == token_type {
            true => self.advance(),
            false => self.error_at_current(message),
        }
    }

    fn error_at_current(&mut self, message: &str) {
        match self.panic_mode {
            true => return,
            false => {
                self.had_error = true;
                self.panic_mode = true;
                self.current.send_error(message);
            }
        }
    }

    fn error_at_previous(&mut self, message: &str) {
        match self.panic_mode {
            true => return,
            false => {
                self.had_error = true;
                self.panic_mode = true;
                self.previous.send_error(message);
            }
        }
    }

    pub fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        self.chunk.write_code(OpCode::Constant.into(), self.previous.line);
        let value: Value = self.previous.lexeme.parse().expect("Failed to parse float");
        let idx = self.chunk.add_constant(value);
        match idx > 0xff {
            true => {
                eprintln!("Too many constants in one chunk.");
                self.chunk.write_code(0, self.previous.line);
            }
            false => self.chunk.write_code(idx as u8, self.previous.line),
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression");
    }

    fn unary(&mut self) {
        let token_type: TokenType = self.previous.r#type.clone();
        self.parse_precedence(Precedence::Unary);
        match token_type {
            TokenType::Minus => self.chunk.write_code(OpCode::Negate.into(), self.previous.line),
            _ => return,
        }
    }

    fn binary(&mut self) {
        let token_type: TokenType = self.previous.r#type.clone();
        let precedence: Precedence = self.previous.r#type.clone().into();
        self.parse_precedence((u8::from(precedence) + 1).into());

        match token_type {
            TokenType::Plus => self.chunk.write_code(OpCode::Addition.into(), self.previous.line),
            TokenType::Minus => self.chunk.write_code(OpCode::Subtract.into(), self.previous.line), 
            TokenType::Star => self.chunk.write_code(OpCode::Multiply.into(), self.previous.line),
            TokenType::Slash => self.chunk.write_code(OpCode::Divide.into(), self.previous.line),
            _ => return
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();
        match self.previous.r#type {
            TokenType::Number => self.number(),
            TokenType::LeftParen => self.grouping(),
            TokenType::Minus => self.unary(),
            _ => {
                self.error_at_previous("Expect expression");
                return
            }
        }
        
        while u8::from(precedence.clone()) <= Precedence::from(self.previous.r#type.clone()).into() {
            self.advance();
            match self.previous.r#type {
                TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Slash => self.binary(), 
                _ => continue
            }
        }
    }
}