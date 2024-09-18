pub enum OpCode {
    Return = 0,
    Nil,
    True,
    False,
    Number,
    String,
    Not,
    Negate,
    Addition,
    Subtract,
    Multiply,
    Divide,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
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
            1 => Self::Nil,
            2 => Self::True,
            3 => Self::False,
            4 => Self::Number,
            5 => Self::String,
            6 => Self::Not,
            7 => Self::Negate,
            8 => Self::Addition,
            9 => Self::Subtract,
            10 => Self::Multiply,
            11 => Self::Divide,
            12 => Self::Equal,
            13 => Self::Greater,
            14 => Self::Less,
            15 => Self::Print,
            16 => Self::Pop,
            17 => Self::DefineGlobal,
            18 => Self::GetGlobal,
            19 => Self::SetGlobal,
            _ => unimplemented!("Invalid OpCode"),
        }
    }
}

impl ToString for OpCode {
    fn to_string(&self) -> String {
        match self {
            Self::Return => "OP_RETURN".to_string(),
            Self::Nil => "OP_NIL".to_string(),
            Self::True => "OP_TRUE".to_string(),
            Self::False => "OP_FALSE".to_string(),
            Self::Number => "OP_NUMBER".to_string(),
            Self::String => "OP_STRING".to_string(),
            Self::Not => "OP_NOT".to_string(),
            Self::Negate => "OP_NEGATE".to_string(),
            Self::Addition => "OP_ADDITION".to_string(),
            Self::Subtract => "OP_SUBTRACT".to_string(),
            Self::Multiply => "OP_MULTIPLY".to_string(),
            Self::Divide => "OP_DIVIDE".to_string(),
            Self::Equal => "OP_EQUAL".to_string(),
            Self::Greater => "OP_GREATER".to_string(),
            Self::Less => "OP_LESS".to_string(),
            Self::Print => "OP_PRINT".to_string(),
            Self::Pop => "OP_POP".to_string(),
            Self::DefineGlobal => "OP_DEFINE_GLOBAL".to_string(),
            Self::GetGlobal => "OP_GET_GLOBAL".to_string(),
            Self::SetGlobal => "OP_SET_GLOBAL".to_string(),
        }
    }
}

pub struct Chunk {
    code: Vec<u8>,
    numbers: Vec<f64>,
    strings: Vec<String>,
    identifiers: Vec<String>,
    lines: Vec<u32>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::<u8>::new(),
            numbers: Vec::<f64>::new(),
            strings: Vec::<String>::new(),
            identifiers: Vec::<String>::new(),
            lines: Vec::<u32>::new(),
        }
    }

    pub fn write_code(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_number(&mut self, number: f64) -> Result<usize, String> {
        match self.numbers.len() < 0x100 {
            true => {
                self.numbers.push(number);
                Ok(self.numbers.len() - 1)
            }
            false => Err("Too many numbers in one chunk".to_string()),
        }
    }

    pub fn add_string(&mut self, string: String) -> Result<usize, String> {
        match self.strings.len() < 0x100 {
            true => {
                self.strings.push(string);
                Ok(self.strings.len() - 1)
            }
            false => Err("Too many strings in one chunk".to_string()),
        }
    }

    pub fn add_identifier(&mut self, identifier: String) -> Result<usize, String> {
        match self.identifiers.len() < 0x100 {
            true => {
                self.identifiers.push(identifier);
                Ok(self.identifiers.len() - 1)
            }
            false => Err("Too many identifiers in one chunk".to_string()),
        }
    }

    pub fn clear(&mut self) {
        self.code.clear();
        self.numbers.clear();
        self.strings.clear();
        self.identifiers.clear();
        self.lines.clear();
    }

    pub fn read_code(&self, offset: usize) -> u8 {
        self.code[offset]
    }

    pub fn read_number(&self, offset: usize) -> &f64 {
        &self.numbers[offset]
    }

    pub fn read_string(&self, offset: usize) -> &String {
        &self.strings[offset]
    }

    pub fn read_identifier(&self, offset: usize) -> &String {
        &self.identifiers[offset]
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
            OpCode::Nil => self.simple_instruction(instruction, offset),
            OpCode::True => self.simple_instruction(instruction, offset),
            OpCode::False => self.simple_instruction(instruction, offset),
            OpCode::Number => self.constant_instruction(instruction, offset),
            OpCode::String => self.constant_instruction(instruction, offset),
            OpCode::Not => self.simple_instruction(instruction, offset),
            OpCode::Negate => self.simple_instruction(instruction, offset),
            OpCode::Addition => self.simple_instruction(instruction, offset),
            OpCode::Subtract => self.simple_instruction(instruction, offset),
            OpCode::Multiply => self.simple_instruction(instruction, offset),
            OpCode::Divide => self.simple_instruction(instruction, offset),
            OpCode::Equal => self.simple_instruction(instruction, offset),
            OpCode::Greater => self.simple_instruction(instruction, offset),
            OpCode::Less => self.simple_instruction(instruction, offset),
            OpCode::Print => self.simple_instruction(instruction, offset),
            OpCode::Pop => self.simple_instruction(instruction, offset),
            OpCode::DefineGlobal => self.constant_instruction(instruction, offset),
            OpCode::GetGlobal => self.constant_instruction(instruction, offset),
            OpCode::SetGlobal => self.constant_instruction(instruction, offset),
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
            match instruction {
                OpCode::Number => self.numbers[constant_offset].to_string(),
                OpCode::String => format!("\"{}\"", self.strings[constant_offset]),
                OpCode::DefineGlobal => format!("{}", self.identifiers[constant_offset]),
                OpCode::GetGlobal => format!("{}", self.identifiers[constant_offset]),
                OpCode::SetGlobal => format!("{}", self.identifiers[constant_offset]),
                _ => "".to_string(),
            }
        );
        offset + 2
    }
}
