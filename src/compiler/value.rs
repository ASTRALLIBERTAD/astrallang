#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(i64),
    Bool(bool),
    Str(String),
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => s.clone(),
        }
    }
}
