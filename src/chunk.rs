use crate::value::*;

pub enum OpCode {
    Return = 0,
    Constant = 1,
    Negate = 2,
    Addition = 3,
    Subtract = 4,
    Multiply = 5,
    Divide = 6,
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        value as u8
    }
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Return,
            1 => Self::Constant,
            2 => Self::Negate,
            3 => Self::Addition,
            4 => Self::Subtract,
            5 => Self::Multiply,
            6 => Self::Divide,
            _ => unimplemented!("Invalid OpCode"),
        }
    }
}

impl ToString for OpCode {
    fn to_string(&self) -> String {
        match self {
            OpCode::Return => "OP_RETURN".to_string(),
            OpCode::Constant => "OP_CONSTANT".to_string(),
            OpCode::Negate => "OP_NEGATE".to_string(),
            OpCode::Addition => "OP_ADDITION".to_string(),
            OpCode::Subtract => "OP_SUBTRACT".to_string(),
            OpCode::Multiply => "OP_MULTIPLY".to_string(),
            OpCode::Divide => "OP_DIVIDE".to_string(),
        }
    }
}

pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    lines: Vec<u32>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::<u8>::new(),
            constants: Vec::<Value>::new(),
            lines: Vec::<u32>::new(),
        }
    }

    pub fn write_code(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        self.constants.len() as u8 - 1
    }

    pub fn clear(&mut self) {
        self.code.clear();
        self.constants.clear();
        self.lines.clear();
    }

    pub fn read_code(&self, idx: usize) -> u8 {
        self.code[idx]
    }

    pub fn read_constant(&self, idx: usize) -> Value {
        self.constants[idx]
    }

    pub fn disassemble(&self, disassemble_name: &str) {
        println!("== {} ==", disassemble_name);

        let mut offset: usize = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        let instruction: OpCode = self.code[offset].into();
        match instruction {
            OpCode::Return => self.simple_instruction(instruction, offset),
            OpCode::Constant => self.constant_instruction(instruction, offset),
            OpCode::Negate => self.simple_instruction(instruction, offset),
            OpCode::Addition => self.simple_instruction(instruction, offset),
            OpCode::Subtract => self.simple_instruction(instruction, offset),
            OpCode::Multiply => self.simple_instruction(instruction, offset),
            OpCode::Divide => self.simple_instruction(instruction, offset),
        }
    }

    pub fn simple_instruction(&self, instruction: OpCode, offset: usize) -> usize {
        println!(
            "line:{}  code:{}    {}    ",
            self.lines[offset],
            offset,
            instruction.to_string()
        );
        offset + 1
    }

    pub fn constant_instruction(&self, instruction: OpCode, offset: usize) -> usize {
        let constant_idx: usize = self.code[offset + 1].into();
        println!(
            "line:{}  code:{}    {}    {}",
            self.lines[offset],
            offset,
            instruction.to_string(),
            self.constants[constant_idx]
        );
        offset + 2
    }
}
