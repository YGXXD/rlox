use crate::value::*;
use crate::chunk::*;
use crate::compiler::*;

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

pub struct VM {
    ip: usize,
    value_stack: Vec<Value>,
}

macro_rules! vm_binary_op {
    ($vm: expr, $op: tt) => {
        {
            let a: Value = $vm.value_stack.pop().unwrap();
            let b: Value = $vm.value_stack.pop().unwrap();
            $vm.value_stack.push(a $op b);
        }
    };
}

impl VM {
    pub fn new() -> Self {
        VM {
            ip: 0,
            value_stack: Vec::<Value>::new(),
        }
    }

    pub fn interpret_source(&mut self, source: &str) -> InterpretResult {
        match compiler_source(source) {
            Some(chunk) => {
                self.interpret_chunk(&chunk)
            },
            None => InterpretResult::CompileError,
        }
    }

    pub fn interpret_chunk(&mut self, chunk: &Chunk) -> InterpretResult {
        self.ip = 0;
        self.value_stack.clear();
        self.run(chunk)
    }

    fn run(&mut self, chunk: &Chunk) -> InterpretResult {
        let interpret_result = {
            loop {
                #[cfg(debug_assertions)]
                {
                    for value in self.value_stack.iter() {
                        println!("|{:^12.6}|", value);
                    }
                    chunk.disassemble_instruction(self.ip);
                }
                let instruction: OpCode = chunk.read_code(self.ip).into();
                self.ip = self.ip + 1;
                match instruction {
                    OpCode::Return => {
                        println!("{}", self.value_stack.pop().unwrap());
                        break InterpretResult::Success;
                    }
                    OpCode::Constant => {
                        let constant: Value =
                            chunk.read_constant(chunk.read_code(self.ip) as usize);
                        self.ip = self.ip + 1;
                        self.value_stack.push(constant);
                    }
                    OpCode::Negate => {
                        let value: Value = self.value_stack.pop().unwrap();
                        self.value_stack.push(-value);
                    }
                    OpCode::Addition => vm_binary_op!(self, +),
                    OpCode::Subtract => vm_binary_op!(self, -),
                    OpCode::Multiply => vm_binary_op!(self, *),
                    OpCode::Divide => vm_binary_op!(self, /),
                }
            }
        };
        interpret_result
    }
}
