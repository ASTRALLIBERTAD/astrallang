// ast.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Void,
    Array(Box<Type>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Ident(String),
    BinaryOp(Box<Expr>, BinaryOperator, Box<Expr>),
    UnaryOp(UnaryOperator, Box<Expr>),
    Call(String, Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Assignment(String, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not, Neg,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Type, Option<Expr>),
    Assignment(String, Expr),
    Print(Expr),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    For(String, Expr, Expr, Vec<Stmt>),
    Function(Function),
    Return(Option<Expr>),
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: Vec<Stmt>,
    pub is_extern: bool,
}

#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
    pub globals: Vec<(String, Type, Option<Expr>)>,
}

// lexer.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Keywords
    Fn, Let, If, Else, While, For, Return, True, False, Extern,
    
    // Types
    Int, Float, Bool, String, Void,
    
    // Identifiers and literals
    Ident(String),
    Number(i64),
    FloatLit(String),
    StringLit(String),
    
    // Operators
    Plus, Minus, Star, Slash, Percent,
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    And, Or, Not,
    Assign,
    
    // Delimiters
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Semicolon, Comma, Arrow,
    
    // Special
    EOF, Newline,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        
        while self.position < self.input.len() {
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                break;
            }
            
            let ch = self.current_char();
            
            match ch {
                '+' => { tokens.push(Token::Plus); self.advance(); }
                '-' => {
                    self.advance();
                    if self.current_char() == '>' {
                        self.advance();
                        tokens.push(Token::Arrow);
                    } else {
                        tokens.push(Token::Minus);
                    }
                }
                '*' => { tokens.push(Token::Star); self.advance(); }
                '/' => {
                    self.advance();
                    if self.current_char() == '/' {
                        self.skip_line_comment();
                        continue;
                    }
                    tokens.push(Token::Slash);
                }
                '%' => { tokens.push(Token::Percent); self.advance(); }
                '=' => {
                    self.advance();
                    if self.current_char() == '=' {
                        self.advance();
                        tokens.push(Token::Equal);
                    } else {
                        tokens.push(Token::Assign);
                    }
                }
                '!' => {
                    self.advance();
                    if self.current_char() == '=' {
                        self.advance();
                        tokens.push(Token::NotEqual);
                    } else {
                        tokens.push(Token::Not);
                    }
                }
                '<' => {
                    self.advance();
                    if self.current_char() == '=' {
                        self.advance();
                        tokens.push(Token::LessEqual);
                    } else {
                        tokens.push(Token::Less);
                    }
                }
                '>' => {
                    self.advance();
                    if self.current_char() == '=' {
                        self.advance();
                        tokens.push(Token::GreaterEqual);
                    } else {
                        tokens.push(Token::Greater);
                    }
                }
                '&' => {
                    self.advance();
                    if self.current_char() == '&' {
                        self.advance();
                        tokens.push(Token::And);
                    }
                }
                '|' => {
                    self.advance();
                    if self.current_char() == '|' {
                        self.advance();
                        tokens.push(Token::Or);
                    }
                }
                '(' => { tokens.push(Token::LParen); self.advance(); }
                ')' => { tokens.push(Token::RParen); self.advance(); }
                '{' => { tokens.push(Token::LBrace); self.advance(); }
                '}' => { tokens.push(Token::RBrace); self.advance(); }
                '[' => { tokens.push(Token::LBracket); self.advance(); }
                ']' => { tokens.push(Token::RBracket); self.advance(); }
                ';' => { tokens.push(Token::Semicolon); self.advance(); }
                ',' => { tokens.push(Token::Comma); self.advance(); }
                '"' => tokens.push(self.read_string()),
                '\n' => { tokens.push(Token::Newline); self.advance(); }
                _ if ch.is_ascii_digit() => tokens.push(self.read_number()),
                _ if ch.is_ascii_alphabetic() || ch == '_' => tokens.push(self.read_identifier()),
                _ => self.advance(), // Skip unknown characters
            }
        }
        
        tokens.push(Token::EOF);
        tokens
    }

    fn current_char(&self) -> char {
        self.input.get(self.position).copied().unwrap_or('\0')
    }

    fn advance(&mut self) {
        if self.position < self.input.len() && self.input[self.position] == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self.position += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.current_char().is_ascii_whitespace() && self.current_char() != '\n' {
            self.advance();
        }
    }

    fn skip_line_comment(&mut self) {
        while self.current_char() != '\n' && self.position < self.input.len() {
            self.advance();
        }
    }

    fn read_string(&mut self) -> Token {
        self.advance(); // Skip opening quote
        let mut value = String::new();
        
        while self.current_char() != '"' && self.position < self.input.len() {
            if self.current_char() == '\\' {
                self.advance();
                match self.current_char() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    _ => value.push(self.current_char()),
                }
            } else {
                value.push(self.current_char());
            }
            self.advance();
        }
        
        self.advance(); // Skip closing quote
        Token::StringLit(value)
    }

    fn read_number(&mut self) -> Token {
        let mut number = String::new();
        let mut is_float = false;
        
        while self.current_char().is_ascii_digit() || self.current_char() == '.' {
            if self.current_char() == '.' {
                if is_float {
                    break; // Multiple dots not allowed
                }
                is_float = true;
            }
            number.push(self.current_char());
            self.advance();
        }
        
        if is_float {
            Token::FloatLit(number)
        } else {
            Token::Number(number.parse().unwrap_or(0))
        }
    }

    fn read_identifier(&mut self) -> Token {
        let mut identifier = String::new();
        
        while self.current_char().is_ascii_alphanumeric() || self.current_char() == '_' {
            identifier.push(self.current_char());
            self.advance();
        }
        
        match identifier.as_str() {
            "fn" => Token::Fn,
            "let" => Token::Let,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "return" => Token::Return,
            "true" => Token::True,
            "false" => Token::False,
            "extern" => Token::Extern,
            "int" => Token::Int,
            "float" => Token::Float,
            "bool" => Token::Bool,
            "string" => Token::String,
            "void" => Token::Void,
            _ => Token::Ident(identifier),
        }
    }
}

