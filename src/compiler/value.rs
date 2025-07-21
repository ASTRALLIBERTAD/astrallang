#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(i64),
    Bool(bool),
    Str(String),
    Null,
}
