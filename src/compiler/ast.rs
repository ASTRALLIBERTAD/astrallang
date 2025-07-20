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
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
    pub(crate) params: Vec<String>,
}