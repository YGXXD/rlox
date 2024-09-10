#[derive(PartialEq, Clone)]
pub enum TokenType {
    // 单字符词法
    LeftParen = 0,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // 一或两字符词法
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // 字面量
    Identifier,
    String,
    Number,
    // 关键字
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    // 特殊词
    Eof,
    Error,
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Error
    }
}

impl From<TokenType> for usize {
    fn from(value: TokenType) -> Self {
        value as usize
    }
}

impl ToString for TokenType {
    fn to_string(&self) -> String {
        match self {
            TokenType::LeftParen => "LeftParen".to_string(),
            TokenType::RightParen => "RightParen".to_string(),
            TokenType::LeftBrace => "LeftBrace".to_string(),
            TokenType::RightBrace => "RightBrace".to_string(),
            TokenType::Comma => "Comma".to_string(),
            TokenType::Dot => "Dot".to_string(),
            TokenType::Minus => "Minus".to_string(),
            TokenType::Plus => "Plus".to_string(),
            TokenType::Semicolon => "Semicolon".to_string(),
            TokenType::Slash => "Slash".to_string(),
            TokenType::Star => "Star".to_string(),
            TokenType::Bang => "Bang".to_string(),
            TokenType::BangEqual => "BangEqual".to_string(),
            TokenType::Equal => "Equal".to_string(),
            TokenType::EqualEqual => "EqualEqual".to_string(),
            TokenType::Greater => "Greater".to_string(),
            TokenType::GreaterEqual => "GreaterEqual".to_string(),
            TokenType::Less => "Less".to_string(),
            TokenType::LessEqual => "LessEqual".to_string(),
            TokenType::Identifier => "Identifier".to_string(),
            TokenType::String => "String".to_string(),
            TokenType::Number => "Number".to_string(),
            TokenType::And => "And".to_string(),
            TokenType::Class => "Class".to_string(),
            TokenType::Else => "Else".to_string(),
            TokenType::False => "False".to_string(),
            TokenType::For => "For".to_string(),
            TokenType::Fun => "Fun".to_string(),
            TokenType::If => "If".to_string(),
            TokenType::Nil => "Nil".to_string(),
            TokenType::Or => "Or".to_string(),
            TokenType::Print => "Print".to_string(),
            TokenType::Return => "Return".to_string(),
            TokenType::Super => "Super".to_string(),
            TokenType::This => "This".to_string(),
            TokenType::True => "True".to_string(),
            TokenType::Var => "Var".to_string(),
            TokenType::While => "While".to_string(),
            TokenType::Eof => "Eof".to_string(),
            TokenType::Error => "Error".to_string(),
        }
    }
}

#[derive(Default, Clone)]
pub struct Token {
    pub r#type: TokenType,
    pub lexeme: String,
    pub line: u32,
}

impl Token {
    pub fn error(&self, message: &str) {
        eprint!("[line {}] Error ", self.line);
        match self.r#type {
            TokenType::Eof => eprint!("at end"),
            TokenType::Error => eprint!("{}", self.lexeme),
            _ => eprint!("at {}", self.lexeme),
        }
        eprintln!(" : {}", message);
        std::process::exit(65);
    }

    pub fn info(&self, message: &str) {
        print!("[line {}] Info ", self.line);
        match self.r#type {
            TokenType::Eof => print!("at end"),
            TokenType::Error => print!("{}", self.lexeme),
            _ => eprint!("at {}", self.lexeme),
        }
        println!(" : {}", message);
    }
}
