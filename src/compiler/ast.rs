#[derive(Debug)]
pub enum Expr {
    Number(i64),
    Ident(String),
    BinaryOp(Box<Expr>, String, Box<Expr>),
}

#[derive(Debug)]
pub enum Stmt {
    Let(String, Expr),
    Print(Expr),
    Function(Function),
    Return(Expr),
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
}
