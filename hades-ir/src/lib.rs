//! # Hades IR
//! 
//! An LLVM-like intermediate representation for compiling higher-level languages to Hades VM bytecode.
//! 
//! This crate provides an intermediate representation (IR) that sits between a high-level language
//! frontend and the Hades VM's low-level bytecode. It allows for optimization, type checking,
//! and complex language feature implementations before generating the final VM bytecode.
//! 
//! ## Core Components
//! 
//! - **Module**: The top-level container for IR code, similar to an LLVM module
//! - **Function**: Represents a function with basic blocks and parameters
//! - **BasicBlock**: A sequence of instructions with a single entry and exit point
//! - **Instruction**: An IR-level instruction that will be lowered to VM bytecode
//! - **Type**: A type system for IR-level type checking and inference
//! - **Value**: A typed value in the IR, representing constants, variables, etc.
//! - **Builder**: A helper for constructing IR programmatically
//! - **Pass**: Transformations and optimizations that can be applied to the IR
//! 
//! ## Usage
//! 
//! The IR is typically used by creating a `Module`, adding `Function`s to it, and then using
//! a `Builder` to populate each function with instructions. The module can then be passed
//! through various analysis and optimization `Pass`es before finally being compiled to
//! Hades VM bytecode.

// Core components
pub mod module;
pub mod function;
pub mod basic_block;
pub mod instruction;
pub mod types;
pub mod value;
pub mod builder;
// pub mod pass;
pub mod error;
pub mod context;
// pub mod optimization;
// pub mod target;
// pub mod verifier;
pub mod bytecode;
mod type_checker;

// Re-export core components
pub use context::Context;
pub use module::Module;
pub use function::{Function, Linkage};
pub use basic_block::BasicBlock;
pub use instruction::{Instruction, Operation};
pub use types::Type;
pub use value::Value;
pub use builder::Builder;
pub use error::{Error, Result};
pub use bytecode::BytecodeGenerator;
pub use type_checker::TypeChecker;
// pub use target::Target;

/// Version of the IR specification
pub const IR_VERSION: &str = "0.1.0";

/// Main entry point for creating a new IR compilation context
pub fn create_context() -> Context {
    Context::new()
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
