//! # Hades-VM
//! 
//! A virtual machine implementation in Rust providing a simple stack-based architecture.
//! 
//! ## Overview
//! 
//! The Hades-VM is designed with the following main components:
//! 
//! - **VM**: The main virtual machine interface that encapsulates the CPU
//! - **CPU**: Central processing unit that executes instructions 
//! - **Stack**: A typed value stack with support for stack frames and function calls
//! - **Memory**: Segmented memory system with access controls
//! - **Instruction**: Representation of machine instructions
//! 
//! ## Module Structure
//! 
//! - `vm`: Core VM implementation that encapsulates the other components
//! - `cpu`: CPU implementation with fetch, decode, execute cycle
//! - `stack`: Stack implementation with value types and stack frames
//! - `alu`: Arithmetic Logic Unit for mathematical operations
//! - `instruction_processor`: Components for fetching, decoding, and executing instructions
//! - `instruction`: Instruction format and decoding
//! - `memory`: Memory management and segmentation
//! - `opcode`: Instruction opcodes and their properties

pub mod vm; 
pub mod cpu;
pub mod stack;
pub mod alu;
pub mod instruction_processor;
pub mod instruction; 
pub mod memory; 
pub mod opcode;
pub mod primitive_ops; 