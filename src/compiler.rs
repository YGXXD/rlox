use std::collections::HashMap;

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
    vec[TokenType::String as usize] = ParseRule {
        prefix: Some(Compiler::parse_string),
        infix: None,
        precedence: Precedence::None,
    };
    vec[TokenType::Identifier as usize] = ParseRule {
        prefix: Some(Compiler::parse_variable),
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
    is_panic: std::cell::RefCell<bool>,
    had_error: std::cell::RefCell<bool>,

    // depth -> local_map(identifier -> index)
    variables: HashMap<usize, HashMap<String, usize>>,
    curr_local_count: usize,
    curr_depth: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            scanner: Scanner::new(),
            chunk: Chunk::new(),
            current: Token::default(),
            previous: Token::default(),
            is_panic: std::cell::RefCell::<bool>::new(false),
            had_error: std::cell::RefCell::<bool>::new(false),
            variables: HashMap::<usize, HashMap<String, usize>>::new(),
            curr_local_count: 0,
            curr_depth: 0,
        }
    }

    pub fn show_tokens(&mut self, source: &String) {
        self.scanner.reset(source);
        loop {
            let token = self.scanner.scan_token();
            println!(
                "{}    {}    {}",
                token.line,
                token.r#type.to_string(),
                token.lexeme
            );
            match token.r#type {
                TokenType::Eof | TokenType::Error => break,
                _ => continue,
            }
        }
    }

    pub fn compile(&mut self, source: &String) -> Result<&Chunk, String> {
        self.scanner.reset(source);
        self.variables.clear();
        self.curr_depth = 0;
        self.variables
            .insert(self.curr_depth, HashMap::<String, usize>::new());

        self.advance();
        loop {
            match self.r#match(TokenType::Eof) {
                true => break,
                false => self.declaration(),
            }
        }
        self.compile_end();

        if *self.had_error.borrow() {
            Err("Compile error".to_string())
        } else {
            Ok(&self.chunk)
        }
    }

    fn compile_end(&mut self) {
        self.consume(TokenType::Eof, "Expect end of expression");
        self.chunk
            .write_code(OpCode::Return.into(), self.previous.line);
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();
        loop {
            self.current = self.scanner.scan_token();
            match self.current.r#type {
                TokenType::Error => self.throw_error(&self.current, "Scan Lex error"),
                _ => break,
            }
        }
    }

    fn r#match(&mut self, token_type: TokenType) -> bool {
        match self.current.r#type == token_type {
            true => {
                self.advance();
                true
            }
            false => false,
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if !self.r#match(token_type) {
            self.throw_error(&self.current, message)
        }
    }

    fn throw_error(&self, token: &Token, message: &str) {
        match unsafe { *self.is_panic.as_ptr() } {
            true => return,
            false => {
                self.is_panic.replace(true);
                eprint!("[line {}] Error ", token.line);
                match token.r#type {
                    TokenType::Eof => eprint!("at end"),
                    TokenType::Error => eprint!("{}", token.lexeme),
                    _ => eprint!("at '{}", token.lexeme),
                }
                eprintln!(" : {}", message);
                self.had_error.replace(true);
            }
        }
    }

    fn error_synchronize(&mut self) {
        match unsafe { *self.is_panic.as_ptr() } {
            true => {
                self.is_panic.replace(false);
                loop {
                    match self.current.r#type {
                        TokenType::Eof => return,
                        TokenType::Semicolon => {
                            self.advance();
                            return;
                        }
                        _ => self.advance(),
                    }
                }
            }
            false => return,
        }
    }

    fn declaration(&mut self) {
        self.statement();
        self.error_synchronize();
    }

    fn statement(&mut self) {
        match self.current.r#type {
            TokenType::Var => {
                self.advance();
                self.variable_statement();
            }
            TokenType::Print => {
                self.advance();
                self.print_statement();
            }
            TokenType::LeftBrace => {
                self.advance();
                self.block_statement();
            }
            _ => self.expression_statement(),
        }
    }

    fn print_statement(&mut self) {
        self.parse_expression();
        self.consume(TokenType::Semicolon, "Expect ';' after print value");
        self.chunk
            .write_code(OpCode::Print.into(), self.previous.line);
    }

    fn expression_statement(&mut self) {
        self.parse_expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression");
        self.chunk
            .write_code(OpCode::Pop.into(), self.previous.line);
    }

    // fn variable_statement(&mut self) {
    //     match self.r#match(TokenType::Identifier) {
    //         true => {
    //             let identifier_token: Token = self.previous.clone();
    //             match self.chunk.add_identifier(identifier_token.lexeme) {
    //                 Ok(idx) => {
    //                     match self.r#match(TokenType::Equal) {
    //                         true => self.parse_expression(),
    //                         false => self
    //                             .chunk
    //                             .write_code(OpCode::Nil.into(), identifier_token.line),
    //                     }
    //                     self.consume(TokenType::Semicolon, "Expect ';' after variable statement");
    //                     self.chunk
    //                         .write_code(OpCode::DefineGlobal.into(), identifier_token.line);
    //                     self.chunk.write_code(idx as u8, identifier_token.line);
    //                 }
    //                 Err(e) => self.throw_error(&self.previous, &e),
    //             }
    //         }
    //         false => self.throw_error(&self.current, "Expect variable error"),
    //     }
    // }

    fn variable_statement(&mut self) {
        match self.r#match(TokenType::Identifier) {
            true => {
                let identifier_token: Token = self.previous.clone();
                let curr_depth = self.curr_depth;
                match self
                    .variables
                    .get_mut(&curr_depth)
                    .unwrap()
                    .contains_key(&identifier_token.lexeme)
                {
                    true => self.throw_error(&identifier_token, "Redefined variable in curr space"),
                    false => {
                        match self.r#match(TokenType::Equal) {
                            true => self.parse_expression(),
                            false => self
                                .chunk
                                .write_code(OpCode::Nil.into(), identifier_token.line),
                        }
                        self.consume(TokenType::Semicolon, "Expect ';' after variable statement");

                        let curr_variable_map = self.variables.get_mut(&curr_depth).unwrap();
                        match curr_depth {
                            0 => {
                                let global_slot = curr_variable_map.len();
                                match self.chunk.add_variable(global_slot) {
                                    Ok(idx) => {
                                        curr_variable_map
                                            .insert(identifier_token.lexeme, global_slot);
                                        self.chunk.write_code(
                                            OpCode::DefineGlobal.into(),
                                            identifier_token.line,
                                        );
                                        self.chunk.write_code(idx as u8, identifier_token.line);
                                    }
                                    Err(e) => self.throw_error(&identifier_token, &e),
                                }
                            }
                            _ => {
                                curr_variable_map
                                    .insert(identifier_token.lexeme, self.curr_local_count);
                                self.curr_local_count += 1;
                            }
                        }
                    }
                };
            }
            false => self.throw_error(&self.current, "Expect variable error"),
        }
    }

    fn block_statement(&mut self) {
        self.curr_depth += 1;
        self.variables
            .insert(self.curr_depth, HashMap::<String, usize>::new());

        loop {
            match self.current.r#type != TokenType::RightBrace
                && self.current.r#type != TokenType::Eof
            {
                true => self.declaration(),
                false => break,
            }
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block");

        let block_variables_len: usize = self.variables.get(&self.curr_depth).unwrap().len();
        for _ in 0..block_variables_len {
            self.chunk
                .write_code(OpCode::Pop.into(), self.previous.line);
        }
        self.curr_local_count -= block_variables_len;
        self.variables.remove(&self.curr_depth);
        self.curr_depth -= 1;
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
            .write_code(OpCode::Number.into(), self.previous.line);
        match self.previous.lexeme.parse::<f64>() {
            Ok(number) => match self.chunk.add_number(number) {
                Ok(idx) => self.chunk.write_code(idx as u8, self.previous.line),
                Err(e) => self.throw_error(&self.previous, &e),
            },
            Err(_) => self.throw_error(&self.previous, "Expect number Error"),
        };
    }

    fn parse_string(&mut self) {
        self.chunk
            .write_code(OpCode::String.into(), self.previous.line);
        let string_len: usize = self.previous.lexeme.len();
        match string_len >= 2 {
            true => {
                let string: String = if string_len > 2 {
                    self.previous.lexeme[1..(string_len - 1)].to_string()
                } else {
                    "".to_string()
                };
                match self.chunk.add_string(string) {
                    Ok(idx) => self.chunk.write_code(idx as u8, self.previous.line),
                    Err(e) => self.throw_error(&self.previous, &e),
                }
            }
            false => self.throw_error(&self.previous, "Expect string Error"),
        };
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
            _ => self.throw_error(&self.previous, "Expect literal Error"),
        }
    }

    fn parse_variable(&mut self) {
        let variable_token = self.previous.clone();
        let mut curr_depth = self.curr_depth;
        let variable_slot: Option<usize> = loop {
            if curr_depth < 1 {
                break Option::None;
            }
            let variable_map = self.variables.get(&curr_depth).unwrap();
            match variable_map.get(&variable_token.lexeme) {
                Some(v) => break Option::Some(*v),
                None => curr_depth -= 1,
            }
        };

        if let Some(local_slot) = variable_slot {
            match self.r#match(TokenType::Equal) {
                true => {
                    self.parse_expression();
                    self.chunk
                        .write_code(OpCode::SetLocal.into(), variable_token.line);
                }
                false => self
                    .chunk
                    .write_code(OpCode::GetLocal.into(), variable_token.line),
            }
            match self.chunk.add_variable(local_slot) {
                Ok(idx) => self.chunk.write_code(idx as u8, variable_token.line),
                Err(e) => self.throw_error(&variable_token, &e),
            }
        } else if let Some(global_slot) = self
            .variables
            .get_mut(&0)
            .unwrap()
            .get(&variable_token.lexeme)
            .cloned()
        {
            match self.r#match(TokenType::Equal) {
                true => {
                    self.parse_expression();
                    self.chunk
                        .write_code(OpCode::SetGlobal.into(), variable_token.line);
                }
                false => self
                    .chunk
                    .write_code(OpCode::GetGlobal.into(), variable_token.line),
            }
            match self.chunk.add_variable(global_slot.clone()) {
                Ok(idx) => self.chunk.write_code(idx as u8, variable_token.line),
                Err(e) => self.throw_error(&variable_token, &e),
            }
        } else {
            self.throw_error(&variable_token, "Undefined variable Error")
        }
    }

    fn parse_unary(&mut self) {
        let unary_token = self.previous.clone();
        self.parse_precedence(Precedence::Unary);

        match unary_token.r#type {
            TokenType::Minus => self
                .chunk
                .write_code(OpCode::Negate.into(), unary_token.line),
            TokenType::Bang => self.chunk.write_code(OpCode::Not.into(), unary_token.line),
            _ => self.throw_error(&unary_token, "Expect unary Error"),
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
            _ => self.throw_error(&binary_token, "Expect binary Error"),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        // prefix
        match PARSE_RULES[Into::<usize>::into(self.previous.r#type.clone())].prefix {
            Some(parse_fn) => parse_fn(self),
            None => self.throw_error(&self.previous, "Expect prefix error"),
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
