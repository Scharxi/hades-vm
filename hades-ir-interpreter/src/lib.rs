use std::collections::HashMap;
use thiserror::Error;
use hades_ir::{
    instruction::{Instruction as IRInstruction, Operation},
    value::Value,
    types::Type,
};
use vm::{
    cpu::CPU,
    instruction::{RawInstruction, IntoRaw, Instruction as VMInstruction},
    memory::{SegmentedMemory, MemoryRegionType},
    stack::{Stack, StackValue},
    opcode::Opcode,
    instruction_processor::{ExecutionSignal, DecodeError},
};

#[derive(Error, Debug)]
pub enum InterpreterError {
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch {
        expected: String,
        got: String,
    },
    
    #[error("Unknown value: {0}")]
    UnknownValue(String),
    
    #[error("VM error: {0}")]
    VMError(String),

    #[error("VM execution error: {0}")]
    VMExecutionError(String),
    
    #[error("Memory error: {0}")]
    MemoryError(String),
}

impl From<String> for InterpreterError {
    fn from(error: String) -> Self {
        InterpreterError::MemoryError(error)
    }
}

pub type Result<T> = std::result::Result<T, InterpreterError>;

/// The IR interpreter translates IR instructions to VM bytecode and executes them.
pub struct IRInterpreter {
    /// The VM CPU that executes the bytecode
    cpu: CPU,
    /// Maps IR values to their locations in VM memory
    value_locations: HashMap<String, usize>,
    /// The next available memory location for storing values
    next_location: usize,
}

impl IRInterpreter {
    /// Create a new IR interpreter
    pub fn new() -> Self {
        let mut cpu = CPU::new(1024 * 1024); // 1MB memory
        cpu.load_program(&[]); // Load empty program to initialize the CPU
        
        Self {
            cpu,
            value_locations: HashMap::new(),
            next_location: 0,
        }
    }

    /// Execute a single IR instruction
    pub fn execute_instruction(&mut self, instruction: &IRInstruction) -> Result<()> {
        match instruction.operation() {
            // Arithmetic operations
            Operation::Add => self.execute_binary_op(instruction, Opcode::Add)?,
            Operation::Sub => self.execute_binary_op(instruction, Opcode::Sub)?,
            Operation::Mul => self.execute_binary_op(instruction, Opcode::Multiply)?,
            Operation::Div => self.execute_binary_op(instruction, Opcode::Divide)?,
            
            // String operations
            Operation::StringConcat => self.execute_string_concat(instruction)?,
            Operation::StringLength => self.execute_string_length(instruction)?,
            Operation::StringSubstring => self.execute_string_substring(instruction)?,
            Operation::StringCompare => self.execute_string_compare(instruction)?,
            Operation::StringContains => self.execute_string_contains(instruction)?,
            
            // Other operations
            Operation::Print => self.execute_print(instruction)?,
            
            // TODO: Implement other operations
            _ => todo!("Operation {:?} not yet implemented", instruction.operation()),
        }
        
        Ok(())
    }

    /// Execute a binary operation
    fn execute_binary_op(&mut self, instruction: &IRInstruction, opcode: Opcode) -> Result<()> {
        let operands = instruction.operands();
        if operands.len() != 2 {
            return Err(InterpreterError::VMError(
                format!("Binary operation requires 2 operands, got {}", operands.len())
            ));
        }

        // Load operands onto the stack
        self.load_value(&operands[0])?;
        self.load_value(&operands[1])?;

        // Execute the operation
        let raw = RawInstruction::from_bytes(0, 0, 0, opcode as u8);
        self.execute_vm_instruction(raw)
    }

    /// Execute a VM instruction and handle errors
    fn execute_vm_instruction(&mut self, raw: RawInstruction) -> Result<()> {
        let vm_instruction = VMInstruction::try_from(raw)
            .map_err(|e| InterpreterError::VMError(format!("Failed to decode instruction: {:?}", e)))?;
        
        let signal = self.cpu.executor.execute(&vm_instruction, &mut self.cpu.stack, &mut self.cpu.memory, 0);
        match signal {
            ExecutionSignal::Continue => Ok(()),
            _ => Err(InterpreterError::VMExecutionError(format!("Unexpected execution signal: {:?}", signal))),
        }
    }

