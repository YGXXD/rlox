mod chunk;
mod compiler;
// mod paser;
mod scanner;
mod token;
mod value;
mod vm;

use compiler::*;
use vm::*;

fn interpret(byte_stream: &String) {
    let mut vm = VM::new();
    vm.interpret_source(byte_stream);
    // let mut compiler: Compiler = Compiler::new();
    // let _ = compiler.compile(byte_stream);
}

fn repl() {
    let mut input = String::new();
    loop {
        print!("> ");
        let _ = std::io::Write::flush(&mut std::io::stdout());
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
    let path: std::path::PathBuf = std::path::PathBuf::from(file_path);
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
}
