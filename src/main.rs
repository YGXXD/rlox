mod chunk;
mod compiler;
// mod paser;
mod scanner;
mod token;
mod value;
mod vm;

use std::path::PathBuf;

use compiler::*;
use vm::*;

fn interpret(byte_stream: &String) {
    println!("{}", byte_stream);
    println!("");
    let mut compiler: Compiler = Compiler::new();
    let _ = compiler.compile(byte_stream);
}

fn repl() {
    let mut input = String::new();
    loop {
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                let line_stream = input.trim().to_string();
                interpret(&line_stream);
                input.clear();
            }
            Err(_) => {
                println!("input read_line error");
                break;
            }
        }
    }
}

fn run_file(file_path: &String) {
    let path: PathBuf = PathBuf::from(file_path);
    let data: Vec<u8> = std::fs::read(path).unwrap();
    let byte_stream: String = String::from_utf8(data).unwrap();
    interpret(&byte_stream);
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() == 1 {
        repl();
    } else if argv.len() == 2 {
        run_file(&argv[1]);
    } else {
        println!("Usage: clox [path]\n");
    }

    // let mut chunk: Chunk = Chunk::new();

    // let idx: u8 = chunk.add_constant(2.4) as u8;
    // chunk.write_code(OpCode::Constant.into(), 1);
    // chunk.write_code(idx, 1);

    // let idx: u8 = chunk.add_constant(5.6) as u8;
    // chunk.write_code(OpCode::Constant.into(), 1);
    // chunk.write_code(idx, 1);

    // let idx: u8 = chunk.add_constant(8.2) as u8;
    // chunk.write_code(OpCode::Constant.into(), 1);
    // chunk.write_code(idx, 1);

    // chunk.write_code(OpCode::Negate.into(), 1);
    // chunk.write_code(OpCode::Addition.into(), 1);
    // chunk.write_code(OpCode::Multiply.into(), 1);
    // chunk.write_code(OpCode::Negate.into(), 1);

    // chunk.write_code(OpCode::Return.into(), 2);
    // chunk.disassemble("test");

    // let mut vm: VM = VM::new();
    // vm.interpret(&mut chunk);

    // chunk.clear();
}