    /// Execute a string operation
    fn execute_string_op(&mut self, instruction: &IRInstruction, opcode: Opcode, expected_operands: usize) -> Result<()> {
        let operands = instruction.operands();
        if operands.len() != expected_operands {
            return Err(InterpreterError::VMError(
                format!("String operation requires {} operands, got {}", expected_operands, operands.len())
            ));
        }

        // Load operands onto the stack
        for operand in operands {
            self.load_value(operand)?;
        }

        // Execute the operation
        let raw = RawInstruction::from_bytes(0, 0, 0, opcode as u8);
        self.execute_vm_instruction(raw)
    }

    /// Execute a string concatenation operation
    fn execute_string_concat(&mut self, instruction: &IRInstruction) -> Result<()> {
        self.execute_string_op(instruction, Opcode::StringConcat, 2)
    }

    /// Execute a string length operation
    fn execute_string_length(&mut self, instruction: &IRInstruction) -> Result<()> {
        self.execute_string_op(instruction, Opcode::StringLength, 1)
    }

    /// Execute a string substring operation
    fn execute_string_substring(&mut self, instruction: &IRInstruction) -> Result<()> {
        let operands = instruction.operands();
        if operands.len() != 3 {
            return Err(InterpreterError::VMError(
                format!("String substring operation requires 3 operands, got {}", operands.len())
            ));
        }

        // Extract start and length values
        let start = match &operands[1] {
            Value::IntegerConstant { value, .. } => *value as i32,
            _ => return Err(InterpreterError::TypeMismatch {
                expected: "integer".to_string(),
                got: format!("{:?}", operands[1]),
            }),
        };

        let length = match &operands[2] {
            Value::IntegerConstant { value, .. } => *value as i32,
            _ => return Err(InterpreterError::TypeMismatch {
                expected: "integer".to_string(),
                got: format!("{:?}", operands[2]),
            }),
        };

        // Load the string operand onto the stack
        self.load_value(&operands[0])?; // string

        // Create the VM instruction with both operands in the vector
        let vm_instruction = VMInstruction {
            opcode: Opcode::StringSubstring,
            operands: vec![start, length],
        };
        
        let signal = self.cpu.executor.execute(&vm_instruction, &mut self.cpu.stack, &mut self.cpu.memory, 0);
        match signal {
            ExecutionSignal::Continue => Ok(()),
            _ => Err(InterpreterError::VMExecutionError(format!("Unexpected execution signal: {:?}", signal))),
        }
    }

    /// Execute a string compare operation
    fn execute_string_compare(&mut self, instruction: &IRInstruction) -> Result<()> {
        self.execute_string_op(instruction, Opcode::StringCompare, 2)
    }

    /// Execute a string contains operation
    fn execute_string_contains(&mut self, instruction: &IRInstruction) -> Result<()> {
        self.execute_string_op(instruction, Opcode::StringContains, 2)
    }

    /// Execute a print operation
    fn execute_print(&mut self, instruction: &IRInstruction) -> Result<()> {
        let operands = instruction.operands();
        if operands.len() != 1 {
            return Err(InterpreterError::VMError(
                format!("Print operation requires 1 operand, got {}", operands.len())
            ));
        }

        // Load the value to print onto the stack
        self.load_value(&operands[0])?;

        // Get the value from the stack
        if let Some(value) = self.cpu.stack.pop() {
            // Print the value
            match value {
                StackValue::Integer(i) => println!("{}", i),
                StackValue::Float(f) => println!("{}", f),
                StackValue::Boolean(b) => println!("{}", b),
                StackValue::String(s) => println!("{}", s),
                StackValue::Reference(r) => println!("ref({})", r),
            }
            Ok(())
        } else {
            Err(InterpreterError::VMError("Stack underflow".to_string()))
        }
    }

