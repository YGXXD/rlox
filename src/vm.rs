use std::cell::RefCell;
use std::rc::Rc;

use crate::chunk::*;
use crate::compiler::*;
use crate::function::*;
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

struct CallFrame {
    ip: RefCell<usize>,
    function: Rc<Function>,
    slot: usize,
}

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    globals: Vec<Option<Value>>,
}

macro_rules! push_constant {
    ($vm: expr, $value_type: ident, $read_op: ident) => {{
        let index: usize = $vm.read_byte() as usize;
        let value: Value = Value::$value_type($vm.curr_chunk().$read_op(index).clone());
        $vm.stack.push(value);
    }};
}

macro_rules! unary_op {
    ($vm: expr, $op: expr) => {{
        let top = $vm.stack.pop().unwrap();
        match $op(top) {
            Ok(v) => $vm.stack.push(v),
            Err(msg) => {
                $vm.runtime_error(msg);
                break InterpretResult::RuntimeError;
            }
        }
    }};
}

macro_rules! binary_op {
    ($vm: expr, $op: expr) => {{
        let b: Value = $vm.stack.pop().unwrap();
        let a: Value = $vm.stack.pop().unwrap();
        match $op(a, b) {
            Ok(v) => $vm.stack.push(v),
            Err(msg) => {
                $vm.runtime_error(msg);
                break InterpretResult::RuntimeError;
            }
        }
    }};
}

impl VM {
    pub fn new() -> Self {
        VM {
            frames: Vec::<CallFrame>::new(),
            stack: Vec::<Value>::new(),
            globals: Vec::<Option<Value>>::new(),
        }
    }

    pub fn interpret_source(&mut self, source: &String) -> InterpretResult {
        let mut compiler: Compiler = Compiler::new();
        match compiler.compile(source) {
            Ok(function) => {
                self.frames.push(CallFrame {
                    ip: RefCell::new(0),
                    function: Rc::new(function),
                    slot: 0,
                });
                self.run()
            }
            Err(_) => InterpretResult::CompileError,
        }
    }

    // pub fn interpret_chunk(&mut self, chunk: &Chunk) -> InterpretResult {
    //     self.stack.clear();
    //     self.globals.clear();
    //     self.globals.resize(256, Option::None);
    //     self.run(chunk)
    // }

    pub fn reset_stack(&mut self) {
        self.stack = Vec::<Value>::new();
    }

