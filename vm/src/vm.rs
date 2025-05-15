/// Main Virtual Machine implementation for the Hades VM.
///
/// This module provides the high-level VM interface that encapsulates
/// the CPU and other components to execute programs.

use crate::{
    cpu::CPU,
    stack::StackValue,
};

/// Main Virtual Machine implementation that encapsulates the CPU.
///
/// The VirtualMachine provides a high-level interface for:
/// - Loading and running programs
/// - Managing execution
/// - Inspecting machine state
pub struct VirtualMachine {
    /// The CPU that executes instructions
    pub cpu: CPU,
}

impl VirtualMachine {
    /// Creates a new VirtualMachine with default memory size.
    pub fn new() -> Self {
        Self { cpu: CPU::new(10000) }
    }

    /// Creates a new VirtualMachine with the specified memory size.
    pub fn with_memory_size(size: usize) -> Self {
        Self { cpu: CPU::new(size) }
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
    pub fn run_until_completion(&mut self) {
        while self.run() {}
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
        
        let mut vm = VirtualMachine::new();
        vm.load_program(&program);
        
        // Run until completion
        vm.run_until_completion();
        
        // Check the result
        assert_eq!(vm.stack_top(), Some(StackValue::Integer(7)));
    }

    #[test]
    fn test_vm_basic() {
        // Create a simple VM
        let mut vm = VirtualMachine::new();
        
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