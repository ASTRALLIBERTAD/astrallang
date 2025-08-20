use core::{option::Option::Some, result::Result::Ok, write};

use crate::compiler::ast::{Type, Value};

#[derive(Clone, Debug)]
pub enum Token {
    // Literals
    Literal(Value),
    Ident(String),
    StringLiteral(String),
    Bool(bool),

    // Keywords
    Match,
    Let,
    Fn,
    If,
    Else,
    While,
    Return,
    Print,
    As, // For casting

    // Type keywords
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128,
    F32, F64, Usize,
    BoolType,
    StringType,
    StrType,
    Any,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Bang,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Comma,
    Colon,
    // Sign
    Arrow,
    Underscore,

    // Special
    EOF,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Arrow => write!(f, "=>"),
            Token::Underscore => write!(f, "_"),
            Token::Literal(v) => write!(f, "Literal({:?})", v),
            Token::Ident(s) => write!(f, "Ident({})", s),
            Token::StringLiteral(s) => write!(f, "\"{}\"", s),
            Token::Bool(b) => write!(f, "Bool({})", b),
            Token::Match => write!(f, "match"),
            Token::Let => write!(f, "let"),
            Token::Fn => write!(f, "fn"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::While => write!(f, "while"),
            Token::Return => write!(f, "return"),
            Token::Print => write!(f, "print"),
            Token::As => write!(f, "as"),
            Token::I8 => write!(f, "i8"),
            Token::I16 => write!(f, "i16"),
            Token::I32 => write!(f, "i32"),
            Token::I64 => write!(f, "i64"),
            Token::I128 => write!(f, "i128"),
            Token::U8 => write!(f, "u8"),
            Token::U16 => write!(f, "u16"),
            Token::U32 => write!(f, "u32"),
            Token::U64 => write!(f, "u64"),
            Token::U128 => write!(f, "u128"),
            Token::F32 => write!(f, "f32"),
            Token::F64 => write!(f, "f64"),
            Token::Usize => write!(f, "usize"),
            Token::BoolType => write!(f, "bool"),
            Token::StringType => write!(f, "string"),
            Token::StrType => write!(f, "str"),
            Token::Any => write!(f, "any"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Equal => write!(f, "="),
            Token::EqualEqual => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::Bang => write!(f, "!"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::EOF => write!(f, "EOF"),
        }
    }
}

// Add PartialEq implementation for Token comparisons
impl PartialEq<Token> for Token {
    fn eq(&self, other: &Token) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Lexer error at line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for LexError {}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            if let Some(token) = self.next_token()? {
                tokens.push(token);
            }
        }

