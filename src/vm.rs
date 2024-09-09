use crate::chunk;
use crate::chunk::*;
use crate::compiler::*;
use crate::value::Value;

pub enum InterpretResult {
    Success,
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

macro_rules! binary_op {
    ($vm: expr, $op: tt) => {
        {
            let b: Value = $vm.stack.pop().unwrap();
            let a: Value = $vm.stack.pop().unwrap();
            $vm.stack.push(a $op b);
        }
    };
}

pub struct VM {
    ip: usize,
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            ip: 0,
            stack: Vec::<Value>::new(),
        }
    }

    pub fn interpret_source(&mut self, source: &String) -> InterpretResult {
        match Compiler::compile(source) {
            Ok(chunk) => self.interpret_chunk(&chunk),
            Err(result) => result,
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
                    for value in self.stack.iter() {
                        println!("|{:^12.6}|", value);
                    }
                    chunk.disassemble_instruction(self.ip);
                }
                let instruction: OpCode = self.read_byte(chunk);
                match instruction {
                    OpCode::Return => {
                        println!("{}", self.stack.pop().unwrap());
                        break InterpretResult::Success;
                    }
                    OpCode::Constant => {
                        let constant: Value = self.read_constant(chunk);
                        self.stack.push(constant);
                    }
                    OpCode::Negate => {
                        let value: Value = self.stack.pop().unwrap();
                        self.stack.push(-value);
                    }
                    OpCode::Addition => binary_op!(self, +),
                    OpCode::Subtract => binary_op!(self, -),
                    OpCode::Multiply => binary_op!(self, *),
                    OpCode::Divide => binary_op!(self, /),
                }
            }
        };
        interpret_result
    }

    fn read_byte(&mut self, chunk: &Chunk) -> OpCode {
        let byte: OpCode = chunk.read_code(self.ip).into();
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self, chunk: &Chunk) -> Value {
        let index: usize = chunk.read_code(self.ip) as usize;
        let constant: Value = chunk.read_constant(index);
        self.ip += 1;
        constant
    }
}
