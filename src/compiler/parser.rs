use crate::compiler::ast::*;
use crate::compiler::lexer::Token;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub token: Token,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error: {} (got token: {})", self.message, self.token)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while *self.peek() != Token::EOF {
            if *self.peek() == Token::Fn {
                let func = self.parse_function()?;
                statements.push(Stmt::Function(func));
            } else {
                statements.push(self.parse_stmt()?);
            }
        }
        Ok(statements)
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        self.expect(Token::Fn)?;
        let name = match self.next() {
            Token::Ident(s) => s,
            token => return Err(ParseError {
                message: "Expected function name".to_string(),
                token,
            }),
        };

        self.expect(Token::LParen)?;

        let mut params = Vec::new();
        while *self.peek() != Token::RParen {
            if let Token::Ident(param) = self.next() {
                params.push(Param { name: param, default_value: None, param_type: ParamType::Any });
                if *self.peek() == Token::Comma {
                    self.next(); // consume comma
                } else if *self.peek() != Token::RParen {
                    return Err(ParseError {
                        message: "Expected ',' or ')' in parameter list".to_string(),
                        token: self.peek().clone(),
                    });
                }
            } else {
                return Err(ParseError {
                    message: "Expected parameter name".to_string(),
                    token: self.peek().clone(),
                });
            }
        }

        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;

        let mut body = Vec::new();
        while *self.peek() != Token::RBrace {
            body.push(self.parse_stmt()?);
        }
        self.expect(Token::RBrace)?;

        Ok(Function { name, params, body })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek().clone() {
            Token::Let => {
                self.next(); // consume let
                let name = match self.next() {
                    Token::Ident(s) => s,
                    token => return Err(ParseError {
                        message: "Expected identifier after let".to_string(),
                        token,
                    }),
                };
                self.expect(Token::Equal)?;
                let expr = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Let(name, expr))
            }
            Token::Print => {
                self.next(); // consume print
                self.expect(Token::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Print(expr))
            }
            Token::Return => {
                self.next(); // consume return
                let expr = if *self.peek() == Token::Semicolon {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Return(expr))
            }
            Token::If => {
                self.next(); // consume if
                self.expect(Token::LParen)?;
                let condition = self.parse_expr()?;
                self.expect(Token::RParen)?;
                self.expect(Token::LBrace)?;

                let mut then_body = Vec::new();
                while *self.peek() != Token::RBrace {
                    then_body.push(self.parse_stmt()?);
                }
                self.expect(Token::RBrace)?;

                let else_body = if *self.peek() == Token::Else {
                    self.next(); // consume else
                    self.expect(Token::LBrace)?;
                    let mut body = Vec::new();
                    while *self.peek() != Token::RBrace {
                        body.push(self.parse_stmt()?);
                    }
                    self.expect(Token::RBrace)?;
                    Some(body)
                } else {
                    None
                };

                Ok(Stmt::If(condition, then_body, else_body))
            }
            Token::While => {
                self.next(); // consume while
                self.expect(Token::LParen)?;
                let condition = self.parse_expr()?;
                self.expect(Token::RParen)?;
                self.expect(Token::LBrace)?;

                let mut body = Vec::new();
                while *self.peek() != Token::RBrace {
                    body.push(self.parse_stmt()?);
                }
                self.expect(Token::RBrace)?;

                Ok(Stmt::While(condition, body))
            }
            _ => {
                let expr = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Expression(expr))
            }
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_comparison()?;

        while matches!(self.peek(), Token::EqualEqual | Token::NotEqual) {
            let op = match self.next() {
                Token::EqualEqual => BinaryOperator::Equal,
                Token::NotEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_term()?;

        while matches!(self.peek(), Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual) {
            let op = match self.next() {
                Token::Greater => BinaryOperator::Greater,
                Token::GreaterEqual => BinaryOperator::GreaterEqual,
                Token::Less => BinaryOperator::Less,
                Token::LessEqual => BinaryOperator::LessEqual,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_factor()?;

        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let op = match self.next() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary()?;

        while matches!(self.peek(), Token::Star | Token::Slash | Token::Percent) {
            let op = match self.next() {
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                Token::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Token::Minus => {
                self.next();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOperator::Minus, Box::new(expr)))
            }
            Token::Bang => {
                self.next();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOperator::Not, Box::new(expr)))
            }
            _ => self.parse_call(),
        }
    }

    fn parse_call(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_primary()?;

        if let Expr::Ident(name) = &expr {
            if *self.peek() == Token::LParen {
                self.next(); // consume (
                let mut args = Vec::new();

                while *self.peek() != Token::RParen {
                    args.push(self.parse_expr()?);
                    if *self.peek() == Token::Comma {
                        self.next(); // consume comma
                    } else if *self.peek() != Token::RParen {
                        return Err(ParseError {
                            message: "Expected ',' or ')' in function call".to_string(),
                            token: self.peek().clone(),
                        });
                    }
                }

                self.expect(Token::RParen)?;
                return Ok(Expr::Call(name.clone(), args));
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.next() {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::Ident(s) => Ok(Expr::Ident(s)),
            Token::String(s) => Ok(Expr::String(s)),
            Token::Bool(b) => Ok(Expr::Bool(b)),
            Token::LParen => {
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            token => Err(ParseError {
                message: "Unexpected token in expression".to_string(),
                token,
            }),
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

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let actual = self.next();
        if std::mem::discriminant(&actual) != std::mem::discriminant(&expected) {
            Err(ParseError {
                message: format!("Expected {:?}", expected),
                token: actual,
            })
        } else {
            Ok(())
        }
    }
}
