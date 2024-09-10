#[derive(Clone, Copy)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Bool(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
            Value::Number(n) => n.to_string(),
        }
    }
}

impl std::ops::Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Value::Number(n) => Self::Number(-n),
            _ => panic!("Invalid operation"),
        }
    }
}

impl std::ops::Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Self::Number(x + y),
            _ => panic!("Invalid operation"),
        }
    }
}

impl std::ops::Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Self::Number(x - y),
            _ => panic!("Invalid operation"),
        }
    }
}

impl std::ops::Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Self::Number(x * y),
            _ => panic!("Invalid operation"),
        }
    }
}

impl std::ops::Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Self::Number(x / y),
            _ => panic!("Invalid operation"),
        }
    }
}

impl Value {
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }
}
