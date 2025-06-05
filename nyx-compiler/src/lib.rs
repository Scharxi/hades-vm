//! Nyx Programming Language Compiler
//! 
//! This crate provides the compiler implementation for the Nyx programming language,
//! a modern language that combines features from Kotlin and Rust.

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod module_parser;
pub mod module_resolver;
pub mod codegen;
pub mod hex_compiler;
pub mod visibility_tests;
pub mod nested_module_tests;
pub mod class_tests;
pub mod primitive_types_tests;

/// Re-export commonly used types
pub use lexer::{Lexer, Token};
pub use parser::{Parser, ParseError};
pub use ast::*;
pub use codegen::{CodeGenerator, CodeGenError};
pub use hex_compiler::{HexCompiler, HexCompilerError, compile_source_to_hex_file};
pub use module_resolver::{ModuleResolver, ResolvedModule, create_stdlib_resolver};

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

    #[test]
    fn test_module_parsing() {
        let source = r#"
            import std.io
            
            pub mod math_utils {
                pub fun square(x: Int): Int {
                    return x * x
                }
            }
            
            fun main(): Int {
                return math_utils.square(5)
            }
        "#;
        
        let mut parser = Parser::new(source);
        let program = parser.parse_module_aware_program().unwrap();
        
        assert_eq!(program.imports.len(), 1);
        assert_eq!(program.modules.len(), 1);
        assert_eq!(program.functions.len(), 1);
    }

    #[test]
    fn test_tokenization() {
        let source = r#"
            import std.io
            
            pub mod math_utils {
                pub fun square(x: Int): Int {
                    return x * x
                }
            }
            
            fun main(): Int {
                return math_utils.square(5)
            }
        "#;
        
        let tokens = crate::lexer::tokenize(source);
        println!("Tokens:");
        for (i, token) in tokens.iter().enumerate() {
            println!("{}: {:?}", i, token);
        }
        
        assert!(!tokens.is_empty());
    }
} 