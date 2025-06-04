use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    #[token("fun")]
    Fun,
    #[token("val")]
    Val,
    #[token("var")]
    Var,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("return")]
    Return,
    #[token("true")]
    True,
    #[token("false")]
    False,

    // Types
    #[token("Int")]
    Int,
    #[token("Float")]
    Float,
    #[token("Bool")]
    Bool,
    #[token("String")]
    String,

    // Built-in functions for memory operations
    #[token("alloc")]
    Alloc,
    #[token("free")]
    Free,
    #[token("load")]
    Load,
    #[token("store")]
    Store,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Multiply,
    #[token("/")]
    Divide,
    #[token("=")]
    Assign,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,

    // Delimiters
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,

    // Literals and identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    #[regex(r"-?[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    IntegerLiteral(i64),
    
    #[regex(r"-?[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    FloatLiteral(f64),
    
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        // Remove quotes and handle escape sequences
        s[1..s.len()-1].to_string()
    })]
    StringLiteral(String),

    // Skip whitespace and comments
    #[regex(r"[ \t\r\n\f]+", logos::skip)]
    #[regex(r"//[^\r\n]*", logos::skip)]
    #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip)]
    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Fun => write!(f, "fun"),
            Token::Val => write!(f, "val"),
            Token::Var => write!(f, "var"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::While => write!(f, "while"),
            Token::Return => write!(f, "return"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Int => write!(f, "Int"),
            Token::Float => write!(f, "Float"),
            Token::Bool => write!(f, "Bool"),
            Token::String => write!(f, "String"),
            Token::Alloc => write!(f, "alloc"),
            Token::Free => write!(f, "free"),
            Token::Load => write!(f, "load"),
            Token::Store => write!(f, "store"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Assign => write!(f, "="),
            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Not => write!(f, "!"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Identifier(name) => write!(f, "{}", name),
            Token::IntegerLiteral(value) => write!(f, "{}", value),
            Token::FloatLiteral(value) => write!(f, "{}", value),
            Token::StringLiteral(value) => write!(f, "\"{}\"", value),
            Token::Error => write!(f, "ERROR"),
        }
    }
}

pub struct Lexer<'input> {
    lexer: logos::Lexer<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            lexer: Token::lexer(input),
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        match self.lexer.next() {
            Some(Ok(token)) => Some(token),
            Some(Err(_)) => Some(Token::Error),
            None => None,
        }
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    
    while let Some(token) = lexer.next_token() {
        if token != Token::Error {
            tokens.push(token);
        }
    }
    
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let input = "fun main(): Int { return 42 }";
        let tokens = tokenize(input);
        
        assert_eq!(tokens, vec![
            Token::Fun,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::Colon,
            Token::Int,
            Token::LeftBrace,
            Token::Return,
            Token::IntegerLiteral(42),
            Token::RightBrace,
        ]);
    }

    #[test]
    fn test_arithmetic() {
        let input = "val x = a + b * c";
        let tokens = tokenize(input);
        
        assert_eq!(tokens, vec![
            Token::Val,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::Identifier("a".to_string()),
            Token::Plus,
            Token::Identifier("b".to_string()),
            Token::Multiply,
            Token::Identifier("c".to_string()),
        ]);
    }
} 