use crate::compiler::ast::*;
use crate::compiler::lexer::Token;
use core::clone::Clone;
use core::matches;
use core::option::Option::Some;
use core::result::Result::{self, Err, Ok};
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

impl std::error::Error for ParseError {}

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
        while !self.is_at_end() {
            if matches!(self.peek(), Token::Fn) {
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
            token => {
                return Err(ParseError {
                    message: "Expected function name".to_string(),
                    token,
                });
            }
        };

        self.expect(Token::LParen)?;

        let mut params = Vec::new();
        while !matches!(self.peek(), Token::RParen) {
            // Parameter name
            let param_name = match self.next() {
                Token::Ident(s) => s,
                token => {
                    return Err(ParseError {
                        message: "Expected parameter name".to_string(),
                        token,
                    });
                }
            };
            
            // Expect colon
            self.expect(Token::Colon)?;

            // Parameter type
            let param_type = self.parse_param_type()?;

            params.push(Param {
                name: param_name,
                param_type,
                default_value: None,
            });

            // Comma or closing paren
            if matches!(self.peek(), Token::Comma) {
                self.next(); // consume comma
            } else if !matches!(self.peek(), Token::RParen) {
                return Err(ParseError {
                    message: "Expected ',' or ')' in parameter list".to_string(),
                    token: self.peek().clone(),
                });
            }
        }

        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;

        let mut body = Vec::new();
        while !matches!(self.peek(), Token::RBrace) {
            body.push(self.parse_stmt()?);
        }

        self.expect(Token::RBrace)?;

        Ok(Function {
            name,
            params,
            body,
        })
    }

    fn parse_param_type(&mut self) -> Result<ParamType, ParseError> {
        match self.next() {
            Token::StringType => Ok(ParamType::String),
            Token::BoolType => Ok(ParamType::Bool),
            Token::I8 => Ok(ParamType::Number(Type::I8)),
            Token::I16 => Ok(ParamType::Number(Type::I16)),
            Token::I32 => Ok(ParamType::Number(Type::I32)),
            Token::I64 => Ok(ParamType::Number(Type::I64)),
            Token::I128 => Ok(ParamType::Number(Type::I128)),
            Token::U8 => Ok(ParamType::Number(Type::U8)),
            Token::U16 => Ok(ParamType::Number(Type::U16)),
            Token::U32 => Ok(ParamType::Number(Type::U32)),
            Token::U64 => Ok(ParamType::Number(Type::U64)),
            Token::U128 => Ok(ParamType::Number(Type::U128)),
            Token::F32 => Ok(ParamType::Number(Type::F32)),
            Token::F64 => Ok(ParamType::Number(Type::F64)),
            Token::Any => Ok(ParamType::Any),
            token => Err(ParseError {
                message: "Expected parameter type".to_string(),
                token,
            })
        }
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        match self.next() {
            Token::I8 => Ok(Type::I8),
            Token::I16 => Ok(Type::I16),
            Token::I32 => Ok(Type::I32),
            Token::I64 => Ok(Type::I64),
            Token::I128 => Ok(Type::I128),
            Token::U8 => Ok(Type::U8),
            Token::U16 => Ok(Type::U16),
            Token::U32 => Ok(Type::U32),
            Token::U64 => Ok(Type::U64),
            Token::U128 => Ok(Type::U128),
            Token::F32 => Ok(Type::F32),
            Token::F64 => Ok(Type::F64),
            Token::BoolType => Ok(Type::Bool),
            Token::StringType => Ok(Type::String),
            Token::Any => Ok(Type::Any),
            token => Err(ParseError {
                message: "Expected type".to_string(),
                token,
            })
        }
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Token::Let => {
                self.next(); // consume let
                let name = match self.next() {
                    Token::Ident(s) => s,
                    token => return Err(ParseError {
                        message: "Expected identifier after let".to_string(),
                        token,
                    }),
                };
                
                // Check for optional type annotation
                let var_type = if matches!(self.peek(), Token::Colon) {
                    self.next(); // consume colon
                    self.parse_type()?
                } else {
                    Type::Any
                };
                
                self.expect(Token::Equal)?;
                let expr = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Let(name, var_type, expr))
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
                let expr = if matches!(self.peek(), Token::Semicolon) {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Return(expr))
            }
            Token::If => {
                self.next(); // consume if
                // self.expect(Token::LParen)?;
                let condition = self.parse_expr()?;
                // self.expect(Token::RParen)?;
                self.expect(Token::LBrace)?;

                let mut then_body = Vec::new();
                while !matches!(self.peek(), Token::RBrace) {
                    then_body.push(self.parse_stmt()?);
                }
                self.expect(Token::RBrace)?;

                let else_body = if matches!(self.peek(), Token::Else) {
                    self.next(); // consume else
                    self.expect(Token::LBrace)?;
                    let mut body = Vec::new();
                    while !matches!(self.peek(), Token::RBrace) {
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
                while !matches!(self.peek(), Token::RBrace) {
                    body.push(self.parse_stmt()?);
                }
                self.expect(Token::RBrace)?;

                Ok(Stmt::While(condition, body))
            }
            Token::LBrace => {
                self.next(); // consume {
                let mut stmts = Vec::new();
                while !matches!(self.peek(), Token::RBrace) {
                    stmts.push(self.parse_stmt()?);
                }
                self.expect(Token::RBrace)?;
                Ok(Stmt::Block(stmts))
            }
            _ => {
                let expr = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Expression(expr))
            }
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_cast()
    }

    fn parse_cast(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_equality()?;

        while matches!(self.peek(), Token::As) {
            self.next(); // consume 'as'
            let target_type = self.parse_type()?;
            expr = Expr::Cast(Box::new(expr), target_type);
        }

        Ok(expr)
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

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.peek() {
            
            Token::Underscore => {
                self.next();
                Ok(Pattern::Wildcard)
            }
            Token::Ident(name) => {
                Ok(Pattern::Identifier(name.clone()))
            }
            Token::Literal(_) | Token::Bool(_) | Token::StringLiteral(_) => {
                let expr = self.parse_primary()?;
                match expr {
                    Expr::Literal(val) => Ok(Pattern::Value(val)),
                    _ => Err(ParseError { message: "Expected literal".to_string(), token: self.peek().clone() })
                    
                }
            }
            token => Err(ParseError { message: "Expected pattern (Identifier, _, or literal)".to_string(), token: token.clone()})   
        }
    }

    fn parse_match(&mut self) -> Result<Expr, ParseError> {
        self.expect(Token::Match)?;
        let match_expr = self.parse_expr()?;
        self.expect(Token::LBrace)?;
        let mut arms = Vec::new();
        while !matches!(self.peek(), Token::RBrace) {
            let pattern = self.parse_pattern()?;
            self.expect(Token::Arrow)?;
            let result_expr = self.parse_expr()?;
            self.expect(Token::Comma)?;
            arms.push((pattern, result_expr)); 
        }
        self.expect(Token::RBrace)?;
        Ok(Expr::Match(Box::new(match_expr), arms))
        
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
            if matches!(self.peek(), Token::LParen) {
                self.next(); // consume (
                let mut args = Vec::new();

                while !matches!(self.peek(), Token::RParen) {
                    args.push(self.parse_expr()?);
                    if matches!(self.peek(), Token::Comma) {
                        self.next(); // consume comma
                    } else if !matches!(self.peek(), Token::RParen) {
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
        match self.peek() {
            Token::Match => self.parse_match(),
            _ => match self.next() {
                    
                    Token::Literal(value) => {
                        Ok(Expr::Literal(value))
                    }
                    Token::StringLiteral(s) => {
                        Ok(Expr::Literal(Value::String(s)))
                    }
                    Token::Bool(b) => {
                        Ok(Expr::Literal(Value::Bool(b)))
                    }
                    Token::Ident(s) => Ok(Expr::Ident(s)),
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
}

    fn peek(&mut self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::EOF)
    }

    fn next(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF);
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn is_at_end(&mut self) -> bool {
        matches!(self.peek(), Token::EOF) || self.pos >= self.tokens.len()
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let actual = self.next();
        // if self.peek() == &expected {
        //     drop(actual);
        //     Ok(())
        // } else {
        //     Err(ParseError {
        //         message: format!("Expected {:?}, got {:?}", expected, actual),
        //         token: self.peek().clone(),
        //     })
        // }
        
        if std::mem::discriminant(&actual) != std::mem::discriminant(&expected) {
            Err(ParseError {
                message: format!("Expected {:?}, got {:?}", expected, actual),
                token: actual,
            })
        } else {
            Ok(())
        }
    }
}