// parser.rs
use crate::ast::*;
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();
        let mut globals = Vec::new();

        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }

            match self.current() {
                Token::Fn | Token::Extern => {
                    functions.push(self.parse_function()?);
                }
                Token::Let => {
                    let (name, typ, expr) = self.parse_global_let()?;
                    globals.push((name, typ, expr));
                }
                _ => return Err("Expected function or global variable declaration".to_string()),
            }
        }

        Ok(Program { functions, globals })
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        let is_extern = self.match_token(&Token::Extern);
        self.expect(Token::Fn)?;
        
        let name = self.expect_identifier()?;
        self.expect(Token::LParen)?;
        
        let mut params = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                let param_name = self.expect_identifier()?;
                self.expect(Token::Arrow)?;
                let param_type = self.parse_type()?;
                params.push((param_name, param_type));
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.expect(Token::RParen)?;
        self.expect(Token::Arrow)?;
        let return_type = self.parse_type()?;
        
        let body = if is_extern {
            self.expect(Token::Semicolon)?;
            Vec::new()
        } else {
            self.expect(Token::LBrace)?;
            let mut statements = Vec::new();
            while !self.check(&Token::RBrace) && !self.is_at_end() {
                statements.push(self.parse_statement()?);
            }
            self.expect(Token::RBrace)?;
            statements
        };

        Ok(Function {
            name,
            params,
            return_type,
            body,
            is_extern,
        })
    }

    fn parse_global_let(&mut self) -> Result<(String, Type, Option<Expr>), String> {
        self.expect(Token::Let)?;
        let name = self.expect_identifier()?;
        self.expect(Token::Arrow)?;
        let typ = self.parse_type()?;
        
        let expr = if self.match_token(&Token::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.expect(Token::Semicolon)?;
        Ok((name, typ, expr))
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        self.skip_newlines();
        
        match self.current() {
            Token::Let => self.parse_let_statement(),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::For => self.parse_for_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Ident(_) => {
                if self.check_ahead(1, &Token::Assign) {
                    self.parse_assignment_statement()
                } else {
                    Ok(Stmt::Expression(self.parse_expression()?))
                }
            }
            _ => Ok(Stmt::Expression(self.parse_expression()?)),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Let)?;
        let name = self.expect_identifier()?;
        self.expect(Token::Arrow)?;
        let typ = self.parse_type()?;
        
        let expr = if self.match_token(&Token::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.expect(Token::Semicolon)?;
        Ok(Stmt::Let(name, typ, expr))
    }

    fn parse_assignment_statement(&mut self) -> Result<Stmt, String> {
        let name = self.expect_identifier()?;
        self.expect(Token::Assign)?;
        let expr = self.parse_expression()?;
        self.expect(Token::Semicolon)?;
        Ok(Stmt::Assignment(name, expr))
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, String> {
        self.expect(Token::If)?;
        let condition = self.parse_expression()?;
        self.expect(Token::LBrace)?;
        
        let mut then_branch = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            then_branch.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;
        
        let else_branch = if self.match_token(&Token::Else) {
            self.expect(Token::LBrace)?;
            let mut statements = Vec::new();
            while !self.check(&Token::RBrace) && !self.is_at_end() {
                statements.push(self.parse_statement()?);
            }
            self.expect(Token::RBrace)?;
            Some(statements)
        } else {
            None
        };

        Ok(Stmt::If(condition, then_branch, else_branch))
    }

    fn parse_while_statement(&mut self) -> Result<Stmt, String> {
        self.expect(Token::While)?;
        let condition = self.parse_expression()?;
        self.expect(Token::LBrace)?;
        
        let mut body = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;
        
        Ok(Stmt::While(condition, body))
    }

    fn parse_for_statement(&mut self) -> Result<Stmt, String> {
        self.expect(Token::For)?;
        let var = self.expect_identifier()?;
        self.expect(Token::Assign)?;
        let start = self.parse_expression()?;
        self.expect(Token::Semicolon)?;
        let end = self.parse_expression()?;
        self.expect(Token::LBrace)?;
        
        let mut body = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;
        
        Ok(Stmt::For(var, start, end, body))
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Return)?;
        let expr = if self.check(&Token::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(Token::Semicolon)?;
        Ok(Stmt::Return(expr))
    }

    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_and()?;
        
        while self.match_token(&Token::Or) {
            let right = self.parse_and()?;
            expr = Expr::BinaryOp(Box::new(expr), BinaryOperator::Or, Box::new(right));
        }
        
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_equality()?;
        
        while self.match_token(&Token::And) {
            let right = self.parse_equality()?;
            expr = Expr::BinaryOp(Box::new(expr), BinaryOperator::And, Box::new(right));
        }
        
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_comparison()?;
        
        while let Some(op) = self.match_equality_op() {
            let right = self.parse_comparison()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_term()?;
        
        while let Some(op) = self.match_comparison_op() {
            let right = self.parse_term()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_factor()?;
        
        while let Some(op) = self.match_term_op() {
            let right = self.parse_factor()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_unary()?;
        
        while let Some(op) = self.match_factor_op() {
            let right = self.parse_unary()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if let Some(op) = self.match_unary_op() {
            let expr = self.parse_unary()?;
            Ok(Expr::UnaryOp(op, Box::new(expr)))
        } else {
            self.parse_call()
        }
    }

    fn parse_call(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        
        while self.match_token(&Token::LParen) {
            let mut args = Vec::new();
            if !self.check(&Token::RParen) {
                loop {
                    args.push(self.parse_expression()?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            self.expect(Token::RParen)?;
            
            if let Expr::Ident(name) = expr {
                expr = Expr::Call(name, args);
            } else {
                return Err("Can only call functions".to_string());
            }
        }
        
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current() {
            Token::Number(n) => {
                let value = *n;
                self.advance();
                Ok(Expr::Number(value))
            }
            Token::FloatLit(s) => {
                let value: f64 = s.parse().map_err(|_| "Invalid float literal")?;
                self.advance();
                Ok(Expr::Float(value))
            }
            Token::StringLit(s) => {
                let value = s.clone();
                self.advance();
                Ok(Expr::String(value))
            }
            Token::True => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Ident(name))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            _ => Err(format!("Unexpected token: {:?}", self.current())),
        }
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.current() {
            Token::Int => { self.advance(); Ok(Type::Int) }
            Token::Float => { self.advance(); Ok(Type::Float) }
            Token::Bool => { self.advance(); Ok(Type::Bool) }
            Token::String => { self.advance(); Ok(Type::String) }
            Token::Void => { self.advance(); Ok(Type::Void) }
            _ => Err("Expected type".to_string()),
        }
    }

    // Helper methods for operator matching
    fn match_equality_op(&mut self) -> Option<BinaryOperator> {
        match self.current() {
            Token::Equal => { self.advance(); Some(BinaryOperator::Eq) }
            Token::NotEqual => { self.advance(); Some(BinaryOperator::Ne) }
            _ => None,
        }
    }

    fn match_comparison_op(&mut self) -> Option<BinaryOperator> {
        match self.current() {
            Token::Less => { self.advance(); Some(BinaryOperator::Lt) }
            Token::LessEqual => { self.advance(); Some(BinaryOperator::Le) }
            Token::Greater => { self.advance(); Some(BinaryOperator::Gt) }
            Token::GreaterEqual => { self.advance(); Some(BinaryOperator::Ge) }
            _ => None,
        }
    }

    fn match_term_op(&mut self) -> Option<BinaryOperator> {
        match self.current() {
            Token::Plus => { self.advance(); Some(BinaryOperator::Add) }
            Token::Minus => { self.advance(); Some(BinaryOperator::Sub) }
            _ => None,
        }
    }

    fn match_factor_op(&mut self) -> Option<BinaryOperator> {
        match self.current() {
            Token::Star => { self.advance(); Some(BinaryOperator::Mul) }
            Token::Slash => { self.advance(); Some(BinaryOperator::Div) }
            Token::Percent => { self.advance(); Some(BinaryOperator::Mod) }
            _ => None,
        }
    }

    fn match_unary_op(&mut self) -> Option<UnaryOperator> {
        match self.current() {
            Token::Not => { self.advance(); Some(UnaryOperator::Not) }
            Token::Minus => { self.advance(); Some(UnaryOperator::Neg) }
            _ => None,
        }
    }

    // Utility methods
    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::EOF)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.position += 1;
        }
        self.previous()
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.position - 1]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current(), Token::EOF)
    }

    fn check(&self, token: &Token) -> bool {
        std::mem::discriminant(self.current()) == std::mem::discriminant(token)
    }

    fn check_ahead(&self, offset: usize, token: &Token) -> bool {
        if let Some(t) = self.tokens.get(self.position + offset) {
            std::mem::discriminant(t) == std::mem::discriminant(token)
        } else {
            false
        }
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token: Token) -> Result<(), String> {
        if self.check(&token) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, found {:?}", token, self.current()))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, String> {
        if let Token::Ident(name) = self.current() {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(format!("Expected identifier, found {:?}", self.current()))
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.current(), Token::Newline) {
            self.advance();
        }
    }
}

// codegen.rs - Enhanced x86-64 code generation
use crate::ast::*;
use std::collections::HashMap;

pub struct CodeGenerator {
    output: String,
    label_counter: usize,
    locals: HashMap<String, i32>,
    stack_offset: i32,
    function_locals: HashMap<String, HashMap<String, i32>>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            label_counter: 0,
            locals: HashMap::new(),
            stack_offset: 0,
            function_locals: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.emit_header();
        
        // Generate global variables
        if !program.globals.is_empty() {
            self.emit_line("section .data");
            for (name, _typ, expr) in &program.globals {
                if let Some(expr) = expr {
                    match expr {
                        Expr::Number(n) => {
                            self.emit_line(&format!("{}: dq {}", name, n));
                        }
                        Expr::String(s) => {
                            self.emit_line(&format!("{}: db '{}', 0", name, s));
                        }
                        _ => {
                            self.emit_line(&format!("{}: dq 0", name));
                        }
                    }
                } else {
                    self.emit_line(&format!("{}: dq 0", name));
                }
            }
            self.emit_line("");
        }

        self.emit_line("section .text");
        self.emit_line("global _start");

        // Generate functions
        for function in &program.functions {
            if !function.is_extern {
                self.generate_function(function);
            }
        }

        // Generate main entry point
        if program.functions.iter().any(|f| f.name == "main") {
            self.emit_line("_start:");
            self.emit_line("    call main");
            self.emit_line("    mov rax, 60    ; sys_exit");
            self.emit_line("    mov rdi, 0     ; exit status");
            self.emit_line("    syscall");
        }

        // Built-in functions
        self.generate_builtin_print();
        
        self.output.clone()
    }

    fn generate_function(&mut self, function: &Function) -> {
        self.locals.clear();
        self.stack_offset = 0;

        self.emit_line(&format!("{}:", function.name));
        self.emit_line("    push rbp");
        self.emit_line("    mov rbp, rsp");

        // Reserve space for parameters
        for (i, (name, _typ)) in function.params.iter().enumerate() {
            self.stack_offset -= 8;
            self.locals.insert(name.clone(), self.stack_offset);
            // Move parameters from registers to stack
            match i {
                0 => self.emit_line(&format!("    mov [rbp{}], rdi", self.stack_offset)),
                1 => self.emit_line(&format!("    mov [rbp{}], rsi", self.stack_offset)),
                2 => self.emit_line(&format!("    mov [rbp{}], rdx", self.stack_offset)),
                3 => self.emit_line(&format!("    mov [rbp{}], rcx", self.stack_offset)),
                4 => self.emit_line(&format!("    mov [rbp{}], r8", self.stack_offset)),
                5 => self.emit_line(&format!("    mov [rbp{}], r9", self.stack_offset)),
                _ => {} // More parameters would be on stack
            }
        }

        // Generate function body
        for stmt in &function.body {
            self.generate_statement(stmt);
        }

        // Default return if no explicit return
        if function.return_type != Type::Void {
            self.emit_line("    xor rax, rax   ; default return 0");
        }
        
        self.emit_line("    mov rsp, rbp");
        self.emit_line("    pop rbp");
        self.emit_line("    ret");
        self.emit_line("");

        // Store function locals for later reference
        self.function_locals.insert(function.name.clone(), self.locals.clone());
    }

    fn generate_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(name, _typ, expr) => {
                self.stack_offset -= 8;
                self.locals.insert(name.clone(), self.stack_offset);
                
                if let Some(expr) = expr {
                    self.generate_expression(expr);
                    self.emit_line(&format!("    mov [rbp{}], rax", self.stack_offset));
                } else {
                    self.emit_line(&format!("    mov qword [rbp{}], 0", self.stack_offset));
                }
            }
            
            Stmt::Assignment(name, expr) => {
                self.generate_expression(expr);
                if let Some(&offset) = self.locals.get(name) {
                    self.emit_line(&format!("    mov [rbp{}], rax", offset));
                } else {
                    // Global variable
                    self.emit_line(&format!("    mov [{}], rax", name));
                }
            }
            
            Stmt::Print(expr) => {
                self.generate_expression(expr);
                self.emit_line("    mov rdi, rax");
                self.emit_line("    call print_int");
            }
            
            Stmt::If(condition, then_branch, else_branch) => {
                let else_label = self.new_label("else");
                let end_label = self.new_label("endif");
                
                self.generate_expression(condition);
                self.emit_line("    test rax, rax");
                self.emit_line(&format!("    jz {}", else_label));
                
                for stmt in then_branch {
                    self.generate_statement(stmt);
                }
                self.emit_line(&format!("    jmp {}", end_label));
                
                self.emit_line(&format!("{}:", else_label));
                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.generate_statement(stmt);
                    }
                }
                
                self.emit_line(&format!("{}:", end_label));
            }
            
            Stmt::While(condition, body) => {
                let loop_label = self.new_label("loop");
                let end_label = self.new_label("endloop");
                
                self.emit_line(&format!("{}:", loop_label));
                self.generate_expression(condition);
                self.emit_line("    test rax, rax");
                self.emit_line(&format!("    jz {}", end_label));
                
                for stmt in body {
                    self.generate_statement(stmt);
                }
                
                self.emit_line(&format!("    jmp {}", loop_label));
                self.emit_line(&format!("{}:", end_label));
            }
            
            Stmt::For(var, start, end, body) => {
                // Allocate loop variable
                self.stack_offset -= 8;
                self.locals.insert(var.clone(), self.stack_offset);
                
                let loop_label = self.new_label("forloop");
                let end_label = self.new_label("endfor");
                
                // Initialize loop variable
                self.generate_expression(start);
                self.emit_line(&format!("    mov [rbp{}], rax", self.stack_offset));
                
                self.emit_line(&format!("{}:", loop_label));
                
                // Check condition
                self.generate_expression(end);
                self.emit_line("    mov rbx, rax");
                self.emit_line(&format!("    mov rax, [rbp{}]", self.stack_offset));
                self.emit_line("    cmp rax, rbx");
                self.emit_line(&format!("    jge {}", end_label));
                
                // Execute body
                for stmt in body {
                    self.generate_statement(stmt);
                }
                
                // Increment loop variable
                self.emit_line(&format!("    inc qword [rbp{}]", self.stack_offset));
                self.emit_line(&format!("    jmp {}", loop_label));
                self.emit_line(&format!("{}:", end_label));
            }
            
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.generate_expression(expr);
                } else {
                    self.emit_line("    xor rax, rax");
                }
                self.emit_line("    mov rsp, rbp");
                self.emit_line("    pop rbp");
                self.emit_line("    ret");
            }
            
            Stmt::Expression(expr) => {
                self.generate_expression(expr);
            }
            
            _ => {}
        }
    }

    fn generate_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(n) => {
                self.emit_line(&format!("    mov rax, {}", n));
            }
            
            Expr::Float(f) => {
                // For now, convert to integer (proper float support would need SSE)
                self.emit_line(&format!("    mov rax, {}", *f as i64));
            }
            
            Expr::Bool(b) => {
                self.emit_line(&format!("    mov rax, {}", if *b { 1 } else { 0 }));
            }
            
            Expr::String(_s) => {
                // String literals would need more complex handling
                self.emit_line("    mov rax, 0  ; string literal placeholder");
            }
            
            Expr::Ident(name) => {
                if let Some(&offset) = self.locals.get(name) {
                    self.emit_line(&format!("    mov rax, [rbp{}]", offset));
                } else {
                    // Global variable
                    self.emit_line(&format!("    mov rax, [{}]", name));
                }
            }
            
            Expr::BinaryOp(left, op, right) => {
                self.generate_expression(right);
                self.emit_line("    push rax");
                self.generate_expression(left);
                self.emit_line("    pop rbx");
                
                match op {
                    BinaryOperator::Add => self.emit_line("    add rax, rbx"),
                    BinaryOperator::Sub => self.emit_line("    sub rax, rbx"),
                    BinaryOperator::Mul => self.emit_line("    imul rax, rbx"),
                    BinaryOperator::Div => {
                        self.emit_line("    cqo");  // sign extend rax to rdx:rax
                        self.emit_line("    idiv rbx");
                    }
                    BinaryOperator::Mod => {
                        self.emit_line("    cqo");
                        self.emit_line("    idiv rbx");
                        self.emit_line("    mov rax, rdx");
                    }
                    BinaryOperator::Eq => {
                        self.emit_line("    cmp rax, rbx");
                        self.emit_line("    sete al");
                        self.emit_line("    movzx rax, al");
                    }
                    BinaryOperator::Ne => {
                        self.emit_line("    cmp rax, rbx");
                        self.emit_line("    setne al");
                        self.emit_line("    movzx rax, al");
                    }
                    BinaryOperator::Lt => {
                        self.emit_line("    cmp rax, rbx");
                        self.emit_line("    setl al");
                        self.emit_line("    movzx rax, al");
                    }
                    BinaryOperator::Le => {
                        self.emit_line("    cmp rax, rbx");
                        self.emit_line("    setle al");
                        self.emit_line("    movzx rax, al");
                    }
                    BinaryOperator::Gt => {
                        self.emit_line("    cmp rax, rbx");
                        self.emit_line("    setg al");
                        self.emit_line("    movzx rax, al");
                    }
                    BinaryOperator::Ge => {
                        self.emit_line("    cmp rax, rbx");
                        self.emit_line("    setge al");
                        self.emit_line("    movzx rax, al");
                    }
                    BinaryOperator::And => {
                        self.emit_line("    test rax, rax");
                        self.emit_line("    setne al");
                        self.emit_line("    test rbx, rbx");
                        self.emit_line("    setne bl");
                        self.emit_line("    and al, bl");
                        self.emit_line("    movzx rax, al");
                    }
                    BinaryOperator::Or => {
                        self.emit_line("    test rax, rax");
                        self.emit_line("    setne al");
                        self.emit_line("    test rbx, rbx");
                        self.emit_line("    setne bl");
                        self.emit_line("    or al, bl");
                        self.emit_line("    movzx rax, al");
                    }
                }
            }
            
            Expr::UnaryOp(op, operand) => {
                self.generate_expression(operand);
                match op {
                    UnaryOperator::Neg => self.emit_line("    neg rax"),
                    UnaryOperator::Not => {
                        self.emit_line("    test rax, rax");
                        self.emit_line("    setz al");
                        self.emit_line("    movzx rax, al");
                    }
                }
            }
            
            Expr::Call(name, args) => {
                // Save caller-saved registers
                self.emit_line("    push rcx");
                self.emit_line("    push rdx");
                self.emit_line("    push rsi");
                self.emit_line("    push rdi");
                self.emit_line("    push r8");
                self.emit_line("    push r9");
                
                // Pass arguments in registers (System V ABI)
                for (i, arg) in args.iter().enumerate() {
                    self.generate_expression(arg);
                    match i {
                        0 => self.emit_line("    mov rdi, rax"),
                        1 => self.emit_line("    mov rsi, rax"),
                        2 => self.emit_line("    mov rdx, rax"),
                        3 => self.emit_line("    mov rcx, rax"),
                        4 => self.emit_line("    mov r8, rax"),
                        5 => self.emit_line("    mov r9, rax"),
                        _ => self.emit_line("    push rax"), // Additional args on stack
                    }
                }
                
                self.emit_line(&format!("    call {}", name));
                
                // Clean up stack for extra arguments
                if args.len() > 6 {
                    let stack_cleanup = (args.len() - 6) * 8;
                    self.emit_line(&format!("    add rsp, {}", stack_cleanup));
                }
                
                // Restore caller-saved registers
                self.emit_line("    pop r9");
                self.emit_line("    pop r8");
                self.emit_line("    pop rdi");
                self.emit_line("    pop rsi");
                self.emit_line("    pop rdx");
                self.emit_line("    pop rcx");
            }
            
            _ => {
                self.emit_line("    mov rax, 0  ; unimplemented expression");
            }
        }
    }

    fn generate_builtin_print(&mut self) {
        self.emit_line("print_int:");
        self.emit_line("    push rbp");
        self.emit_line("    mov rbp, rsp");
        self.emit_line("    push rbx");
        self.emit_line("    push r12");
        self.emit_line("    push r13");
        self.emit_line("    push r14");
        self.emit_line("    push r15");
        self.emit_line("");
        self.emit_line("    mov rax, rdi        ; number to print");
        self.emit_line("    mov rbx, 10         ; divisor");
        self.emit_line("    xor rcx, rcx        ; digit counter");
        self.emit_line("    sub rsp, 32         ; buffer space");
        self.emit_line("    mov rsi, rsp");
        self.emit_line("    add rsi, 31         ; point to end of buffer");
        self.emit_line("    mov byte [rsi], 0   ; null terminator");
        self.emit_line("    dec rsi");
        self.emit_line("");
        self.emit_line("    test rax, rax");
        self.emit_line("    jns .positive");
        self.emit_line("    neg rax");
        self.emit_line("    mov r15, 1          ; negative flag");
        self.emit_line("    jmp .convert");
        self.emit_line("");
        self.emit_line(".positive:");
        self.emit_line("    xor r15, r15        ; positive flag");
        self.emit_line("");
        self.emit_line(".convert:");
        self.emit_line("    xor rdx, rdx");
        self.emit_line("    div rbx");
        self.emit_line("    add rdx, '0'");
        self.emit_line("    mov [rsi], dl");
        self.emit_line("    dec rsi");
        self.emit_line("    inc rcx");
        self.emit_line("    test rax, rax");
        self.emit_line("    jnz .convert");
        self.emit_line("");
        self.emit_line("    test r15, r15");
        self.emit_line("    jz .print");
        self.emit_line("    mov byte [rsi], '-'");
        self.emit_line("    dec rsi");
        self.emit_line("    inc rcx");
        self.emit_line("");
        self.emit_line(".print:");
        self.emit_line("    inc rsi             ; point to first character");
        self.emit_line("    mov rax, 1          ; sys_write");
        self.emit_line("    mov rdi, 1          ; stdout");
        self.emit_line("    mov rdx, rcx        ; length");
        self.emit_line("    syscall");
        self.emit_line("");
        self.emit_line("    ; print newline");
        self.emit_line("    mov rax, 1");
        self.emit_line("    mov rdi, 1");
        self.emit_line("    mov rsi, newline");
        self.emit_line("    mov rdx, 1");
        self.emit_line("    syscall");
        self.emit_line("");
        self.emit_line("    add rsp, 32");
        self.emit_line("    pop r15");
        self.emit_line("    pop r14");
        self.emit_line("    pop r13");
        self.emit_line("    pop r12");
        self.emit_line("    pop rbx");
        self.emit_line("    pop rbp");
        self.emit_line("    ret");
        self.emit_line("");
        self.emit_line("section .rodata");
        self.emit_line("newline: db 10");
    }

    fn emit_header(&mut self) {
        self.emit_line("; Generated by Astral Compiler");
        self.emit_line("; x86-64 assembly for Linux");
        self.emit_line("");
    }

    fn emit_line(&mut self, line: &str) {
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!(".{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }
}

// semantic.rs - Type checker and semantic analyzer
use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SemanticError {
    pub message: String,
}

pub struct SemanticAnalyzer {
    functions: HashMap<String, Function>,
    globals: HashMap<String, Type>,
    locals: HashMap<String, Type>,
    current_function_return_type: Option<Type>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        
        // Built-in functions
        functions.insert("print".to_string(), Function {
            name: "print".to_string(),
            params: vec![("value".to_string(), Type::Int)],
            return_type: Type::Void,
            body: Vec::new(),
            is_extern: true,
        });

        Self {
            functions,
            globals: HashMap::new(),
            locals: HashMap::new(),
            current_function_return_type: None,
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), SemanticError> {
        // First pass: collect all function signatures and globals
        for func in &program.functions {
            self.functions.insert(func.name.clone(), func.clone());
        }

        for (name, typ, _) in &program.globals {
            self.globals.insert(name.clone(), typ.clone());
        }

        // Second pass: type check function bodies
        for func in &program.functions {
            if !func.is_extern {
                self.check_function(func)?;
            }
        }

        // Check for main function
        if !self.functions.contains_key("main") {
            return Err(SemanticError {
                message: "No main function found".to_string(),
            });
        }

        Ok(())
    }

    fn check_function(&mut self, func: &Function) -> Result<(), SemanticError> {
        self.locals.clear();
        self.current_function_return_type = Some(func.return_type.clone());

        // Add parameters to locals
        for (name, typ) in &func.params {
            self.locals.insert(name.clone(), typ.clone());
        }

        for stmt in &func.body {
            self.check_statement(stmt)?;
        }

        Ok(())
    }

    fn check_statement(&mut self, stmt: &Stmt) -> Result<(), SemanticError> {
        match stmt {
            Stmt::Let(name, declared_type, expr) => {
                if let Some(expr) = expr {
                    let expr_type = self.check_expression(expr)?;
                    if !self.types_compatible(&expr_type, declared_type) {
                        return Err(SemanticError {
                            message: format!(
                                "Type mismatch in variable '{}': expected {:?}, got {:?}",
                                name, declared_type, expr_type
                            ),
                        });
                    }
                }
                self.locals.insert(name.clone(), declared_type.clone());
            }

            Stmt::Assignment(name, expr) => {
                let expr_type = self.check_expression(expr)?;
                
                let var_type = self.locals.get(name)
                    .or_else(|| self.globals.get(name))
                    .ok_or_else(|| SemanticError {
                        message: format!("Undefined variable '{}'", name),
                    })?;

                if !self.types_compatible(&expr_type, var_type) {
                    return Err(SemanticError {
                        message: format!(
                            "Type mismatch in assignment to '{}': expected {:?}, got {:?}",
                            name, var_type, expr_type
                        ),
                    });
                }
            }

            Stmt::Print(expr) => {
                let expr_type = self.check_expression(expr)?;
                // Print accepts any type for now
                if !matches!(expr_type, Type::Int | Type::Float | Type::Bool) {
                    return Err(SemanticError {
                        message: "Print statement requires numeric or boolean type".to_string(),
                    });
                }
            }

            Stmt::If(condition, then_branch, else_branch) => {
                let cond_type = self.check_expression(condition)?;
                if !matches!(cond_type, Type::Bool) {
                    return Err(SemanticError {
                        message: "If condition must be boolean".to_string(),
                    });
                }

                for stmt in then_branch {
                    self.check_statement(stmt)?;
                }

                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.check_statement(stmt)?;
                    }
                }
            }

            Stmt::While(condition, body) => {
                let cond_type = self.check_expression(condition)?;
                if !matches!(cond_type, Type::Bool) {
                    return Err(SemanticError {
                        message: "While condition must be boolean".to_string(),
                    });
                }

                for stmt in body {
                    self.check_statement(stmt)?;
                }
            }

            Stmt::For(var, start, end, body) => {
                let start_type = self.check_expression(start)?;
                let end_type = self.check_expression(end)?;

                if !matches!(start_type, Type::Int) || !matches!(end_type, Type::Int) {
                    return Err(SemanticError {
                        message: "For loop bounds must be integers".to_string(),
                    });
                }

                // Add loop variable to scope temporarily
                let old_var = self.locals.get(var).cloned();
                self.locals.insert(var.clone(), Type::Int);

                for stmt in body {
                    self.check_statement(stmt)?;
                }

                // Restore old variable if it existed
                if let Some(old_type) = old_var {
                    self.locals.insert(var.clone(), old_type);
                } else {
                    self.locals.remove(var);
                }
            }

            Stmt::Return(expr) => {
                let return_type = if let Some(expr) = expr {
                    self.check_expression(expr)?
                } else {
                    Type::Void
                };

                if let Some(expected_type) = &self.current_function_return_type {
                    if !self.types_compatible(&return_type, expected_type) {
                        return Err(SemanticError {
                            message: format!(
                                "Return type mismatch: expected {:?}, got {:?}",
                                expected_type, return_type
                            ),
                        });
                    }
                }
            }

            Stmt::Expression(expr) => {
                self.check_expression(expr)?;
            }

            _ => {}
        }

        Ok(())
    }

    fn check_expression(&self, expr: &Expr) -> Result<Type, SemanticError> {
        match expr {
            Expr::Number(_) => Ok(Type::Int),
            Expr::Float(_) => Ok(Type::Float),
            Expr::Bool(_) => Ok(Type::Bool),
            Expr::String(_) => Ok(Type::String),

            Expr::Ident(name) => {
                self.locals.get(name)
                    .or_else(|| self.globals.get(name))
                    .cloned()
                    .ok_or_else(|| SemanticError {
                        message: format!("Undefined variable '{}'", name),
                    })
            }

            Expr::BinaryOp(left, op, right) => {
                let left_type = self.check_expression(left)?;
                let right_type = self.check_expression(right)?;

                match op {
                    BinaryOperator::Add | BinaryOperator::Sub | 
                    BinaryOperator::Mul | BinaryOperator::Div | 
                    BinaryOperator::Mod => {
                        if matches!(left_type, Type::Int) && matches!(right_type, Type::Int) {
                            Ok(Type::Int)
                        } else if matches!(left_type, Type::Float) || matches!(right_type, Type::Float) {
                            Ok(Type::Float)
                        } else {
                            Err(SemanticError {
                                message: "Arithmetic operations require numeric types".to_string(),
                            })
                        }
                    }

                    BinaryOperator::Eq | BinaryOperator::Ne |
                    BinaryOperator::Lt | BinaryOperator::Le |
                    BinaryOperator::Gt | BinaryOperator::Ge => {
                        if self.types_compatible(&left_type, &right_type) {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError {
                                message: "Comparison requires compatible types".to_string(),
                            })
                        }
                    }

                    BinaryOperator::And | BinaryOperator::Or => {
                        if matches!(left_type, Type::Bool) && matches!(right_type, Type::Bool) {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError {
                                message: "Logical operations require boolean types".to_string(),
                            })
                        }
                    }
                }
            }

            Expr::UnaryOp(op, operand) => {
                let operand_type = self.check_expression(operand)?;
                
                match op {
                    UnaryOperator::Neg => {
                        if matches!(operand_type, Type::Int | Type::Float) {
                            Ok(operand_type)
                        } else {
                            Err(SemanticError {
                                message: "Negation requires numeric type".to_string(),
                            })
                        }
                    }
                    UnaryOperator::Not => {
                        if matches!(operand_type, Type::Bool) {
                            Ok(Type::Bool)
                        } else {
                            Err(SemanticError {
                                message: "Logical NOT requires boolean type".to_string(),
                            })
                        }
                    }
                }
            }

            Expr::Call(name, args) => {
                let function = self.functions.get(name)
                    .ok_or_else(|| SemanticError {
                        message: format!("Undefined function '{}'", name),
                    })?;

                if args.len() != function.params.len() {
                    return Err(SemanticError {
                        message: format!(
                            "Function '{}' expects {} arguments, got {}",
                            name, function.params.len(), args.len()
                        ),
                    });
                }

                for (i, arg) in args.iter().enumerate() {
                    let arg_type = self.check_expression(arg)?;
                    let param_type = &function.params[i].1;
                    
                    if !self.types_compatible(&arg_type, param_type) {
                        return Err(SemanticError {
                            message: format!(
                                "Argument {} to function '{}': expected {:?}, got {:?}",
                                i + 1, name, param_type, arg_type
                            ),
                        });
                    }
                }

                Ok(function.return_type.clone())
            }

            _ => Err(SemanticError {
                message: "Unsupported expression type".to_string(),
            }),
        }
    }

    fn types_compatible(&self, actual: &Type, expected: &Type) -> bool {
        match (actual, expected) {
            (Type::Int, Type::Float) => true,  // Allow int to float conversion
            (a, b) => a == b,
        }
    }
}

