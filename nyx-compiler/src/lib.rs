//! Nyx Programming Language Compiler
//! 
//! This crate provides the compiler implementation for the Nyx programming language,
//! a modern language that combines features from Kotlin and Rust.

pub mod lexer;
pub mod parser;
pub mod ast;

/// Re-export commonly used types
pub use lexer::{Lexer, Token, TokenKind, Span};

/// Initialize the logger for the compiler
pub fn init_logger() {
    env_logger::init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_integration() {
        let source = r#"
            fn main() {
                let x = 42;
                println("Hello, Nyx!");
            }
        "#;
        
        let mut lexer = Lexer::new(source);
        let tokens: Vec<Token> = std::iter::from_fn(move || {
            let token = lexer.next_token();
            if token.kind == TokenKind::EOF {
                None
            } else {
                Some(token)
            }
        }).collect();
        
        assert!(tokens.len() > 0);
        assert!(tokens.iter().all(|t| matches!(t.kind, TokenKind::Error(_)) == false));
    }
} 