    /// Load a value onto the stack
    fn load_value(&mut self, value: &Value) -> Result<()> {
        match value {
            Value::IntegerConstant { value, .. } => {
                self.cpu.stack.push(StackValue::Integer(*value as i32));
                Ok(())
            },
            Value::FloatConstant { value, .. } => {
                self.cpu.stack.push(StackValue::Float(*value as f32));
                Ok(())
            },
            Value::BooleanConstant { value } => {
                self.cpu.stack.push(StackValue::Boolean(*value));
                Ok(())
            },
            Value::StringConstant { value } => {
                // For string constants, we directly create a StackValue::String
                self.cpu.stack.push(StackValue::String(value.clone()));
                Ok(())
            },
            Value::GlobalVariable { name, .. } => {
                if let Some(&addr) = self.value_locations.get(name) {
                    let value = self.cpu.memory.read_from_region(MemoryRegionType::Data, addr)?;
                    self.cpu.stack.push(StackValue::Integer(value));
                    Ok(())
                } else {
                    Err(InterpreterError::UnknownValue(name.clone()))
                }
            },
            Value::NullPointer { .. } => {
                self.cpu.stack.push(StackValue::Reference(0));
                Ok(())
            },
            Value::Instruction { id, .. } => {
                // For instruction values, we need to get the result from the stack
                // For now, we'll just peek at the top of the stack since that's where the result should be
                if let Some(value) = self.cpu.stack.peek() {
                    self.cpu.stack.push(value.clone());
                    Ok(())
                } else {
                    Err(InterpreterError::VMError("No value on stack for instruction result".to_string()))
                }
            },
            _ => Err(InterpreterError::TypeMismatch {
                expected: "loadable value".to_string(),
                got: format!("{:?}", value),
            }),
        }
    }

    /// Store a value from the stack into memory
    fn store_value(&mut self, name: &str, value_type: &Type) -> Result<()> {
        let addr = self.next_location;
        self.next_location += match value_type {
            Type::Integer(_) | Type::Float(_) | Type::Boolean => 8,
            Type::String => {
                let value = self.cpu.stack.pop()
                    .ok_or_else(|| InterpreterError::VMError("Stack underflow".to_string()))?;
                let addr = value.as_reference();
                let mut len = 0;
                loop {
                    let byte = self.cpu.memory.read_from_region(MemoryRegionType::Data, addr + len)?;
                    if byte == 0 {
                        break;
                    }
                    len += 1;
                }
                len + 1 // Include null terminator
            },
            Type::Array { element_type, size } => size * element_type.size_in_bytes().unwrap_or(8),
            Type::Pointer(_) => 8,
            _ => return Err(InterpreterError::TypeMismatch {
                expected: "storable type".to_string(),
                got: format!("{:?}", value_type),
            }),
        };

        let value = self.cpu.stack.pop()
            .ok_or_else(|| InterpreterError::VMError("Stack underflow".to_string()))?;
        
        match value {
            StackValue::Integer(i) => {
                self.cpu.memory.write_to_region(MemoryRegionType::Data, addr, i)?;
            },
            StackValue::Float(f) => {
                self.cpu.memory.write_to_region(MemoryRegionType::Data, addr, f as i32)?;
            },
            StackValue::Boolean(b) => {
                self.cpu.memory.write_to_region(MemoryRegionType::Data, addr, b as i32)?;
            },
            StackValue::Reference(r) => {
                self.cpu.memory.write_to_region(MemoryRegionType::Data, addr, r as i32)?;
            },
            StackValue::String(_) => {
                return Err(InterpreterError::TypeMismatch {
                    expected: "primitive value".to_string(),
                    got: "string".to_string(),
                });
            }
        }
        
        self.value_locations.insert(name.to_string(), addr);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_string_operations() {
        let mut interpreter = IRInterpreter::new();

        // Test string concatenation
        let hello = Value::string_constant("Hello ");
        let world = Value::string_constant("World");
        let concat = IRInstruction::string_concat(hello, world);
        interpreter.execute_instruction(&concat).unwrap();

        // Test string length
        let str = Value::string_constant("Hello World");
        let length = IRInstruction::string_length(str);
        interpreter.execute_instruction(&length).unwrap();

        // Test string substring
        let str = Value::string_constant("Hello World");
        let start = Value::integer_constant(0, Arc::new(Type::Integer(32)));
        let len = Value::integer_constant(5, Arc::new(Type::Integer(32)));
        let substring = IRInstruction::string_substring(str, start, len);
        interpreter.execute_instruction(&substring).unwrap();
    }
}
