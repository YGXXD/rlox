use crate::chunk::*;
use crate::scanner::*;
use crate::token::*;

#[derive(PartialEq, PartialOrd, Copy, Clone)]
enum Precedence {
    None = 0,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
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
            6 => Self::Term,
            7 => Self::Factor,
            8 => Self::Unary,
            9 => Self::Call,
            10 => Self::Primary,
            _ => unimplemented!("Invalid Precedence"),
        }
    }
}

impl Precedence {
    fn promote(&self) -> Self {
        return (u8::from(self.clone()) + 1).into();
    }
}

#[derive(Clone, Copy)]
struct ParseRule {
    prefix: Option<fn(&mut Compiler)>,
    infix: Option<fn(&mut Compiler)>,
    precedence: Precedence,
}

static PARSE_RULES: [ParseRule; TokenType::Error as usize] = {
    let mut vec: [ParseRule; TokenType::Error as usize] = [ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    }; TokenType::Error as usize];

    vec[TokenType::LeftParen as usize] = ParseRule {
        prefix: Some(Compiler::parse_grouping),
        infix: None,
        precedence: Precedence::None,
    };
    vec[TokenType::Plus as usize] = ParseRule {
        prefix: None,
        infix: Some(Compiler::parse_binary),
        precedence: Precedence::Term,
    };
    vec[TokenType::Minus as usize] = ParseRule {
        prefix: Some(Compiler::parse_unary),
        infix: Some(Compiler::parse_binary),
        precedence: Precedence::Term,
    };
    vec[TokenType::Star as usize] = ParseRule {
        prefix: None,
        infix: Some(Compiler::parse_binary),
        precedence: Precedence::Factor,
    };
    vec[TokenType::Slash as usize] = ParseRule {
        prefix: None,
        infix: Some(Compiler::parse_binary),
        precedence: Precedence::Factor,
    };
    vec[TokenType::Number as usize] = ParseRule {
        prefix: Some(Compiler::parse_number),
        infix: None,
        precedence: Precedence::None,
    };
    vec[TokenType::Nil as usize] = ParseRule {
        prefix: Some(Compiler::parse_literal),
        infix: None,
        precedence: Precedence::None,
    };
    vec[TokenType::True as usize] = ParseRule {
        prefix: Some(Compiler::parse_literal),
        infix: None,
        precedence: Precedence::None,
    };
    vec[TokenType::False as usize] = ParseRule {
        prefix: Some(Compiler::parse_literal),
        infix: None,
        precedence: Precedence::None,
    };
    vec[TokenType::Bang as usize] = ParseRule {
        prefix: Some(Compiler::parse_unary),
        infix: None,
        precedence: Precedence::None,
    };
    vec[TokenType::BangEqual as usize] = ParseRule {
        prefix: None,
        infix: Some(Compiler::parse_binary),
        precedence: Precedence::Equality,
    };
    vec[TokenType::EqualEqual as usize] = ParseRule {
        prefix: None,
        infix: Some(Compiler::parse_binary),
        precedence: Precedence::Equality,
    };
    vec[TokenType::Greater as usize] = ParseRule {
        prefix: None,
        infix: Some(Compiler::parse_binary),
        precedence: Precedence::Comparison,
    };
    vec[TokenType::GreaterEqual as usize] = ParseRule {
        prefix: None,
        infix: Some(Compiler::parse_binary),
        precedence: Precedence::Comparison,
    };
    vec[TokenType::Less as usize] = ParseRule {
        prefix: None,
        infix: Some(Compiler::parse_binary),
        precedence: Precedence::Comparison,
    };
    vec[TokenType::LessEqual as usize] = ParseRule {
        prefix: None,
        infix: Some(Compiler::parse_binary),
        precedence: Precedence::Comparison,
    };
    vec
};

