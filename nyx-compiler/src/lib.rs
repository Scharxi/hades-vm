//! Nyx Programming Language Compiler
//! 
//! This crate provides the compiler implementation for the Nyx programming language,
//! a modern language that combines features from Kotlin and Rust.

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod codegen;
pub mod hex_compiler;

/// Re-export commonly used types
pub use lexer::{Lexer, Token};
pub use parser::{Parser, ParseError};
pub use ast::*;
pub use codegen::{CodeGenerator, CodeGenError};
pub use hex_compiler::{HexCompiler, HexCompilerError, compile_source_to_hex_file};

/// Initialize the logger for the compiler
pub fn init_logger() {
    env_logger::init();
}

pub fn compile_to_bytecode(source: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Parse the source code
    let mut parser = Parser::new(source);
    let program = parser.parse_program()?;
    
    // Generate bytecode
    let mut codegen = CodeGenerator::new()?;
    let bytecode = codegen.generate(&program)?;
    
    Ok(bytecode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_function() {
        let source = r#"
            fun main(): Int {
                return 42
            }
        "#;
        
        let bytecode = compile_to_bytecode(source).unwrap();
        assert!(!bytecode.is_empty());
    }

    #[test]
    fn test_compile_arithmetic() {
        let source = r#"
            fun add(a: Int, b: Int): Int {
                return a + b
            }
            
            fun main(): Int {
                return add(10, 5)
            }
        "#;
        
        let bytecode = compile_to_bytecode(source).unwrap();
        assert!(!bytecode.is_empty());
    }
} 