// main.rs - Updated main with enhanced features
mod lexer;
mod parser;
mod ast;
mod semantic;
mod codegen;

use lexer::Lexer;
use parser::Parser;
use semantic::SemanticAnalyzer;
use codegen::CodeGenerator;
use std::fs;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <source_file> [options]", args[0]);
        eprintln!("Options:");
        eprintln!("  --emit-asm    Emit assembly instead of object file");
        eprintln!("  --run         Compile and run immediately");
        std::process::exit(1);
    }

    let source_file = &args[1];
    let emit_asm = args.contains(&"--emit-asm".to_string());
    let run_immediately = args.contains(&"--run".to_string());

    println!("Compiling {}...", source_file);

    // Read source code
    let source = fs::read_to_string(source_file)
        .map_err(|e| format!("Failed to read source file: {}", e))?;

    // Lexical analysis
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();
    
    if std::env::var("ASTRAL_DEBUG").is_ok() {
        println!("Tokens: {:#?}", tokens);
    }

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse()
        .map_err(|e| format!("Parse error: {}", e))?;

    if std::env::var("ASTRAL_DEBUG").is_ok() {
        println!("AST: {:#?}", program);
    }

    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&program)
        .map_err(|e| format!("Semantic error: {}", e.message))?;

    println!("✓ Semantic analysis passed");

    // Code generation
    let mut codegen = CodeGenerator::new();
    let assembly = codegen.generate(&program);

    let output_name = source_file.strip_suffix(".astral")
        .unwrap_or(source_file);