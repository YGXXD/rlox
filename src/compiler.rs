use crate::chunk::*;
use crate::scanner::*;
use crate::token::*;
use crate::InterpretResult;

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
        prefix: Some(Compiler::parse_unary),
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
    vec
};

pub struct Compiler {
    scanner: Scanner,
    chunk: Chunk,
    current: Token,
    previous: Token,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            scanner: Scanner::new(),
            chunk: Chunk::new(),
            current: Token::default(),
            previous: Token::default(),
        }
    }

    pub fn compile(&mut self, source: &String) -> Result<&Chunk, InterpretResult> {
        self.scanner.reset(source);
        self.advance();
        self.parse_expression();
        self.consume(TokenType::Eof, "Expect end of expression");
        self.chunk.write_code(OpCode::Return.into(), self.previous.line);

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

        self.chunk.disassemble("chunk");
        
        Ok(&self.chunk)
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();
        loop {
            self.current = self.scanner.scan_token();
            match self.current.r#type {
                TokenType::Error => self.current.error("Scan Lex error"),
                _ => break,
            }
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        match self.current.r#type == token_type {
            true => self.advance(),
            false => self.current.error(message),
        }
    }

    fn parse_expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_grouping(&mut self) {
        self.parse_expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression")
    }

    fn parse_number(&mut self) {
        self.chunk
            .write_code(OpCode::Constant.into(), self.previous.line);
        match self.previous.lexeme.parse::<f64>() {
            Ok(value) => match self.chunk.add_constant(value) {
                Ok(idx) => self.chunk.write_code(idx as u8, self.previous.line),
                Err(e) => self.previous.error(e),
            },
            Err(_) => self.previous.error("Expect number Error"),
        };
    }

    fn parse_unary(&mut self) {
        let unary_token = self.previous.clone();
        self.parse_precedence(Precedence::Unary);

        match unary_token.r#type {
            TokenType::Plus => return,
            TokenType::Minus => self
                .chunk
                .write_code(OpCode::Negate.into(), unary_token.line),
            _ => unary_token.error("Expect unary Error"),
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
            _ => binary_token.error("Expect binary Error"),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        // prefix
        match PARSE_RULES[Into::<usize>::into(self.previous.r#type.clone())].prefix {
            Some(parse_fn) => parse_fn(self),
            None => self.previous.error("Expect error"),
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

    // pub fn compiler(source: &str) -> Result<Chunk, InterpretResult> {
    //     let mut scanner: Scanner = Scanner::new(source);
    //     let mut chunk: Chunk = Chunk::new();
    //     let mut paser = Parser::new(&mut scanner, &mut chunk);

    //     // loop {
    //     //     let token = paser.scanner.scan_token();
    //     //     println!("{}    {}    {}", token.line, token.r#type.to_string(), token.lexeme);
    //     //     match token.r#type {
    //     //         TokenType::Eof | TokenType::Error => break,
    //     //         _ => continue
    //     //     }
    //     // }

    //     paser.advance();
    //     paser.expression();
    //     paser.consume(TokenType::Eof, "Expect end of expression");
    //     paser
    //         .chunk
    //         .write_code(OpCode::Return.into(), paser.previous.line);
    //     paser.chunk.disassemble("test");

    //     let mut vm = VM::new();
    //     vm.interpret_chunk(&paser.chunk);

    //     match paser.had_error {
    //         true => Ok(chunk),
    //         false => Err(InterpretResult::CompileError),
    //     }
    // }
}
