use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Let,
    Mut,
    Fn,
    Struct,
    Enum,
    Match,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Break,
    Continue,
    True,
    False,
    
    // Types
    IntType,
    BoolType,
    StringType,
    CharType,
    
    // Literals
    Number(i64),
    StringLit(String),
    CharLit(char),
    Identifier(String),
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    Ampersand,
    EqualEqual,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Not,
    And,
    Or,
    
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Semicolon,
    Colon,
    Comma,
    Dot,
    Arrow,
    FatArrow,
    DotDot,
    
    // Special
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

pub struct Lexer<'a> {
    source: &'a str,
    filename: &'a str,
    chars: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, filename: &'a str) -> Self {
        Lexer {
            source,
            filename,
            chars: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            self.skip_whitespace_and_comments();
            
            if self.is_at_end() {
                break;
            }
            
            let token = self.next_token()?;
            tokens.push(token);
        }
        
        tokens.push(Token {
            token_type: TokenType::Eof,
            line: self.line,
            column: self.column,
        });
        
        Ok(tokens)
    }
    
    fn next_token(&mut self) -> Result<Token, String> {
        let line = self.line;
        let column = self.column;
        let ch = self.peek();
        
        let token_type = match ch {
            '+' => {
                self.advance();
                TokenType::Plus
            }
            '-' => {
                self.advance();
                if self.peek() == '>' {
                    self.advance();
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            }
            '*' => {
                self.advance();
                TokenType::Star
            }
            '/' => {
                self.advance();
                TokenType::Slash
            }
            '%' => {
                self.advance();
                TokenType::Percent
            }
            '=' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    TokenType::EqualEqual
                } else if self.peek() == '>' {
                    self.advance();
                    TokenType::FatArrow
                } else {
                    TokenType::Assign
                }
            }
            '<' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    TokenType::LessEqual
                } else {
                    TokenType::LessThan
                }
            }
            '>' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    TokenType::GreaterEqual
                } else {
                    TokenType::GreaterThan
                }
            }
            '!' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    TokenType::NotEqual
                } else {
                    TokenType::Not
                }
            }
            '&' => {
                self.advance();
                if self.peek() == '&' {
                    self.advance();
                    TokenType::And
                } else {
                    TokenType::Ampersand
                }
            }
            '|' => {
                self.advance();
                if self.peek() == '|' {
                    self.advance();
                    TokenType::Or
                } else {
                    return Err(format!("{}:{}:{}: Unexpected character: '|'", self.filename, line, column));
                }
            }
            '(' => {
                self.advance();
                TokenType::LParen
            }
            ')' => {
                self.advance();
                TokenType::RParen
            }
            '{' => {
                self.advance();
                TokenType::LBrace
            }
            '}' => {
                self.advance();
                TokenType::RBrace
            }
            '[' => {
                self.advance();
                TokenType::LBracket
            }
            ']' => {
                self.advance();
                TokenType::RBracket
            }
            ';' => {
                self.advance();
                TokenType::Semicolon
            }
            ':' => {
                self.advance();
                TokenType::Colon
            }
            ',' => {
                self.advance();
                TokenType::Comma
            }
            '.' => {
                self.advance();
                if self.peek() == '.' {
                    self.advance();
                    TokenType::DotDot
                } else {
                    TokenType::Dot
                }
            }
            '"' => self.read_string()?,
            '\'' => self.read_char()?,
            _ if ch.is_ascii_digit() => self.read_number(),
            _ if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
            _ => {
                return Err(format!(
                    "{}:{}:{}: Unexpected character: '{}'",
                    self.filename, line, column, ch
                ));
            }
        };
        
        Ok(Token {
            token_type,
            line,
            column,
        })
    }
    
    fn read_string(&mut self) -> Result<TokenType, String> {
        let line = self.line;
        let column = self.column;
        
        self.advance();
        let mut value = String::new();
        
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                return Err(format!(
                    "{}:{}:{}: Unterminated string literal",
                    self.filename, line, column
                ));
            }
            if self.peek() == '\\' {
                self.advance();
                let escaped = match self.peek() {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    _ => self.peek(),
                };
                value.push(escaped);
                self.advance();
            } else {
                value.push(self.advance());
            }
        }
        
        if self.is_at_end() {
            return Err(format!(
                "{}:{}:{}: Unterminated string literal",
                self.filename, line, column
            ));
        }
        
        self.advance();
        Ok(TokenType::StringLit(value))
    }
    
    fn read_char(&mut self) -> Result<TokenType, String> {
        let line = self.line;
        let column = self.column;
        
        self.advance();
        
        if self.is_at_end() {
            return Err(format!("{}:{}:{}: Unterminated char literal", self.filename, line, column));
        }
        
        let ch = if self.peek() == '\\' {
            self.advance();
            match self.peek() {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '\\' => '\\',
                '\'' => '\'',
                _ => return Err(format!("{}:{}:{}: Invalid escape sequence", self.filename, line, column)),
            }
        } else {
            self.peek()
        };
        
        self.advance();
        
        if self.peek() != '\'' {
            return Err(format!("{}:{}:{}: Unterminated char literal", self.filename, line, column));
        }
        
        self.advance();
        Ok(TokenType::CharLit(ch))
    }
    
    fn read_number(&mut self) -> TokenType {
        let mut value = String::new();
        
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            value.push(self.advance());
        }
        
        TokenType::Number(value.parse().unwrap())
    }
    
    fn read_identifier(&mut self) -> TokenType {
        let mut value = String::new();
        
        while !self.is_at_end() {
            let ch = self.peek();
            if ch.is_alphanumeric() || ch == '_' {
                value.push(self.advance());
            } else {
                break;
            }
        }
        
        match value.as_str() {
            "let" => TokenType::Let,
            "mut" => TokenType::Mut,
            "fn" => TokenType::Fn,
            "struct" => TokenType::Struct,
            "enum" => TokenType::Enum,
            "match" => TokenType::Match,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "return" => TokenType::Return,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "int" => TokenType::IntType,
            "bool" => TokenType::BoolType,
            "string" => TokenType::StringType,
            "char" => TokenType::CharType,
            _ => TokenType::Identifier(value),
        }
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                '/' if self.peek_ahead(1) == '/' => {
                    while !self.is_at_end() && self.peek() != '\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }
    
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }
    
    fn peek_ahead(&self, offset: usize) -> char {
        let pos = self.current + offset;
        if pos >= self.chars.len() {
            '\0'
        } else {
            self.chars[pos]
        }
    }
    
    fn advance(&mut self) -> char {
        let ch = self.chars[self.current];
        self.current += 1;
        self.column += 1;
        ch
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }
}