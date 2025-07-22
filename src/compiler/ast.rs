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

#[derive(Clone, Debug)]
pub enum Expr {
    Number(i64),
    Ident(String),
    BinaryOp(Box<Expr>, BinaryOperator, Box<Expr>),
    Call(String, Vec<Expr>),
    UnaryOp(UnaryOperator, Box<Expr>),
    Bool(bool),
    String(String),
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Let(String, Expr),
    Print(Expr),
    Function(Function),
    Return(Option<Expr>),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    Expression(Expr),
}
#[derive(Clone, Debug)]
pub enum ParamType {
    String,
    Bool,
    Number,
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