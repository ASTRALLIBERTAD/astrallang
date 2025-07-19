use crate::compiler::ast::*;
use crate::compiler::lexer::Token;
use crate::compiler::ast::Stmt;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while *self.peek() != Token::EOF {
            if *self.peek() == Token::Fn {
                let func = self.parse_function();
                statements.push(Stmt::Function(func));
            } else {
                statements.push(self.parse_stmt());
            }
        }
        statements
    }

    pub fn parse_function(&mut self) -> Function {
        self.expect(Token::Fn);
        let name = match self.next() {
            Token::Ident(s) => s,
            _ => panic!("Expected function name"),
        };
        self.expect(Token::LParen);
        self.expect(Token::RParen);
        self.expect(Token::Arrow);
        self.expect(Token::Ident("void".into()));
        self.expect(Token::LBrace);

        let mut body = Vec::new();
        while *self.peek() != Token::RBrace {
            body.push(self.parse_stmt());
        }
        self.expect(Token::RBrace);

        Function { name, body }
    }

    fn parse_stmt(&mut self) -> Stmt {
        match self.next() {
            Token::Let => {
                let name = match self.next() {
                    Token::Ident(s) => s,
                    _ => panic!("Expected identifier after let"),
                };
                self.expect(Token::Equal);
                let expr = self.parse_expr();
                self.expect(Token::Semicolon);
                Stmt::Let(name, expr)
            }
            Token::Ident(s) if s == "print" => {
                self.expect(Token::LParen);
                let expr = self.parse_expr();
                self.expect(Token::RParen);
                self.expect(Token::Semicolon);
                Stmt::Print(expr)
            }
            Token::Return => {
                let expr = self.parse_expr();
                self.expect(Token::Semicolon);
                Stmt::Return(expr)
            }
            other => panic!("Unknown statement: {:?}", other),
        }
    }

    // 🔢 Parse expressions with precedence
    fn parse_expr(&mut self) -> Expr {
        self.parse_term()
    }

    fn parse_term(&mut self) -> Expr {
        let mut expr = self.parse_factor();

        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let op = match self.next() {
                Token::Plus => "+".to_string(),
                Token::Minus => "-".to_string(),
                _ => unreachable!(),
            };
            let right = self.parse_factor();
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }

        expr
    }

    fn parse_factor(&mut self) -> Expr {
        let mut expr = self.parse_primary();

        while matches!(self.peek(), Token::Star | Token::Slash) {
            let op = match self.next() {
                Token::Star => "*".to_string(),
                Token::Slash => "/".to_string(),
                _ => unreachable!(),
            };
            let right = self.parse_primary();
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }

        expr
    }

    fn parse_primary(&mut self) -> Expr {
        match self.next() {
            Token::Number(n) => Expr::Number(n),
            Token::Ident(s) => Expr::Ident(s),
            Token::LParen => {
                let expr = self.parse_expr();
                self.expect(Token::RParen);
                expr
            }
            other => panic!("Unexpected token in expression: {:?}", other),
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::EOF)
    }

    fn next(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF);
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: Token) {
        let actual = self.next();
        if std::mem::discriminant(&actual) != std::mem::discriminant(&expected) {
            panic!("Expected {:?}, got {:?}", expected, actual);
        }
    }
}
