use std::collections::HashMap;

use crate::chunk::*;
use crate::compiler::*;
use crate::value::Value;

pub enum InterpretResult {
    Success = 0,
    CompileError,
    RuntimeError,
}

impl ToString for InterpretResult {
    fn to_string(&self) -> String {
        match self {
            InterpretResult::Success => "Success".to_string(),
            InterpretResult::CompileError => "CompileError".to_string(),
            InterpretResult::RuntimeError => "RuntimeError".to_string(),
        }
    }
}

pub struct VM {
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

macro_rules! push_constant {
    ($vm: expr, $chunk: expr, $value_type: ident, $read_op: ident) => {{
        let index: usize = $vm.read_byte($chunk) as usize;
        let value: Value = Value::$value_type($chunk.$read_op(index).clone());
        $vm.stack.push(value);
    }};
}

macro_rules! unary_op {
    ($vm: expr, $chunk: expr, $op: expr) => {{
        let top = $vm.stack.pop().unwrap();
        match $op(top) {
            Ok(v) => $vm.stack.push(v),
            Err(msg) => {
                $vm.runtime_error($chunk, msg);
                break InterpretResult::RuntimeError;
            }
        }
    }};
}

macro_rules! binary_op {
    ($vm: expr, $chunk: expr, $op: expr) => {{
        let b: Value = $vm.stack.pop().unwrap();
        let a: Value = $vm.stack.pop().unwrap();
        match $op(a, b) {
            Ok(v) => $vm.stack.push(v),
            Err(msg) => {
                $vm.runtime_error($chunk, msg);
                break InterpretResult::RuntimeError;
            }
        }
    }};
}

impl VM {
    pub fn new() -> Self {
        VM {
            ip: 0,
            stack: Vec::<Value>::new(),
            globals: HashMap::<String, Value>::new(),
        }
    }

    pub fn interpret_source(&mut self, source: &String) -> InterpretResult {
        let mut compiler: Compiler = Compiler::new();
        match compiler.compile(source) {
            Ok(chunk) => self.interpret_chunk(&chunk),
            Err(_) => InterpretResult::CompileError,
        }
    }

    pub fn interpret_chunk(&mut self, chunk: &Chunk) -> InterpretResult {
        self.ip = 0;
        self.stack.clear();
        self.run(chunk)
    }

    pub fn reset_stack(&mut self) {
        self.stack = Vec::<Value>::new();
    }

    fn run(&mut self, chunk: &Chunk) -> InterpretResult {
        let interpret_result = {
            loop {
                #[cfg(debug_assertions)]
                {
                    println!("");
                    println!("|{:^16}|", "--stack--");
                    for value in self.stack.iter() {
                        println!("|{:^16}|", value.to_string());
                    }
                    chunk.disassemble_instruction(self.ip);
                }
                let instruction: OpCode = self.read_byte(chunk).into();
                match instruction {
                    OpCode::Return => {
                        // exit interpret
                        break InterpretResult::Success;
                    }
                    OpCode::Nil => self.stack.push(Value::Nil),
                    OpCode::True => self.stack.push(Value::Bool(true)),
                    OpCode::False => self.stack.push(Value::Bool(false)),
                    OpCode::Number => push_constant!(self, chunk, Number, read_number),
                    OpCode::String => push_constant!(self, chunk, String, read_string),
                    OpCode::Equal => binary_op!(self, chunk, |x: Value, y: Value| x.equal(&y)),
                    OpCode::Greater => binary_op!(self, chunk, |x: Value, y: Value| x.greater(&y)),
                    OpCode::Less => binary_op!(self, chunk, |x: Value, y: Value| x.less(&y)),
                    OpCode::Not => unary_op!(self, chunk, |x: Value| !x),
                    OpCode::Negate => unary_op!(self, chunk, |x: Value| -x),
                    OpCode::Addition => binary_op!(self, chunk, |x: Value, y: Value| x + y),
                    OpCode::Subtract => binary_op!(self, chunk, |x: Value, y: Value| x - y),
                    OpCode::Multiply => binary_op!(self, chunk, |x: Value, y: Value| x * y),
                    OpCode::Divide => binary_op!(self, chunk, |x: Value, y: Value| x / y),
                    OpCode::Print => println!("{}", self.stack.pop().unwrap().to_string()),
                    OpCode::Pop => {
                        let _ = self.stack.pop().unwrap();
                    }
                    OpCode::DefineGlobal => {
                        let index: usize = self.read_byte(chunk) as usize;
                        let identifier: &String = chunk.read_identifier(index);
                        let value: Value = self.stack.pop().unwrap();
                        self.globals.insert(identifier.clone(), value);
                    }
                    OpCode::GetGlobal => {
                        let index: usize = self.read_byte(chunk) as usize;
                        let identifier: &String = chunk.read_identifier(index);
                        match self.globals.get(identifier) {
                            Some(v) => self.stack.push(v.clone()),
                            None => {
                                self.runtime_error(
                                    chunk,
                                    &format!("Undefined variable '{}'", identifier),
                                );
                                break InterpretResult::RuntimeError;
                            }
                        };
                    }
                    OpCode::SetGlobal => {
                        let index: usize = self.read_byte(chunk) as usize;
                        let identifier: &String = chunk.read_identifier(index);
                        match self.globals.get(identifier) {
                            Some(_) => {
                                let value: Value = self.stack.last().unwrap().clone();
                                self.globals.insert(identifier.clone(), value);
                            }
                            None => {
                                self.runtime_error(
                                    chunk,
                                    &format!("Undefined variable '{}'", identifier),
                                );
                                break InterpretResult::RuntimeError;
                            }
                        }
                    }
                }
            }
        };
        interpret_result
    }

    fn read_byte(&mut self, chunk: &Chunk) -> u8 {
        let byte: u8 = chunk.read_code(self.ip);
        self.ip += 1;
        byte
    }

    fn runtime_error(&mut self, chunk: &Chunk, message: &str) {
        eprintln!(
            "{} : [line {}] in script",
            message,
            chunk.read_line(self.ip - 1)
        );
        self.reset_stack()
    }
}
