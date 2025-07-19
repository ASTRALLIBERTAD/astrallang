#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Fn,
    Let,
    Ident(String),
    Number(i64),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Equal,
    Semicolon,
    Arrow,
    EOF,
    Return,
    Minus,
    Plus,
    Star,
    Slash,
}





pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\n' | '\t' => { chars.next(); }
            '=' => { tokens.push(Token::Equal); chars.next(); }
            ';' => { tokens.push(Token::Semicolon); chars.next(); }
            '(' => { tokens.push(Token::LParen); chars.next(); }
            ')' => { tokens.push(Token::RParen); chars.next(); }
            '{' => { tokens.push(Token::LBrace); chars.next(); }
            '}' => { tokens.push(Token::RBrace); chars.next(); }
            '+' => { tokens.push(Token::Plus); chars.next(); }
            '*' => { tokens.push(Token::Star); chars.next(); }
            '/' => { tokens.push(Token::Slash); chars.next(); }
            '-' => { tokens.push(Token::Minus); chars.next(); }
            // "return" => { tokens.push(Token::Return); chars.next(); }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::Arrow);
                }
            }
            '0'..='9' => {
                let mut num = String::new();
                while chars.peek().map(|c| c.is_digit(10)) == Some(true) {
                    num.push(chars.next().unwrap());
                }
                tokens.push(Token::Number(num.parse().unwrap()));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while chars.peek().map(|c| c.is_alphanumeric() || *c == '_') == Some(true) {
                    ident.push(chars.next().unwrap());
                }
                match ident.as_str() {
                    "fn" => tokens.push(Token::Fn),
                    "let" => tokens.push(Token::Let),
                    _ => tokens.push(Token::Ident(ident)),
                }
            }
            _ => { chars.next(); }
        }
    }

    tokens.push(Token::EOF);
    tokens
}