        tokens.push(Token::EOF);
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(None);
        }

        let ch = self.current_char();
        self.advance();

        match ch {
            '+' => Ok(Some(Token::Plus)),
            '-' => Ok(Some(Token::Minus)),
            '*' => Ok(Some(Token::Star)),
            '/' => {
                if self.match_char('/') {
                    self.skip_line_comment();
                    self.next_token()
                } else if self.match_char('*') {
                    self.skip_block_comment()?;
                    self.next_token()
                } else {
                    Ok(Some(Token::Slash))
                }
            }
            '%' => Ok(Some(Token::Percent)),
            '=' => {
                if self.match_char('=') {
                    Ok(Some(Token::EqualEqual))
                } else if self.match_char('>') {
                    Ok(Some(Token::Arrow))
                }
                else {
                    Ok(Some(Token::Equal))
                }
            }
            '!' => {
                if self.match_char('=') {
                    Ok(Some(Token::NotEqual))
                } else {
                    Ok(Some(Token::Bang))
                }
            }
            '<' => {
                if self.match_char('=') {
                    Ok(Some(Token::LessEqual))
                } else {
                    Ok(Some(Token::Less))
                }
            }
            '>' => {
                if self.match_char('=') {
                    Ok(Some(Token::GreaterEqual))
                } else {
                    Ok(Some(Token::Greater))
                }
            }
            '(' => Ok(Some(Token::LParen)),
            ')' => Ok(Some(Token::RParen)),
            '{' => Ok(Some(Token::LBrace)),
            '}' => Ok(Some(Token::RBrace)),
            ';' => Ok(Some(Token::Semicolon)),
            ',' => Ok(Some(Token::Comma)),
            ':' => Ok(Some(Token::Colon)),
            '"' => self.read_string(),
            '_' => Ok(Some(Token::Underscore)),
            _ => {
                if ch.is_ascii_digit() {
                    self.pos -= 1;
                    self.column -= 1;
                    Ok(Some(self.read_number()?))
                } else if ch.is_ascii_alphabetic() || ch == '_' {
                    self.pos -= 1;
                    self.column -= 1;
                    Ok(Some(self.read_identifier()))
                } else {
                    Err(LexError {
                        message: format!("Unexpected character: '{}'", ch),
                        line: self.line,
                        column: self.column - 1,
                    })
                }
            }
        }
    }

    fn read_string(&mut self) -> Result<Option<Token>, LexError> {
        let mut string = String::new();
        let start_line = self.line;
        let start_column = self.column - 1; // Account for opening quote
        
        while !self.is_at_end() && self.current_char() != '"' {
            let ch = self.current_char();
            if ch == '\\' {
                self.advance();
                if self.is_at_end() {
                    return Err(LexError {
                        message: "Unterminated string literal".to_string(),
                        line: start_line,
                        column: start_column,
                    });
                }
                
                match self.current_char() {
                    'n' => string.push('\n'),
                    't' => string.push('\t'),
                    'r' => string.push('\r'),
                    '\\' => string.push('\\'),
                    '"' => string.push('"'),
                    '\'' => string.push('\''),
                    '0' => string.push('\0'),
                    c => {
                        return Err(LexError {
                            message: format!("Invalid escape sequence: \\{}", c),
                            line: self.line,
                            column: self.column - 1,
                        });
                    }
                }
            } else {
                if ch == '\n' {
                    self.line += 1;
                    self.column = 0;
                }
                string.push(ch);
            }
            self.advance();
        }
        
        if self.is_at_end() {
            return Err(LexError {
                message: "Unterminated string literal".to_string(),
                line: start_line,
                column: start_column,
            });
        }
        
        self.advance(); // closing "
        Ok(Some(Token::StringLiteral(string)))
    }

    fn read_number(&mut self) -> Result<Token, LexError> {
        let mut num_str = String::new();

        // Handle special number formats (hex, binary, octal)
        if self.current_char() == '0' && !self.is_at_end() {
            if let Some(next_ch) = self.peek() {
                match next_ch {
                    'x' | 'X' => return self.read_hex_number(),
                    'b' | 'B' => return self.read_binary_number(),
                    'o' | 'O' => return self.read_octal_number(),
                    _ => {}
                }
            }
        }

        // Read the integer part
        while !self.is_at_end()
            && (self.current_char().is_ascii_digit() || self.current_char() == '_')
        {
            if self.current_char() != '_' {
                num_str.push(self.current_char());
            }
            self.advance();
        }

        // Check for decimal point
        if !self.is_at_end() && self.current_char() == '.' {
            // Look ahead to make sure it's actually a float and not something like "5.method()"
            if let Some(next_char) = self.peek() {
                if next_char.is_ascii_digit() {
                    num_str.push('.');
                    self.advance(); // consume the '.'
                    
                    // Read fractional part
                    while !self.is_at_end()
                        && (self.current_char().is_ascii_digit() || self.current_char() == '_')
                    {
                        if self.current_char() != '_' {
                            num_str.push(self.current_char());
                        }
                        self.advance();
                    }
                    
                    // Parse as float
                    return num_str
                        .parse::<f64>()
                        .map(|f| Token::Literal(Value::F64(f)))
                        .map_err(|_| self.error(&format!("Invalid float: {}", num_str)));
                }
            }
        }

        // Parse as integer
        if num_str.is_empty() {
            return Err(self.error("Invalid number format"));
        }

        num_str
            .parse::<crate::compiler::ast::Value >()
            .map(Token::Literal)
            .map_err(|_| self.error(&format!("Invalid number: {}", num_str)))
    }

    fn read_hex_number(&mut self) -> Result<Token, LexError> {
        self.advance(); // consume '0'
        self.advance(); // consume 'x' or 'X'
        
        let mut hex_str = String::new();
        
        while !self.is_at_end()
            && (self.current_char().is_ascii_hexdigit() || self.current_char() == '_')
        {
            if self.current_char() != '_' {
                hex_str.push(self.current_char());
            }
            self.advance();
        }
        
        if hex_str.is_empty() {
            return Err(self.error("Invalid hexadecimal number"));
        }
        
        i64::from_str_radix(&hex_str, 16)
            .map(|v| Token::Literal(Value::I64(v)))
            .map_err(|_| self.error("Invalid hexadecimal number"))
    }

    fn read_binary_number(&mut self) -> Result<Token, LexError> {
        self.advance(); // consume '0'
        self.advance(); // consume 'b' or 'B'
        
        let mut bin_str = String::new();
        
        while !self.is_at_end()
            && (self.current_char() == '0' || self.current_char() == '1' || self.current_char() == '_')
        {
            if self.current_char() != '_' {
                bin_str.push(self.current_char());
            }
            self.advance();
        }
        
        if bin_str.is_empty() {
            return Err(self.error("Invalid binary number"));
        }
        
        i64::from_str_radix(&bin_str, 2)
            .map(|v| Token::Literal(Value::I64(v)))
            .map_err(|_| self.error("Invalid binary number"))
    }

    fn read_octal_number(&mut self) -> Result<Token, LexError> {
        self.advance(); // consume '0'
        self.advance(); // consume 'o' or 'O'
        
        let mut oct_str = String::new();
        
        while !self.is_at_end()
            && (('0'..='7').contains(&self.current_char()) || self.current_char() == '_')
        {
            if self.current_char() != '_' {
                oct_str.push(self.current_char());
            }
            self.advance();
        }
        
        if oct_str.is_empty() {
            return Err(self.error("Invalid octal number"));
        }
        
        i64::from_str_radix(&oct_str, 8)
            .map(|v| Token::Literal(Value::I64(v)))
            .map_err(|_| self.error("Invalid octal number"))
    }

    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();

        while !self.is_at_end()
            && (self.current_char().is_ascii_alphanumeric() || self.current_char() == '_')
        {
            ident.push(self.current_char());
            self.advance();
        }

        match ident.as_str() {
            // Keywords
            "match" => Token::Match,
            "let" => Token::Let,
            "fn" => Token::Fn,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "return" => Token::Return,
            "print" => Token::Print,
            "as" => Token::As,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            
            // Type keywords
            "i8" => Token::I8,
            "i16" => Token::I16,
            "i32" => Token::I32,
            "i64" => Token::I64,
            "i128" => Token::I128,
            "u8" => Token::U8,
            "u16" => Token::U16,
            "u32" => Token::U32,
            "u64" => Token::U64,
            "u128" => Token::U128,
            "f32" => Token::F32,
            "f64" => Token::F64,
            "usize" => Token::Usize,
            "bool" => Token::BoolType,
            "string" => Token::StringType,
            "str" => Token::StrType,
            "any" => Token::Any,
            
            _ => Token::Ident(ident),
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.current_char() {
                ' ' | '\r' | '\t' => self.advance(),
                '\n' => {
                    self.line += 1;
                    self.column = 0;
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), LexError> {
        let mut depth = 1;

        while !self.is_at_end() && depth > 0 {
            if self.current_char() == '/' && self.peek() == Some('*') {
                self.advance();
                self.advance();
                depth += 1;
            } else if self.current_char() == '*' && self.peek() == Some('/') {
                self.advance();
                self.advance();
                depth -= 1;
            } else {
                if self.current_char() == '\n' {
                    self.line += 1;
                    self.column = 0;
                }
                self.advance();
            }
        }

        if depth > 0 {
            Err(self.error("Unterminated block comment"))
        } else {
            Ok(())
        }
    }

    fn current_char(&self) -> char {
        self.input.get(self.pos).copied().unwrap_or('\0')
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
            self.column += 1;
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.current_char() != expected {
            false
        } else {
            self.advance();
            true
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn error(&self, msg: &str) -> LexError {
        LexError {
            message: msg.to_string(),
            line: self.line,
            column: self.column,
        }
    }
}