    fn run(&mut self) -> InterpretResult {
        self.stack.clear();
        self.globals.clear();
        self.globals.resize(256, Option::None);
        let interpret_result = {
            loop {
                #[cfg(debug_assertions)]
                {
                    // println!("-----------------");
                    // println!("{:^16}", "--stack--");
                    // for value in self.stack.iter() {
                    //     match value {
                    //         Value::String(s) => println!("{:^16}", format!("\"{}\"", s)),
                    //         _ => println!("{:^16}", value.to_string()),
                    //     }
                    // }
                    // self.curr_chunk().disassemble_instruction(self.ip);
                }
                let instruction: OpCode = self.read_byte().into();
                match instruction {
                    OpCode::Return => {
                        // exit interpret
                        break InterpretResult::Success;
                    }
                    OpCode::Nil => self.stack.push(Value::Nil),
                    OpCode::True => self.stack.push(Value::Bool(true)),
                    OpCode::False => self.stack.push(Value::Bool(false)),
                    OpCode::Number => push_constant!(self, Number, read_number),
                    OpCode::String => push_constant!(self, String, read_string),
                    OpCode::Equal => binary_op!(self, |x: Value, y: Value| x.equal(&y)),
                    OpCode::Greater => binary_op!(self, |x: Value, y: Value| x.greater(&y)),
                    OpCode::Less => binary_op!(self, |x: Value, y: Value| x.less(&y)),
                    OpCode::Not => unary_op!(self, |x: Value| !x),
                    OpCode::Negate => unary_op!(self, |x: Value| -x),
                    OpCode::Addition => binary_op!(self, |x: Value, y: Value| x + y),
                    OpCode::Subtract => binary_op!(self, |x: Value, y: Value| x - y),
                    OpCode::Multiply => binary_op!(self, |x: Value, y: Value| x * y),
                    OpCode::Divide => binary_op!(self, |x: Value, y: Value| x / y),
                    OpCode::Print => println!("{}", self.stack.pop().unwrap().to_string()),
                    OpCode::Pop => {
                        let _ = self.stack.pop().unwrap();
                    }
                    OpCode::DefineGlobal => {
                        let index: usize = self.read_byte() as usize;
                        let slot: usize = self.curr_chunk().read_variable(index).clone();
                        let value: Value = self.stack.pop().unwrap();
                        if slot >= self.globals.len() {
                            self.runtime_error(&format!(
                                "Global variable slot only in 0 ~ {}",
                                self.globals.len() - 1
                            ));
                            break InterpretResult::RuntimeError;
                        }
                        match &self.globals[slot] {
                            Some(_) => {
                                self.runtime_error("Redefine global variable");
                                break InterpretResult::RuntimeError;
                            }
                            None => self.globals[slot] = Some(value),
                        }
                    }
                    OpCode::GetGlobal => {
                        let index: usize = self.read_byte() as usize;
                        let slot: usize = self.curr_chunk().read_variable(index).clone();
                        match &self.globals[slot] {
                            Some(v) => {
                                self.stack.push(v.clone());
                            }
                            None => {
                                self.runtime_error(&format!(
                                    "Undefined variable in global slot[{}]",
                                    slot
                                ));
                                break InterpretResult::RuntimeError;
                            }
                        }
                    }
                    OpCode::SetGlobal => {
                        let index: usize = self.read_byte() as usize;
                        let slot: usize = self.curr_chunk().read_variable(index).clone();
                        match &self.globals[slot] {
                            Some(_) => {
                                let value: &Value = self.stack.last().unwrap();
                                self.globals[slot] = Some(value.clone());
                            }
                            None => {
                                self.runtime_error(&format!(
                                    "Undefined variable in global slot[{}]",
                                    slot
                                ));
                                break InterpretResult::RuntimeError;
                            }
                        }
                    }
                    OpCode::GetLocal => {
                        let index: usize = self.read_byte() as usize;
                        let slot: usize = self.curr_chunk().read_variable(index).clone();
                        match self.stack.get(slot) {
                            Some(v) => {
                                self.stack.push(v.clone());
                            }
                            None => {
                                self.runtime_error(&format!(
                                    "Undefined variable in stack slot[{}]",
                                    slot
                                ));
                                break InterpretResult::RuntimeError;
                            }
                        }
                    }
                    OpCode::SetLocal => {
                        let index: usize = self.read_byte() as usize;
                        let slot: usize = self.curr_chunk().read_variable(index).clone();
                        match self.stack.get(slot) {
                            Some(_) => {
                                let value: &Value = self.stack.last().unwrap();
                                self.stack[slot] = value.clone();
                            }
                            None => {
                                self.runtime_error(&format!(
                                    "Undefined variable in stack slot[{}]",
                                    slot
                                ));
                                break InterpretResult::RuntimeError;
                            }
                        }
                    }
                    OpCode::JumpFalse => {
                        let jump_offset: usize = self.read_short() as usize;
                        let value: &Value = self.stack.last().unwrap();
                        if !value.bool_value() {
                            self.curr_ip_inc(jump_offset);
                        }
                    }
                    OpCode::Jump => {
                        let jump_offset: usize = self.read_short() as usize;
                        self.curr_ip_inc(jump_offset);
                    }
                    OpCode::JumpBack => {
                        let jump_offset: usize = self.read_short() as usize;
                        self.curr_ip_dec(jump_offset);
                    }
                }
            }
        };
        interpret_result
    }

    fn curr_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn curr_chunk(&self) -> Rc<Chunk> {
        self.curr_frame().function.chunk.clone()
    }

    fn curr_ip(&self) -> usize {
        *self.curr_frame().ip.borrow()
    }

    fn curr_ip_inc(&self, amount: usize) {
        *self.curr_frame().ip.borrow_mut() += amount;
    }

    fn curr_ip_dec(&self, amount: usize) {
        *self.curr_frame().ip.borrow_mut() -= amount;
    }

    fn read_byte(&mut self) -> u8 {
        let byte: u8 = self.curr_chunk().read_code(self.curr_ip());
        self.curr_ip_inc(1);
        byte
    }

    fn read_short(&mut self) -> u16 {
        let low: u16 = self.curr_chunk().read_code(self.curr_ip()).into();
        let high: u16 = self.curr_chunk().read_code(self.curr_ip() + 1).into();
        self.curr_ip_inc(2);
        low | (high << 8)
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!(
            "{} : [line {}] in script",
            message,
            self.curr_chunk().read_line(self.curr_ip() - 1).clone()
        );
        self.reset_stack()
    }
}
