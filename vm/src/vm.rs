/// Main Virtual Machine implementation for the Hades VM.
///
/// This module provides the high-level VM interface that encapsulates
/// the CPU and other components to execute programs.

use thiserror::Error;
use crate::{
    cpu::CPU,
    stack::StackValue,
};

#[derive(Error, Debug)]
pub enum VMError {
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u8),
    
    #[error("Stack underflow")]
    StackUnderflow,
    
    #[error("Stack overflow")]
    StackOverflow,
    
    #[error("Invalid memory access")]
    InvalidMemoryAccess,
    
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

/// Main Virtual Machine implementation that encapsulates the CPU.
///
/// The VirtualMachine provides a high-level interface for:
/// - Loading and running programs
/// - Managing execution
/// - Inspecting machine state
pub struct VirtualMachine {
    /// The CPU that executes instructions
    pub cpu: CPU,
    /// Debug mode
    debug: bool,
}

impl VirtualMachine {
    /// Creates a new VirtualMachine with default memory size.
    pub fn new(debug: bool) -> Self {
        Self {
            cpu: CPU::new(65536), // 64KB memory
            debug,
        }
    }

    /// Creates a new VirtualMachine with the specified memory size.
    pub fn with_memory_size(size: usize) -> Self {
        Self { cpu: CPU::new(size), debug: false }
    }

    /// Executes a single instruction step.
    ///
    /// Returns true if an instruction was executed, false if
    /// there are no more instructions.
    pub fn run(&mut self) -> bool {
        self.cpu.step()
    }

    /// Loads a program into the VM.
    pub fn load_program(&mut self, program: &[u8]) {
        self.cpu.load_program(program);
    }

    /// Runs the program until completion (no more instructions).
    pub fn run_until_completion(&mut self) -> Result<(), String> {
        while self.run() {
            // Check if there was a panic in the last instruction
            if let Some(frame) = self.cpu.stack.current_frame {
                if frame >= self.cpu.stack.frames.len() {
                    return Err("Invalid stack frame".to_string());
                }
            }
        }
        Ok(())
    }
    
    /// Gets the top value on the stack.
    pub fn stack_top(&self) -> Option<StackValue> {
        self.cpu.stack.peek().map(|value| value.clone())
    }
    
    /// Prints a debug view of the memory map.
    pub fn print_memory_map(&self) {
        self.cpu.memory.print_memory_map();
    }
    
    /// Prints the current stack state.
    pub fn print_stack_state(&self) {
        println!("Stack Depth: {}", self.cpu.stack.len());
        println!("Frame Depth: {}", self.cpu.stack.frame_depth());
    }

    pub fn push_arg(&mut self, value: i32) -> Result<(), VMError> {
        // Arguments are pushed onto the stack before the stack frame is created
        // They will become local variables when the frame is created
        self.cpu.stack.push(StackValue::Integer(value));
        
        if self.debug {
            println!("Pushed argument: {}, stack: {:?}", value, self.cpu.stack.values);
        }
        
        Ok(())
    }

    pub fn execute(&mut self, bytecode: &[u8]) -> Result<i64, VMError> {
        // Save the arguments that are already on the stack
        let saved_args = self.cpu.stack.values.clone();

        // Load the bytecode into the VM
        self.cpu.load_program(bytecode);

        // Restore the arguments
        self.cpu.stack.values = saved_args;

        if self.debug {
            println!("Starting execution with arguments: {:?}", self.cpu.stack.values);
        }

        // Run the program until we get a return value or error
        let mut result = None;
        while self.cpu.step() {
            // Check if we have a return value and no frames
            if self.cpu.stack.frames.is_empty() && !self.cpu.stack.values.is_empty() {
                // Get the result and stop execution
                result = match self.cpu.stack.values.last() {
                    Some(StackValue::Integer(val)) => Some(*val as i64),
                    _ => None
                };
                break;
            }
        }

        // Return the result or error
        match result {
            Some(val) => Ok(val),
            None => Err(VMError::RuntimeError("No return value from main function".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack::StackValue;

    #[test]
    fn test_vm_execution() {
        // Create a simple program that adds two numbers (3 + 4 = 7)
        // Format: opcode is last byte, operand is 24 bits [0-2]
        let program = vec![
            0x00, 0x00, 0x03, 0x04, // LoadConstant (opcode 4) with operand 3
            0x00, 0x00, 0x04, 0x04, // LoadConstant (opcode 4) with operand 4
            0x00, 0x00, 0x00, 0x01, // Add (opcode 1)
        ];
        
        let mut vm = VirtualMachine::new(false);
        vm.load_program(&program);
        
        // Run until completion
        vm.run_until_completion();
        
        // Check the result
        assert_eq!(vm.stack_top(), Some(StackValue::Integer(7)));
    }

    #[test]
    fn test_vm_basic() {
        // Create a simple VM
        let mut vm = VirtualMachine::new(false);
        
        // Create a simple program that loads a constant
        // LoadConstant with operand 42 (0x2A)
        // Format: opcode is last byte, operand is 24 bits [0-2]
        let program = vec![
            0x00, 0x00, 0x2A, 0x04, // LoadConstant (opcode 4) with operand 42 (0x2A) in third byte
        ];
        
        // Load the program
        vm.load_program(&program);
        
        // Execute one instruction
        let executed = vm.run();
        assert!(executed, "Failed to execute the instruction");
        
        // Check the stack top value
        match vm.stack_top() {
            Some(StackValue::Integer(val)) => assert_eq!(val, 42),
            _ => panic!("Expected Integer(42) on stack top, got {:?}", vm.stack_top()),
        }
    }
}