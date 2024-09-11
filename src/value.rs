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
    type Output = Result<Self, &'static str>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Number(n) => Ok(Self::Number(-n)),
            _ => Err("Neg operation must be number"),
        }
    }
}

impl std::ops::Add for Value {
    type Output = Result<Self, &'static str>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Ok(Self::Number(x + y)),
            _ => Err("Add operation must between tow numbers"),
        }
    }
}

impl std::ops::Sub for Value {
    type Output = Result<Self, &'static str>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Ok(Self::Number(x - y)),
            _ => Err("Sub operation must between tow numbers"),
        }
    }
}

impl std::ops::Mul for Value {
    type Output = Result<Self, &'static str>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Ok(Self::Number(x * y)),
            _ => Err("Mul operation must between tow numbers"),
        }
    }
}

impl std::ops::Div for Value {
    type Output = Result<Self, &'static str>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Ok(Self::Number(x / y)),
            _ => Err("Div operation must between tow numbers"),
        }
    }
}

impl std::ops::Not for Value {
    type Output = Result<Self, &'static str>;

    fn not(self) -> Self::Output {
        match self {
            Value::Bool(b) => Ok(Self::Bool(!b)),
            Value::Nil => Ok(Self::Bool(true)),
            Value::Number(n) => Ok(Self::Bool(n == 0.0)),
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

    pub fn equal(&self, rhs: &Self) -> Result<Self, &'static str> {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Ok(Self::Bool(x == y)),
            (Value::Bool(x), Value::Bool(y)) => Ok(Self::Bool(x == y)),
            (Value::Nil, Value::Nil) => Ok(Self::Bool(0 == 0)),
            _ => Err("Equal operation must between tow numbers"),
        }
    }

    pub fn not_equal(&self, rhs: &Self) -> Result<Self, &'static str> {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Ok(Self::Bool(x != y)),
            (Value::Bool(x), Value::Bool(y)) => Ok(Self::Bool(x != y)),
            (Value::Nil, Value::Nil) => Ok(Self::Bool(0 != 0)),
            _ => Err("Not Equal operation must between tow numbers"),
        }
    }

    pub fn less(&self, rhs: &Self) -> Result<Self, &'static str> {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Ok(Self::Bool(x < y)),
            (Value::Bool(x), Value::Bool(y)) => Ok(Self::Bool(x < y)),
            (Value::Nil, Value::Nil) => Ok(Self::Bool(0 < 0)),
            _ => Err("Less operation must between tow numbers"),
        }
    }

    pub fn less_equal(&self, rhs: &Self) -> Result<Self, &'static str> {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Ok(Self::Bool(x <= y)),
            (Value::Bool(x), Value::Bool(y)) => Ok(Self::Bool(x <= y)),
            (Value::Nil, Value::Nil) => Ok(Self::Bool(0 <= 0)),
            _ => Err("Less Equal operation must between tow numbers"),
        }
    }

    pub fn greater(&self, rhs: &Self) -> Result<Self, &'static str> {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Ok(Self::Bool(x > y)),
            (Value::Bool(x), Value::Bool(y)) => Ok(Self::Bool(x > y)),
            (Value::Nil, Value::Nil) => Ok(Self::Bool(0 > 0)),
            _ => Err("Greater operation must between tow numbers"),
        }
    }

    pub fn greater_equal(&self, rhs: &Self) -> Result<Self, &'static str> {
        match (self, rhs) {
            (Value::Number(x), Value::Number(y)) => Ok(Self::Bool(x >= y)),
            (Value::Bool(x), Value::Bool(y)) => Ok(Self::Bool(x >= y)),
            (Value::Nil, Value::Nil) => Ok(Self::Bool(0 >= 0)),
            _ => Err("Greater Equal operation must between tow numbers"),
        }
    }
}
