use crate::compiler::value;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    // Literals
    Number(i64),
    Ident(String),
    String(String),
    Bool(bool),
    
    // Keywords
    Let,
    Fn,
    If,
    Else,
    While,
    Return,
    Print,

    
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
    
    // Special
    EOF,

}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Number(n) => write!(f, "Number({})", n),
            Token::Ident(s) => write!(f, "Ident({})", s),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Let => write!(f, "let"),
            Token::Fn => write!(f, "fn"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::While => write!(f, "while"),
            Token::Return => write!(f, "return"),
            Token::Print => write!(f, "print"),
            Token::Bool(b) => write!(f, "Bool({})", b),
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

#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lexer error at line {}, column {}: {}", self.line, self.column, self.message)
    }
}

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
                // Handle comments
                if self.match_char('/') {
                    self.skip_line_comment();
                    self.next_token()
                } else if self.match_char('*') {
                    self.skip_block_comment()?;
                    self.next_token()
                } else {
                    Ok(Some(Token::Slash))
                }
            },
            '%' => Ok(Some(Token::Percent)),
            '=' => {
                if self.match_char('=') {
                    Ok(Some(Token::EqualEqual))
                } else {
                    Ok(Some(Token::Equal))
                }
            },
            '!' => {
                if self.match_char('=') {
                    Ok(Some(Token::NotEqual))
                } else {
                    Ok(Some(Token::Bang))
                }
            },
            '<' => {
                if self.match_char('=') {
                    Ok(Some(Token::LessEqual))
                } else {
                    Ok(Some(Token::Less))
                }
            },
            '>' => {
                if self.match_char('=') {
                    Ok(Some(Token::GreaterEqual))
                } else {
                    Ok(Some(Token::Greater))
                }
            },
            '(' => Ok(Some(Token::LParen)),
            ')' => Ok(Some(Token::RParen)),
            '{' => Ok(Some(Token::LBrace)),
            '}' => Ok(Some(Token::RBrace)),
            ';' => Ok(Some(Token::Semicolon)),
            ',' => Ok(Some(Token::Comma)),
            ':' => Ok(Some(Token::Colon)),
            
            '"' => {
                let mut string = String::new();
                while !self.is_at_end() && self.current_char() != '"' {
                    string.push(self.current_char());
                    self.advance();
                }
                if self.is_at_end() {
                    return Err(LexError {
                        message: "Unterminated string literal".to_string(),
                        line: self.line,
                        column: self.column,
                    });
                }
                self.advance(); // consume closing "
                Ok(Some(Token::String(string)))
            },
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
    
    fn read_number(&mut self) -> Result<Token, LexError> {
        let mut num_str = String::new();

        if self.match_char('0') {
            num_str.push('0');
            if let Some(next) = self.current_char().to_digit(16) {
                if self.match_char('x') {
                    while !self.is_at_end()
                        && (self.current_char().is_ascii_hexdigit() || self.current_char() == '_')
                    {
                        num_str.push(self.current_char());
                        self.advance();
                    }
                    let clean = num_str.replace("_", "").trim_start_matches("0x");
                    let value = i64::from_str_radix(&clean, 16)
                        .map_err(|_| self.error("Invalid hexadecimal number"))?;
                        return Ok(Token::Number(value));
                } else if self.match_char('b') {

                    while !self.is_at_end() && (self.current_char() == '0'|| self.current_char() == '1' || self.current_char() == '_') 
                    {
                        num_str.push(self.current_char());
                        self.advance();
                    }
                    let clean = num_str.replace("_", "").trim_start_matches("0b");
                    let  value = i64::from_str_radix(clean, 2).map_err(|_| self.error("Invalid binary number"))?;
                    return Ok(Token::Number(value));
                        
                }else if self.match_char('o') {
                    while !self.is_at_end() && (('0'..='7').contains(&self.current_char() || self.current_char() == '_')) {
                        num_str.push(self.current_char());
                        self.advance();
                    }

                            let clean = num_str.replace("_", "").trim_start_matches("Oo") ;
                            let value = i64::from_str_radix(clean, 8).map_err(|_| self.error("Invalid octal number"))?; ;
                            return Ok(Token::Number(value));
                }
            }
        }

        while !self.is_at_end() && (self.current_char().is_ascii_digit() || self.current_char() == '_') {
            num_str.push(self.current_char());
            self.advance();
            
        }

        let clean = num_str.replace("_", "");
        clean.parse::<i64>()
            .map(Token::Number)
            .map_err(|_| self.error(&format!("Invalid number: {}", clean)))
            
    }
        
    
    
    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();
        
        while !self.is_at_end() && (self.current_char().is_ascii_alphanumeric() || self.current_char() == '_') {
            ident.push(self.current_char());
            self.advance();
        }
        
        match ident.as_str() {
            "let" => Token::Let,
            "fn" => Token::Fn,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "return" => Token::Return,
            "print" => Token::Print,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
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
                },
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
            Err(LexError {
                message: "Unterminated block comment".to_string(),
                line: self.line,
                column: self.column,
            })
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
}