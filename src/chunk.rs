pub enum OpCode {
    Return = 0,
    Constant,
    Nil,
    True,
    False,
    Not,
    Negate,
    Addition,
    Subtract,
    Multiply,
    Divide,
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
            2 => Self::Nil,
            3 => Self::True,
            4 => Self::False,
            5 => Self::Not,
            6 => Self::Negate,
            7 => Self::Addition,
            8 => Self::Subtract,
            9 => Self::Multiply,
            10 => Self::Divide,
            _ => unimplemented!("Invalid OpCode"),
        }
    }
}

impl ToString for OpCode {
    fn to_string(&self) -> String {
        match self {
            Self::Return => "OP_RETURN".to_string(),
            Self::Constant => "OP_CONSTANT".to_string(),
            Self::Nil => "OP_NIL".to_string(),
            Self::True => "OP_TRUE".to_string(),
            Self::False => "OP_FALSE".to_string(),
            Self::Not => "OP_NOT".to_string(),
            Self::Negate => "OP_NEGATE".to_string(),
            Self::Addition => "OP_ADDITION".to_string(),
            Self::Subtract => "OP_SUBTRACT".to_string(),
            Self::Multiply => "OP_MULTIPLY".to_string(),
            Self::Divide => "OP_DIVIDE".to_string(),
        }
    }
}

pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<f64>,
    lines: Vec<u32>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::<u8>::new(),
            constants: Vec::<f64>::new(),
            lines: Vec::<u32>::new(),
        }
    }

    pub fn write_code(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, number: f64) -> Result<usize, String> {
        match self.constants.len() < 0x100 {
            true => {
                self.constants.push(number);
                Ok(self.constants.len() - 1)
            }
            false => Err("Too many constants in one chunk".to_string()),
        }
    }

    pub fn clear(&mut self) {
        self.code.clear();
        self.constants.clear();
        self.lines.clear();
    }

    pub fn read_code(&self, offset: usize) -> u8 {
        self.code[offset]
    }

    pub fn read_constant(&self, offset: usize) -> f64 {
        self.constants[offset]
    }

    pub fn read_line(&self, offset: usize) -> u32 {
        self.lines[offset]
    }
}

pub trait Disassemble {
    fn disassemble(&self, disassemble_name: &str);
    fn disassemble_instruction(&self, offset: usize) -> usize;
    fn simple_instruction(&self, instruction: OpCode, offset: usize) -> usize;
    fn constant_instruction(&self, instruction: OpCode, offset: usize) -> usize;
}

impl Disassemble for Chunk {
    fn disassemble(&self, disassemble_name: &str) {
        println!("== {} ==", disassemble_name);

        let mut offset: usize = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        let instruction: OpCode = self.code[offset].into();
        match instruction {
            OpCode::Return => self.simple_instruction(instruction, offset),
            OpCode::Constant => self.constant_instruction(instruction, offset),
            OpCode::Nil => self.simple_instruction(instruction, offset),
            OpCode::True => self.simple_instruction(instruction, offset),
            OpCode::False => self.simple_instruction(instruction, offset),
            OpCode::Not => self.simple_instruction(instruction, offset),
            OpCode::Negate => self.simple_instruction(instruction, offset),
            OpCode::Addition => self.simple_instruction(instruction, offset),
            OpCode::Subtract => self.simple_instruction(instruction, offset),
            OpCode::Multiply => self.simple_instruction(instruction, offset),
            OpCode::Divide => self.simple_instruction(instruction, offset),
        }
    }

    fn simple_instruction(&self, instruction: OpCode, offset: usize) -> usize {
        println!(
            "line:{}  code:{}    {}    ",
            self.lines[offset],
            offset,
            instruction.to_string()
        );
        offset + 1
    }

    fn constant_instruction(&self, instruction: OpCode, offset: usize) -> usize {
        let constant_offset: usize = self.code[offset + 1].into();
        println!(
            "line:{}  code:{}    {}    {}",
            self.lines[offset],
            offset,
            instruction.to_string(),
            self.constants[constant_offset].to_string()
        );
        offset + 2
    }
}
