use thiserror::Error;
use vm::vm::VirtualMachine;
use vm::stack::StackValue;

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

pub struct VM {
    /// The underlying virtual machine
    vm: VirtualMachine,
    /// Debug mode
    debug: bool,
}

impl VM {
    pub fn new(debug: bool) -> Self {
        Self {
            vm: VirtualMachine::with_memory_size(65536), // 64KB memory
            debug,
        }
    }

    pub fn push_arg(&mut self, value: i32) -> Result<(), VMError> {
        // Arguments are pushed onto the stack before the stack frame is created
        // They will become local variables when the frame is created
        self.vm.cpu.stack.push(StackValue::Integer(value));
        Ok(())
    }

    pub fn execute(&mut self, bytecode: &[u8]) -> Result<i64, VMError> {
        // Save the arguments that are already on the stack
        let saved_args = self.vm.cpu.stack.values.clone();

        // Load the bytecode into the VM
        self.vm.load_program(bytecode);

        // Restore the arguments
        self.vm.cpu.stack.values = saved_args;

        // Run the program
        if let Err(e) = self.vm.run_until_completion() {
            return Err(VMError::RuntimeError(format!("VM error: {:?}", e)));
        }

        // Get the result from the top of the stack
        match self.vm.stack_top() {
            Some(StackValue::Integer(result)) => Ok(result as i64),
            Some(other) => Err(VMError::RuntimeError(format!(
                "Expected integer result, got {:?}",
                other
            ))),
            None => Err(VMError::StackUnderflow),
        }
    }
} 