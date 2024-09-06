use crate::chunk::*;
use crate::paser::Parser;
use crate::scanner::*;
use crate::InterpretResult;
use crate::VM;
pub struct Compiler {}

impl Compiler {
    pub fn compiler(source: &str) -> Result<Chunk, InterpretResult> {
        let mut scanner: Scanner = Scanner::new(source);
        let mut chunk: Chunk = Chunk::new();
        let mut paser = Parser::new(&mut scanner, &mut chunk);

        // loop {
        //     let token = paser.scanner.scan_token();
        //     println!("{}    {}    {}", token.line, token.r#type.to_string(), token.lexeme);
        //     match token.r#type {
        //         TokenType::Eof | TokenType::Error => break,
        //         _ => continue
        //     }
        // }

        paser.advance();
        paser.expression();
        paser.consume(TokenType::Eof, "Expect end of expression");
        paser.chunk.write_code(OpCode::Return.into(), paser.previous.line);
        paser.chunk.disassemble("test");

        let mut vm = VM::new();
        vm.interpret_chunk(&paser.chunk);

        match paser.had_error {
            true => Ok(chunk),
            false => Err(InterpretResult::CompileError),
        }
        
    }
}