pub struct Compiler {
    scanner: Scanner,
    chunk: Chunk,
    current: Token,
    previous: Token,
    had_error: std::cell::RefCell<bool>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            scanner: Scanner::new(),
            chunk: Chunk::new(),
            current: Token::default(),
            previous: Token::default(),
            had_error: std::cell::RefCell::<bool>::new(false),
        }
    }

    pub fn compile(&mut self, source: &String) -> Result<&Chunk, String> {
        self.scanner.reset(source);
        self.advance();
        self.parse_expression();
        self.consume(TokenType::Eof, "Expect end of expression");
        self.chunk
            .write_code(OpCode::Return.into(), self.previous.line);

        // loop {
        //     let token = self.scanner.scan_token();
        //     println!(
        //         "{}    {}    {}",
        //         token.line,
        //         token.r#type.to_string(),
        //         token.lexeme
        //     );
        //     match token.r#type {
        //         TokenType::Eof | TokenType::Error => break,
        //         _ => continue,
        //     }
        // }

        // self.chunk.disassemble("chunk");
        // println!("");

        if *self.had_error.borrow() {
            Err("Compile error".to_string())
        } else {
            Ok(&self.chunk)
        }
    }

    fn compile_error(&self, token: &Token, message: &str) {
        eprint!("[line {}] Error ", token.line);
        match token.r#type {
            TokenType::Eof => eprint!("at end"),
            TokenType::Error => eprint!("{}", token.lexeme),
            _ => eprint!("at '{}", token.lexeme),
        }
        eprintln!(" : {}", message);
        self.had_error.replace(true);
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();
        loop {
            self.current = self.scanner.scan_token();
            match self.current.r#type {
                TokenType::Error => self.compile_error(&self.current, "Scan Lex error"),
                _ => break,
            }
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        match self.current.r#type == token_type {
            true => self.advance(),
            false => self.compile_error(&self.current, message),
        }
    }

    fn parse_expression(&mut self) {
        match self.current.r#type != TokenType::Eof {
            true => self.parse_precedence(Precedence::Assignment),
            false => return,
        }
    }

    fn parse_grouping(&mut self) {
        self.parse_expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression")
    }

    fn parse_number(&mut self) {
        self.chunk
            .write_code(OpCode::Constant.into(), self.previous.line);
        match self.previous.lexeme.parse::<f64>() {
            Ok(number) => match self.chunk.add_constant(number) {
                Ok(idx) => self.chunk.write_code(idx as u8, self.previous.line),
                Err(e) => self.compile_error(&self.previous, &e),
            },
            Err(_) => self.compile_error(&self.previous, "Expect number Error"),
        };
    }

    fn parse_unary(&mut self) {
        let unary_token = self.previous.clone();
        self.parse_precedence(Precedence::Unary);

        match unary_token.r#type {
            TokenType::Minus => self
                .chunk
                .write_code(OpCode::Negate.into(), unary_token.line),
            TokenType::Bang => self.chunk.write_code(OpCode::Not.into(), unary_token.line),
            _ => self.compile_error(&unary_token, "Expect unary Error"),
        }
    }

    fn parse_binary(&mut self) {
        let binary_token = self.previous.clone();
        self.parse_precedence(
            PARSE_RULES[Into::<usize>::into(binary_token.r#type.clone())]
                .precedence
                .promote(),
        );

        match binary_token.r#type {
            TokenType::Plus => self
                .chunk
                .write_code(OpCode::Addition.into(), binary_token.line),
            TokenType::Minus => self
                .chunk
                .write_code(OpCode::Subtract.into(), binary_token.line),
            TokenType::Star => self
                .chunk
                .write_code(OpCode::Multiply.into(), binary_token.line),
            TokenType::Slash => self
                .chunk
                .write_code(OpCode::Divide.into(), binary_token.line),
            TokenType::BangEqual => {
                self.chunk
                    .write_code(OpCode::Equal.into(), binary_token.line);
                self.chunk.write_code(OpCode::Not.into(), binary_token.line);
            }
            TokenType::EqualEqual => self
                .chunk
                .write_code(OpCode::Equal.into(), binary_token.line),
            TokenType::Greater => self
                .chunk
                .write_code(OpCode::Greater.into(), binary_token.line),
            TokenType::GreaterEqual => {
                self.chunk
                    .write_code(OpCode::Less.into(), binary_token.line);
                self.chunk.write_code(OpCode::Not.into(), binary_token.line);
            }
            TokenType::Less => self
                .chunk
                .write_code(OpCode::Less.into(), binary_token.line),
            TokenType::LessEqual => {
                self.chunk
                    .write_code(OpCode::Greater.into(), binary_token.line);
                self.chunk.write_code(OpCode::Not.into(), binary_token.line);
            }
            _ => self.compile_error(&binary_token, "Expect binary Error"),
        }
    }

    fn parse_literal(&mut self) {
        match self.previous.r#type {
            TokenType::Nil => self
                .chunk
                .write_code(OpCode::Nil.into(), self.previous.line),
            TokenType::True => self
                .chunk
                .write_code(OpCode::True.into(), self.previous.line),
            TokenType::False => self
                .chunk
                .write_code(OpCode::False.into(), self.previous.line),
            _ => self.compile_error(&self.previous, "Expect literal Error"),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        // prefix
        match PARSE_RULES[Into::<usize>::into(self.previous.r#type.clone())].prefix {
            Some(parse_fn) => parse_fn(self),
            None => self.compile_error(&self.previous, "Expect prefix error"),
        }

        // infix
        loop {
            match precedence
                <= PARSE_RULES[Into::<usize>::into(self.current.r#type.clone())].precedence
            {
                true => {
                    self.advance();
                    match PARSE_RULES[Into::<usize>::into(self.previous.r#type.clone())].infix {
                        Some(parse_fn) => parse_fn(self),
                        None => continue,
                    }
                }
                false => break,
            }
        }
    }
}
