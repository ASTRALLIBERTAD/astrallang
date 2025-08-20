#[derive(Clone, Debug)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

#[derive(Clone, Debug)]
pub enum UnaryOperator {
    Minus,
    Not,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Usize,
    Bool,
    String,
    Str,
    Any
}

#[derive(Clone, Debug)]
pub enum Pattern {
    Value(Value),
    Wildcard,
    Identifier(String)
    
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal(Value),
    Ident(String),
    BinaryOp(Box<Expr>, BinaryOperator, Box<Expr>),
    Call(String, Vec<Expr>),
    UnaryOp(UnaryOperator, Box<Expr>),
    Cast(Box<Expr>, Type),
    Match (Box<Expr>, Vec<(Pattern, Expr)>),
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Let(String, Type, Expr),
    Print(Expr),
    Function(Function),
    Return(Option<Expr>),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    Expression(Expr),
    Block(Vec<Stmt>),
}
#[derive(Clone, Debug)]
pub enum ParamType {
    String,
    Bool,
    Number(Type),
    Any
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub param_type: ParamType,
    pub default_value: Option<Expr>,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}
use std::str::FromStr;
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    Usize(usize),
    Bool(bool),
    String(String),
    Str(&'static str)
}

// Paste this below the enum
impl FromStr for Value {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("true") {
            return Ok(Value::Bool(true));
        } else if s.eq_ignore_ascii_case("false") {
            return Ok(Value::Bool(false));
        }

        if let Ok(v) = s.parse::<i8>() {
            return Ok(Value::I8(v));
        }
        if let Ok(v) = s.parse::<i16>() {
            return Ok(Value::I16(v));
        }
        if let Ok(v) = s.parse::<i32>() {
            return Ok(Value::I32(v));
        }
        if let Ok(v) = s.parse::<i64>() {
            return Ok(Value::I64(v));
        }
        if let Ok(v) = s.parse::<i128>() {
            return Ok(Value::I128(v));
        }

        if let Ok(v) = s.parse::<u8>() {
            return Ok(Value::U8(v));
        }
        if let Ok(v) = s.parse::<u16>() {
            return Ok(Value::U16(v));
        }
        if let Ok(v) = s.parse::<u32>() {
            return Ok(Value::U32(v));
        }
        if let Ok(v) = s.parse::<u64>() {
            return Ok(Value::U64(v));
        }
        if let Ok(v) = s.parse::<u128>() {
            return Ok(Value::U128(v));
        }

        if let Ok(v) = s.parse::<f32>() {
            return Ok(Value::F32(v));
        }
        if let Ok(v) = s.parse::<f64>() {
            return Ok(Value::F64(v));
        }

        if let Ok(v) = s.parse::<usize>() {
            return Ok(Value::Usize(v));
        }

        // if let Ok(s) = s.parse::<&str>() {
        //     return Ok(Value::Str(s));
        // }

        Ok(Value::String(s.to_string()))
    }
}



impl Value {

    pub fn get_type(&self) -> Type {
        match self {
            Value::I8(_) => Type::I8,
            Value::I16(_) => Type::I16,
            Value::I32(_) => Type::I32,
            Value::I64(_) => Type::I64,
            Value::I128(_) => Type::I128,
            Value::U8(_) => Type::U8,
            Value::U16(_) => Type::U16,
            Value::U32(_) => Type::U32,
            Value::U64(_) => Type::U64,
            Value::U128(_) => Type::U128,
            Value::F32(_) => Type::F32,
            Value::F64(_) => Type::F64,
            Value::Usize(_) => Type::Usize,
            Value::Bool(_) => Type::Bool,
            Value::Str(_) => Type::Str,
            Value::String(_) => Type::String,
        }
    }

    pub fn to_string(&self) -> String {
        
        match self {
            Value::I8(n) => n.to_string(),
            Value::I16(n) => n.to_string(),
            Value::I32(n) => n.to_string(),
            Value::I64(n) => n.to_string(),
            Value::I128(n) => n.to_string(),
            Value::U8(n) => n.to_string(),
            Value::U16(n) => n.to_string(),
            Value::U32(n) => n.to_string(),
            Value::U64(n) => n.to_string(),
            Value::U128(n) => n.to_string(),
            Value::F32(n) => n.to_string(),
            Value::F64(n) => n.to_string(),
            Value::Usize(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => s.to_string(),
            Value::String(s) => s.clone(),
        }
    }

    pub fn matches_type(&self, typ: &Type) -> bool {
        match (self, typ) {
            (Value::I8(_), Type::I8) => true,
            (Value::I16(_), Type::I16) => true,
            (Value::I32(_), Type::I32) => true,
            (Value::I64(_), Type::I64) => true,
            (Value::I128(_), Type::I128) => true,
            (Value::U8(_), Type::U8) => true,
            (Value::U16(_), Type::U16) => true,
            (Value::U32(_), Type::U32) => true,
            (Value::U64(_), Type::U64) => true,
            (Value::U128(_), Type::U128) => true,
            (Value::F32(_), Type::F32) => true,
            (Value::F64(_), Type::F64) => true,
            (Value::Bool(_), Type::Bool) => true,
            (Value::Str(_), Type::Str) => true,
            (Value::String(_), Type::String) => true,
            _ => false,
        }
    }

    pub fn cast_to(&self, target_type: &Type) -> Option<Value> {
        // If source and target types are the same, return self clone
        if &self.get_type() == target_type {
            return Some(self.clone());
        }

        match (self, target_type) {
            (Value::I8(n), Type::I16) => Some(Value::I16(*n as i16)),
            (Value::I8(n), Type::I32) => Some(Value::I32(*n as i32)),
            (Value::I8(n), Type::I64) => Some(Value::I64(*n as i64)),
            (Value::I8(n), Type::I128) => Some(Value::I128(*n as i128)),
            (Value::U8(n), Type::U16) => Some(Value::U16(*n as u16)),
            (Value::U8(n), Type::U32) => Some(Value::U32(*n as u32)),
            (Value::U8(n), Type::U64) => Some(Value::U64(*n as u64)),
            (Value::U8(n), Type::U128) => Some(Value::U128(*n as u128)),
            (Value::F32(n), Type::F64) => Some(Value::F64(*n as f64)),
            (Value::String(s), Type::Str) => Some(Value::Str(Box::leak(s.clone().into_boxed_str()))),
            (Value::Str(s), Type::String) => Some(Value::String(s.to_string())),
            _ => None,
        }
    }

}
