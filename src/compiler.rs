use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::chunk::*;
use crate::function::*;
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
    vec[TokenType::And as usize] = ParseRule {
        prefix: None,
        infix: Some(Compiler::parse_and),
        precedence: Precedence::And,
    };
    vec[TokenType::Or as usize] = ParseRule {
        prefix: None,
        infix: Some(Compiler::parse_or),
        precedence: Precedence::Or,
    };
    vec
};

struct CompileContext {
    // depth -> local_map(identifier -> index)
    variables: RefCell<HashMap<usize, HashMap<String, usize>>>,
    local_count: RefCell<usize>,
    depth: RefCell<usize>,

    // compile result
    chunk: RefCell<Chunk>,
    function_name: RefCell<String>,
}

impl CompileContext {
    fn new() -> Self {
        Self {
            variables: RefCell::new(HashMap::new()),
            local_count: RefCell::new(0),
            depth: RefCell::new(0),
            chunk: RefCell::new(Chunk::new()),
            function_name: RefCell::new(String::default()),
        }
    }
}

pub struct Compiler {
    scanner: Scanner,
    current: Token,
    previous: Token,
    is_panic: RefCell<bool>,
    had_error: RefCell<bool>,

    // compile stack
    compile_context_stack: Vec<Rc<CompileContext>>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            scanner: Scanner::new(),
            current: Token::default(),
            previous: Token::default(),
            is_panic: RefCell::<bool>::new(false),
            had_error: RefCell::<bool>::new(false),
            compile_context_stack: Vec::<Rc<CompileContext>>::new(),
        }
    }

    fn root_context(&self) -> Rc<CompileContext> {
        self.compile_context_stack.first().unwrap().clone()
    }

    fn push_context(&mut self) {
        self.compile_context_stack
            .push(Rc::new(CompileContext::new()));
    }

    fn curr_context(&self) -> Rc<CompileContext> {
        self.compile_context_stack.last().unwrap().clone()
    }

    fn pop_context(&mut self) -> Rc<CompileContext> {
        self.compile_context_stack.pop().unwrap()
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

    pub fn compile(&mut self, source: &String) -> Result<Function, String> {
        self.scanner.reset(source);
        // compile context push
        self.push_context();
        // global variables
        self.root_context()
            .variables
            .borrow_mut()
            .insert(0, HashMap::<String, usize>::new());

        self.advance();
        loop {
            match self.r#match(TokenType::Eof) {
                true => break,
                false => self.declaration(),
            }
        }
        let chunk = self.compile_end();

        if *self.had_error.borrow() {
            Err("Compile error".to_string())
        } else {
            let function: Function = Function {
                name: String::default(),
                params_num: 0,
                chunk: Rc::new(chunk),
            };
            #[cfg(debug_assertions)]
            {
                function.disassemble();
            }
            Ok(function)
        }
    }

    fn compile_end(&mut self) -> Chunk {
        self.consume(TokenType::Eof, "Expect end of expression");
        self.curr_context()
            .chunk
            .borrow_mut()
            .write_code(OpCode::Return.into(), self.previous.line);
        self.pop_context().chunk.replace(Chunk::new())
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
            TokenType::If => {
                self.advance();
                self.if_statement();
            }
            TokenType::While => {
                self.advance();
                self.while_statement();
            }
            TokenType::For => {
                self.advance();
                self.for_statement();
            }
            _ => self.expression_statement(),
        }
    }

    fn print_statement(&mut self) {
        self.parse_expression();
        self.consume(TokenType::Semicolon, "Expect ';' after print value");
        self.curr_context()
            .chunk
            .borrow_mut()
            .write_code(OpCode::Print.into(), self.previous.line);
    }

    fn expression_statement(&mut self) {
        self.parse_expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression");
        self.curr_context()
            .chunk
            .borrow_mut()
            .write_code(OpCode::Pop.into(), self.previous.line);
    }

    fn variable_statement(&mut self) {
        match self.r#match(TokenType::Identifier) {
            true => {
                let context: Rc<CompileContext> = self.curr_context();
                let identifier_token: Token = self.previous.clone();
                let curr_depth = *context.depth.borrow();

                match {
                    let curr_variables = context.variables.borrow();
                    let curr_variable_map = curr_variables.get(&curr_depth).unwrap();
                    curr_variable_map.contains_key(&identifier_token.lexeme)
                } {
                    true => {
                        self.throw_error(&identifier_token, "Redefined identifier in curr space")
                    }
                    false => {
                        match self.r#match(TokenType::Equal) {
                            true => self.parse_expression(),
                            false => context
                                .chunk
                                .borrow_mut()
                                .write_code(OpCode::Nil.into(), identifier_token.line),
                        }
                        self.consume(TokenType::Semicolon, "Expect ';' after variable statement");

                        let mut curr_variables = context.variables.borrow_mut();
                        let curr_variable_map = curr_variables.get_mut(&curr_depth).unwrap();

                        match curr_depth {
                            0 => {
                                let global_slot = curr_variable_map.len();
                                let idx_option =
                                    context.chunk.borrow_mut().add_variable(global_slot);
                                match idx_option {
                                    Ok(idx) => {
                                        curr_variable_map
                                            .insert(identifier_token.lexeme, global_slot);
                                        context.chunk.borrow_mut().write_code(
                                            OpCode::DefineGlobal.into(),
                                            identifier_token.line,
                                        );
                                        context
                                            .chunk
                                            .borrow_mut()
                                            .write_code(idx as u8, identifier_token.line);
                                    }
                                    Err(e) => self.throw_error(&identifier_token, &e),
                                }
                            }
                            _ => {
                                curr_variable_map
                                    .insert(identifier_token.lexeme, *context.local_count.borrow());
                                *context.local_count.borrow_mut() += 1;
                            }
                        }
                    }
                };
            }
            false => self.throw_error(&self.current, "Expect variable error"),
        }
    }

    fn block_statement(&mut self) {
        self.scoop_begin();

        loop {
            match self.current.r#type != TokenType::RightBrace
                && self.current.r#type != TokenType::Eof
            {
                true => self.declaration(),
                false => break,
            }
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block");

        self.scoop_end();
    }

    fn if_statement(&mut self) {
        let context: Rc<CompileContext> = self.curr_context();

        self.consume(TokenType::LeftParen, "Expect '(' after 'if'");
        self.parse_expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition");

        let jump_false_code_offset: usize = self.patch_forward_begin(OpCode::JumpFalse);
        context
            .chunk
            .borrow_mut()
            .write_code(OpCode::Pop.into(), self.previous.line); // pop if expression
        self.statement();

        let jump_code_offset: usize = self.patch_forward_begin(OpCode::Jump);
        self.patch_forward_end(jump_false_code_offset);
        context
            .chunk
            .borrow_mut()
            .write_code(OpCode::Pop.into(), self.previous.line); // pop if expression
        if self.r#match(TokenType::Else) {
            self.statement();
        }
        self.patch_forward_end(jump_code_offset);
    }

    fn while_statement(&mut self) {
        let context: Rc<CompileContext> = self.curr_context();

        let start_code_offset: usize = context.chunk.borrow().code_size() - 1;

        self.consume(TokenType::LeftParen, "Expect '(' after 'while'");
        self.parse_expression();
        self.consume(TokenType::RightParen, "Expect ')' after condition");

        let jump_false_code_offset: usize = self.patch_forward_begin(OpCode::JumpFalse);
        context
            .chunk
            .borrow_mut()
            .write_code(OpCode::Pop.into(), self.previous.line); // pop while expression
        self.statement();
        self.patch_back(OpCode::JumpBack, start_code_offset);
        self.patch_forward_end(jump_false_code_offset);
        context
            .chunk
            .borrow_mut()
            .write_code(OpCode::Pop.into(), self.previous.line); // pop while expression
    }

    fn for_statement(&mut self) {
        self.scoop_begin();

        let context: Rc<CompileContext> = self.curr_context();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'");

        match self.current.r#type {
            TokenType::Semicolon => {
                self.advance();
            }
            TokenType::Var => {
                self.advance();
                self.variable_statement();
            }
            _ => self.expression_statement(),
        }

        let mut start_code_offset: usize = context.chunk.borrow().code_size() - 1;

        let mut jump_false_code_offset: Option<usize> = None;
        if !self.r#match(TokenType::Semicolon) {
            self.parse_expression(); // push condition
            self.consume(TokenType::Semicolon, "Expect ';'");

            jump_false_code_offset = Some(self.patch_forward_begin(OpCode::JumpFalse));
            context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Pop.into(), self.previous.line); // pop for condition
        }

        if !self.r#match(TokenType::RightParen) {
            let jump_code_offset: usize = self.patch_forward_begin(OpCode::Jump);

            let increment_code_offset = context.chunk.borrow().code_size() - 1;
            self.parse_expression();
            context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Pop.into(), self.previous.line); // pop for increment expression
            self.consume(TokenType::RightParen, "Expect ')' after for clauses");
            self.patch_back(OpCode::JumpBack, start_code_offset);
            start_code_offset = increment_code_offset;

            self.patch_forward_end(jump_code_offset);
        }

        self.statement();
        self.patch_back(OpCode::JumpBack, start_code_offset);

        if let Some(code_offset) = jump_false_code_offset {
            self.patch_forward_end(code_offset);
            context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Pop.into(), self.previous.line); // pop for condition
        }

        self.scoop_end();
    }

    fn function_statement(&mut self) {
        match self.r#match(TokenType::Identifier) {
            true => {
                let context: Rc<CompileContext> = self.curr_context();
                let identifier_token: Token = self.previous.clone();
                let curr_depth = *context.depth.borrow();

                match {
                    let curr_variables = context.variables.borrow();
                    let curr_variable_map = curr_variables.get(&curr_depth).unwrap();
                    curr_variable_map.contains_key(&identifier_token.lexeme)
                } {
                    true => {
                        self.throw_error(&identifier_token, "Redefined identifier in curr space")
                    }
                    false => {
                        self.scoop_begin();

                        self.consume(TokenType::LeftParen, "Expect '(' after function name");
                        self.consume(TokenType::RightParen, "Expect ')' after parameters");
                        self.consume(TokenType::LeftBrace, "Expect '{' before function body");
                        self.block_statement();

                        self.compile_end();

                        // emitBytes(OP_CONSTANT, makeConstant(OBJ_VAL(function)));
                        todo!()
                    }
                }
            }
            false => self.throw_error(&self.current, "Expect function error"),
        }
    }

    fn scoop_begin(&mut self) {
        let context: Rc<CompileContext> = self.curr_context();
        *context.depth.borrow_mut() += 1;
        let depth = *context.depth.borrow();
        context
            .variables
            .borrow_mut()
            .insert(depth, HashMap::<String, usize>::new());
    }

    fn scoop_end(&mut self) {
        let context: Rc<CompileContext> = self.curr_context();
        let depth = *context.depth.borrow();
        let block_variables_len: usize = context.variables.borrow().get(&depth).unwrap().len();
        for _ in 0..block_variables_len {
            context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Pop.into(), self.previous.line);
        }
        *context.local_count.borrow_mut() -= block_variables_len;
        context.variables.borrow_mut().remove(&depth);
        *context.depth.borrow_mut() -= 1;
    }

    fn patch_forward_begin(&mut self, jump_code: OpCode) -> usize {
        let context: Rc<CompileContext> = self.curr_context();
        let jump_code_offset: usize = context.chunk.borrow().code_size();
        context
            .chunk
            .borrow_mut()
            .write_code(jump_code.into(), self.previous.line);
        context
            .chunk
            .borrow_mut()
            .write_code(0xff, self.previous.line);
        context
            .chunk
            .borrow_mut()
            .write_code(0xff, self.previous.line);
        jump_code_offset
    }

    fn patch_forward_end(&mut self, jump_code_offset: usize) {
        let context: Rc<CompileContext> = self.curr_context();
        let jump_count: usize = context.chunk.borrow().code_size() - jump_code_offset - 3;
        if jump_count > u16::MAX as usize {
            self.throw_error(&self.previous, "Too much code to jump over");
        }
        context
            .chunk
            .borrow_mut()
            .update_code(jump_code_offset + 1, (jump_count & 0xff) as u8);
        context
            .chunk
            .borrow_mut()
            .update_code(jump_code_offset + 2, ((jump_count >> 8) & 0xff) as u8);
    }

    fn patch_back(&mut self, jump_code: OpCode, start_code_offset: usize) {
        let context: Rc<CompileContext> = self.curr_context();
        let jump_count: usize = context.chunk.borrow_mut().code_size() - start_code_offset + 2;
        if jump_count > u16::MAX as usize {
            self.throw_error(&self.previous, "Too much code to jump over");
        }
        context
            .chunk
            .borrow_mut()
            .write_code(jump_code.into(), self.previous.line);
        context
            .chunk
            .borrow_mut()
            .write_code((jump_count & 0xff) as u8, self.previous.line);
        context
            .chunk
            .borrow_mut()
            .write_code(((jump_count >> 8) & 0xff) as u8, self.previous.line);
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
        let context: Rc<CompileContext> = self.curr_context();
        context
            .chunk
            .borrow_mut()
            .write_code(OpCode::Number.into(), self.previous.line);
        match self.previous.lexeme.parse::<f64>() {
            Ok(number) => {
                let idx_option = context.chunk.borrow_mut().add_number(number);
                match idx_option {
                    Ok(idx) => context
                        .chunk
                        .borrow_mut()
                        .write_code(idx as u8, self.previous.line),
                    Err(e) => self.throw_error(&self.previous, &e),
                }
            }
            Err(_) => self.throw_error(&self.previous, "Expect number Error"),
        };
    }

    fn parse_string(&mut self) {
        let context: Rc<CompileContext> = self.curr_context();
        context
            .chunk
            .borrow_mut()
            .write_code(OpCode::String.into(), self.previous.line);
        let string_len: usize = self.previous.lexeme.len();
        match string_len >= 2 {
            true => {
                let string: String = if string_len > 2 {
                    self.previous.lexeme[1..(string_len - 1)].to_string()
                } else {
                    "".to_string()
                };
                let idx_option = context.chunk.borrow_mut().add_string(string);
                match idx_option {
                    Ok(idx) => context
                        .chunk
                        .borrow_mut()
                        .write_code(idx as u8, self.previous.line),
                    Err(e) => self.throw_error(&self.previous, &e),
                }
            }
            false => self.throw_error(&self.previous, "Expect string Error"),
        };
    }

    fn parse_literal(&mut self) {
        let context: Rc<CompileContext> = self.curr_context();
        match self.previous.r#type {
            TokenType::Nil => context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Nil.into(), self.previous.line),
            TokenType::True => context
                .chunk
                .borrow_mut()
                .write_code(OpCode::True.into(), self.previous.line),
            TokenType::False => context
                .chunk
                .borrow_mut()
                .write_code(OpCode::False.into(), self.previous.line),
            _ => self.throw_error(&self.previous, "Expect literal Error"),
        }
    }

    fn parse_variable(&mut self) {
        let context: Rc<CompileContext> = self.curr_context();
        let root_context: Rc<CompileContext> = self.root_context();
        let variable_token = self.previous.clone();

        if let Some(local_slot) = {
            let mut curr_depth = *context.depth.borrow();
            let curr_variables = context.variables.borrow();
            let variable_slot: Option<usize> = loop {
                if curr_depth < 1 {
                    break Option::None;
                }
                let variable_map = curr_variables.get(&curr_depth).unwrap();
                match variable_map.get(&variable_token.lexeme) {
                    Some(v) => break Option::Some(*v),
                    None => curr_depth -= 1,
                }
            };
            variable_slot
        } {
            match self.r#match(TokenType::Equal) {
                true => {
                    self.parse_expression();
                    context
                        .chunk
                        .borrow_mut()
                        .write_code(OpCode::SetLocal.into(), variable_token.line);
                }
                false => context
                    .chunk
                    .borrow_mut()
                    .write_code(OpCode::GetLocal.into(), variable_token.line),
            }
            let idx_option = context.chunk.borrow_mut().add_variable(local_slot);
            match idx_option {
                Ok(idx) => context
                    .chunk
                    .borrow_mut()
                    .write_code(idx as u8, variable_token.line),
                Err(e) => self.throw_error(&variable_token, &e),
            }
        } else if let Some(global_slot) = {
            let root_variables = root_context.variables.borrow();
            let global_variable_slot = root_variables
                .get(&0)
                .unwrap()
                .get(&variable_token.lexeme)
                .cloned();
            global_variable_slot
        } {
            match self.r#match(TokenType::Equal) {
                true => {
                    self.parse_expression();
                    context
                        .chunk
                        .borrow_mut()
                        .write_code(OpCode::SetGlobal.into(), variable_token.line);
                }
                false => context
                    .chunk
                    .borrow_mut()
                    .write_code(OpCode::GetGlobal.into(), variable_token.line),
            }
            let idx_option = context.chunk.borrow_mut().add_variable(global_slot);
            match idx_option {
                Ok(idx) => context
                    .chunk
                    .borrow_mut()
                    .write_code(idx as u8, variable_token.line),
                Err(e) => self.throw_error(&variable_token, &e),
            }
        } else {
            self.throw_error(&variable_token, "Undefined variable Error")
        }
    }

    fn parse_unary(&mut self) {
        let context: Rc<CompileContext> = self.curr_context();
        let unary_token = self.previous.clone();
        self.parse_precedence(Precedence::Unary);

        match unary_token.r#type {
            TokenType::Minus => context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Negate.into(), unary_token.line),
            TokenType::Bang => context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Not.into(), unary_token.line),
            _ => self.throw_error(&unary_token, "Expect unary Error"),
        }
    }

    fn parse_binary(&mut self) {
        let context: Rc<CompileContext> = self.curr_context();
        let binary_token = self.previous.clone();
        self.parse_precedence(
            PARSE_RULES[Into::<usize>::into(binary_token.r#type.clone())]
                .precedence
                .promote(),
        );

        match binary_token.r#type {
            TokenType::Plus => context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Addition.into(), binary_token.line),
            TokenType::Minus => context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Subtract.into(), binary_token.line),
            TokenType::Star => context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Multiply.into(), binary_token.line),
            TokenType::Slash => context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Divide.into(), binary_token.line),
            TokenType::BangEqual => {
                context
                    .chunk
                    .borrow_mut()
                    .write_code(OpCode::Equal.into(), binary_token.line);
                context
                    .chunk
                    .borrow_mut()
                    .write_code(OpCode::Not.into(), binary_token.line);
            }
            TokenType::EqualEqual => context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Equal.into(), binary_token.line),
            TokenType::Greater => context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Greater.into(), binary_token.line),
            TokenType::GreaterEqual => {
                context
                    .chunk
                    .borrow_mut()
                    .write_code(OpCode::Less.into(), binary_token.line);
                context
                    .chunk
                    .borrow_mut()
                    .write_code(OpCode::Not.into(), binary_token.line);
            }
            TokenType::Less => context
                .chunk
                .borrow_mut()
                .write_code(OpCode::Less.into(), binary_token.line),
            TokenType::LessEqual => {
                context
                    .chunk
                    .borrow_mut()
                    .write_code(OpCode::Greater.into(), binary_token.line);
                context
                    .chunk
                    .borrow_mut()
                    .write_code(OpCode::Not.into(), binary_token.line);
            }
            _ => self.throw_error(&binary_token, "Expect binary Error"),
        }
    }

    fn parse_and(&mut self) {
        let context: Rc<CompileContext> = self.curr_context();
        let jump_code_offset: usize = self.patch_forward_begin(OpCode::JumpFalse);
        context
            .chunk
            .borrow_mut()
            .write_code(OpCode::Pop.into(), self.previous.line);
        self.parse_precedence(Precedence::And);
        self.patch_forward_end(jump_code_offset);
    }

    fn parse_or(&mut self) {
        let context: Rc<CompileContext> = self.curr_context();
        let jump_false_code_offset: usize = self.patch_forward_begin(OpCode::JumpFalse);
        let jump_end_code_offset: usize = self.patch_forward_begin(OpCode::Jump);
        self.patch_forward_end(jump_false_code_offset);
        context
            .chunk
            .borrow_mut()
            .write_code(OpCode::Pop.into(), self.previous.line);
        self.parse_precedence(Precedence::Or);
        self.patch_forward_end(jump_end_code_offset);
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
