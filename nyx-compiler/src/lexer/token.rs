#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Let,
    Var,
    Mut,
    Fn,
    Return,
    If,
    Else,
    When,
    Is,
    Trait,
    Impl,
    Enum,
    Struct,
    Async,
    Await,
    
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    
    // Identifiers
    Identifier(String),
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Assign,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Arrow,
    FatArrow,
    
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Colon,
    Semicolon,
    
    // Special
    EOF,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
    
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self::new(TokenKind::Error(message.into()), span)
    }
} 