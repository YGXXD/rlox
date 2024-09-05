use crate::scanner::*;
use crate::chunk::*;

pub fn compiler_source(source: &str) -> Option<Chunk> {
    let mut scanner: Scanner = Scanner::new(source);
    loop {
        let token = scanner.scan_token();
        println!("{}    {}    {}", token.line, token.r#type.to_string(), token.lexeme);
        match token.r#type {
            TokenType::Eof | TokenType::Error => break,
            _ => continue
        }
    }
    Some(Chunk::new())
}

struct Parser {
    current: Token,
    previous: Token,
    has_error: bool
}

impl Parser {
    fn error_current(&mut self, message: &str) {
        eprint!("[line {}] Error ", self.current.line);
        match self.current.r#type {
            TokenType::Eof => eprint!("at end"),
            TokenType::Error => eprint!("{}", self.current.lexeme),
            _ => eprint!("at {}", self.current.lexeme)
        }
        eprintln!(" : {}", message);
        self.has_error = true;
    }
}