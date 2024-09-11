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
                    println!("|{:^12}|", "--stack--");
                    for value in self.stack.iter() {
                        println!("|{:^12.6}|", value.to_string());
                    }
                    chunk.disassemble_instruction(self.ip);
                }
                let instruction: OpCode = self.read_byte(chunk);
                match instruction {
                    OpCode::Return => {
                        if let Some(value) = self.stack.pop() {
                            println!("{}", value.to_string());
                        }
                        break InterpretResult::Success;
                    }
                    OpCode::Constant => {
                        let constant: Value = self.read_constant(chunk);
                        self.stack.push(constant);
                    }
                    OpCode::Nil => self.stack.push(Value::Nil),
                    OpCode::True => self.stack.push(Value::Bool(true)),
                    OpCode::False => self.stack.push(Value::Bool(false)),
                    OpCode::Equal => binary_op!(self, chunk, |x: Value, y: Value| x.equal(&y)),
                    OpCode::Greater => binary_op!(self, chunk, |x: Value, y: Value| x.greater(&y)),
                    OpCode::Less => binary_op!(self, chunk, |x: Value, y: Value| x.less(&y)),
                    OpCode::Not => unary_op!(self, chunk, |x: Value| !x),
                    OpCode::Negate => unary_op!(self, chunk, |x: Value| -x),
                    OpCode::Addition => binary_op!(self, chunk, |x: Value, y: Value| x + y),
                    OpCode::Subtract => binary_op!(self, chunk, |x: Value, y: Value| x - y),
                    OpCode::Multiply => binary_op!(self, chunk, |x: Value, y: Value| x * y),
                    OpCode::Divide => binary_op!(self, chunk, |x: Value, y: Value| x / y),
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
        let number: f64 = chunk.read_constant(index);
        self.ip += 1;
        Value::Number(number)
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
