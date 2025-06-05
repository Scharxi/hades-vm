use std::collections::HashMap;
use crate::{
    error::{Error, Result},
    executable::{HadesExecutable, SectionType},
    bytecode::BytecodeGenerator,
    module::Module,
    function::Function,
    value::Value,
};

/// Generator for complete Hades Executable (HEX) files
pub struct ExecutableGenerator {
    /// The executable being built
    executable: HadesExecutable,
    /// Constant pool for all constants used in the program
    constant_pool: Vec<Value>,
    /// Maps Values to their indices in the constant pool
    constant_indices: HashMap<Value, u32>,
    /// Current virtual address for allocation
    current_address: u32,
    /// Entry point function name
    entry_function: Option<String>,
}

impl ExecutableGenerator {
    /// Create a new executable generator
    pub fn new() -> Self {
        Self {
            executable: HadesExecutable::new(0), // Entry point will be set later
            constant_pool: Vec::new(),
            constant_indices: HashMap::new(),
            current_address: 0x1000, // Start at 4KB mark
            entry_function: None,
        }
    }

    /// Set the entry function name
    pub fn set_entry_function(&mut self, name: String) {
        self.entry_function = Some(name);
    }

    /// Add a constant to the constant pool and return its index
    fn add_constant(&mut self, value: Value) -> u32 {
        if let Some(&index) = self.constant_indices.get(&value) {
            return index;
        }

        let index = self.constant_pool.len() as u32;
        self.constant_pool.push(value.clone());
        self.constant_indices.insert(value, index);
        index
    }

    /// Generate executable from an IR module
    pub fn generate_from_module(&mut self, module: &Module) -> Result<()> {
        // First pass: collect all constants and generate bytecode for functions
        let mut function_bytecode: HashMap<String, Vec<u8>> = HashMap::new();
        let mut function_addresses: HashMap<String, u32> = HashMap::new();
        
        for (function_name, function) in module.functions() {
            let bytecode = self.generate_function_bytecode(function)?;
            function_addresses.insert(function_name.clone(), self.current_address);
            
            // Update current address for next function
            self.current_address += bytecode.len() as u32;
            // Align to 4-byte boundary
            self.current_address = (self.current_address + 3) & !3;
            
            function_bytecode.insert(function_name.clone(), bytecode);
        }

        // Set entry point if specified
        if let Some(ref entry_name) = self.entry_function {
            if let Some(&entry_addr) = function_addresses.get(entry_name) {
                self.executable.header.entry_point = entry_addr;
            } else {
                return Err(Error::ExecutableError(format!("Entry function '{}' not found", entry_name)));
            }
        } else if let Some(main_addr) = function_addresses.get("main") {
            // Default to main function if it exists
            self.executable.header.entry_point = *main_addr;
        }

        // Create constants section if we have constants
        if !self.constant_pool.is_empty() {
            let constants_data = self.serialize_constant_pool()?;
            self.executable.add_section(".constants", SectionType::Constants, constants_data);
        }

        // Create code section with all function bytecode
        let mut code_data = Vec::new();
        let mut current_offset = 0u32;
        
        for (function_name, function) in module.functions() {
            if let Some(bytecode) = function_bytecode.get(function_name) {
                // Add function symbol
                self.executable.add_symbol(
                    function_name,
                    current_offset,
                    bytecode.len() as u32,
                    0, // Code section index (will be 0 if constants section exists, 1 otherwise)
                );
                
                code_data.extend_from_slice(bytecode);
                current_offset += bytecode.len() as u32;
                
                // Align to 4-byte boundary
                while current_offset % 4 != 0 {
                    code_data.push(0);
                    current_offset += 1;
                }
            }
        }

        // Add code section
        if !code_data.is_empty() {
            self.executable.add_section(".text", SectionType::Code, code_data);
        }

        Ok(())
    }

    /// Generate bytecode for a single function
    fn generate_function_bytecode(&mut self, function: &Function) -> Result<Vec<u8>> {
        let mut generator = BytecodeGenerator::new();
        
        // Generate function prologue
        // CreateFrame with parameter count
        let param_count = function.parameters().len() as u32;
        self.emit_instruction(&mut generator, &[0x00, 0x00, param_count as u8, 0x40])?;
        
        // Generate code for each basic block
        for basic_block in function.basic_blocks() {
            for instruction in basic_block.instructions() {
                self.generate_instruction_bytecode(&mut generator, instruction)?;
            }
        }
        
        // Ensure we have a return instruction at the end
        // if the function doesn't end with one
        let bytecode = generator.get_bytecode();
        if bytecode.is_empty() || bytecode[bytecode.len() - 1] != 0x0E {
            // Add return instruction
            self.emit_instruction(&mut generator, &[0x00, 0x00, 0x00, 0x0E])?;
        }

        Ok(generator.get_bytecode().to_vec())
    }

    /// Generate bytecode for a single IR instruction
    fn generate_instruction_bytecode(&mut self, generator: &mut BytecodeGenerator, instruction: &crate::instruction::Instruction) -> Result<()> {
        use crate::instruction::Operation;
        
        match instruction.operation() {
            Operation::Add => {
                // Operands should already be on stack
                self.emit_instruction(generator, &[0x00, 0x00, 0x00, 0x01])?; // Add
            },
            
            Operation::Sub => {
                self.emit_instruction(generator, &[0x00, 0x00, 0x00, 0x03])?; // Sub
            },
            
            Operation::Mul => {
                self.emit_instruction(generator, &[0x00, 0x00, 0x00, 0x05])?; // Multiply
            },
            
            Operation::Div => {
                self.emit_instruction(generator, &[0x00, 0x00, 0x00, 0x0A])?; // Divide
            },
            
            Operation::Load => {
                // For now, assume we're loading from a constant or local
                if let Some(operand) = instruction.operands().first() {
                    self.generate_value_load(generator, operand)?;
                }
            },
            
            Operation::Store => {
                // Store top stack value to memory location
                self.emit_instruction(generator, &[0x00, 0x00, 0x00, 0x08])?; // StoreMemory
            },
            
            Operation::Ret => {
                if instruction.operands().is_empty() {
                    // Return void
                    self.emit_instruction(generator, &[0x00, 0x00, 0x00, 0x0E])?; // Return
                } else {
                    // Load return value first
                    if let Some(operand) = instruction.operands().first() {
                        self.generate_value_load(generator, operand)?;
                    }
                    self.emit_instruction(generator, &[0x00, 0x00, 0x00, 0x0E])?; // Return
                }
            },
            
            Operation::Call => {
                // For function calls, we need the function address
                // This is simplified - in practice we'd need symbol resolution
                self.emit_instruction(generator, &[0x00, 0x00, 0x00, 0x0D])?; // Call
            },
            
            _ => {
                return Err(Error::BytecodeGenerationError(format!(
                    "Unsupported instruction operation: {:?}", 
                    instruction.operation()
                )));
            }
        }
        
        Ok(())
    }

    /// Generate code to load a value onto the stack
    fn generate_value_load(&mut self, generator: &mut BytecodeGenerator, value: &Value) -> Result<()> {
        match value {
            Value::IntegerConstant { value, .. } => {
                // LoadConstant instruction
                let value_bytes = (*value as u32).to_be_bytes();
                self.emit_instruction(generator, &[value_bytes[1], value_bytes[2], value_bytes[3], 0x04])?;
            },
            
            Value::FloatConstant { value, .. } => {
                // For floats, we'd need to store them in the constant pool
                // and load by index. For now, convert to int
                let int_value = *value as i64;
                let value_bytes = (int_value as u32).to_be_bytes();
                self.emit_instruction(generator, &[value_bytes[1], value_bytes[2], value_bytes[3], 0x04])?;
            },
            
            Value::BooleanConstant { value } => {
                let int_value = if *value { 1u32 } else { 0u32 };
                let value_bytes = int_value.to_be_bytes();
                self.emit_instruction(generator, &[value_bytes[1], value_bytes[2], value_bytes[3], 0x04])?;
            },
            
            Value::StringConstant { value } => {
                // Add string to constant pool and load by index
                let string_value = Value::StringConstant { value: value.clone() };
                let index = self.add_constant(string_value);
                let index_bytes = index.to_be_bytes();
                // Use a different opcode for string constants (hypothetical 0x70)
                self.emit_instruction(generator, &[index_bytes[1], index_bytes[2], index_bytes[3], 0x70])?;
            },
            
            _ => {
                return Err(Error::BytecodeGenerationError(format!(
                    "Unsupported value type for loading: {:?}", 
                    value
                )));
            }
        }
        
        Ok(())
    }

    /// Emit a raw instruction to the bytecode generator
    fn emit_instruction(&self, generator: &mut BytecodeGenerator, bytes: &[u8]) -> Result<()> {
        if bytes.len() != 4 {
            return Err(Error::BytecodeGenerationError(format!(
                "Instructions must be exactly 4 bytes, got {}", 
                bytes.len()
            )));
        }
        
        // Use the new emit_raw_bytes method
        generator.emit_raw_bytes(bytes)?;
        
        Ok(())
    }

    /// Serialize the constant pool to bytes
    fn serialize_constant_pool(&self) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        
        // Write constant count
        let count = self.constant_pool.len() as u32;
        data.extend_from_slice(&count.to_le_bytes());
        
        // Write each constant
        for constant in &self.constant_pool {
            match constant {
                Value::IntegerConstant { value, ty } => {
                    data.push(1); // Type tag for integer
                    data.extend_from_slice(&(*value as u64).to_le_bytes());
                    // Store type information if needed
                },
                
                Value::FloatConstant { value, ty } => {
                    data.push(2); // Type tag for float
                    data.extend_from_slice(&value.to_le_bytes());
                },
                
                Value::BooleanConstant { value } => {
                    data.push(3); // Type tag for boolean
                    data.push(if *value { 1 } else { 0 });
                },
                
                Value::StringConstant { value } => {
                    data.push(4); // Type tag for string
                    let len = value.len() as u32;
                    data.extend_from_slice(&len.to_le_bytes());
                    data.extend_from_slice(value.as_bytes());
                },
                
                _ => {
                    return Err(Error::BytecodeGenerationError(format!(
                        "Unsupported constant type: {:?}", 
                        constant
                    )));
                }
            }
        }
        
        Ok(data)
    }

    /// Get the generated executable
    pub fn get_executable(&self) -> &HadesExecutable {
        &self.executable
    }

    /// Take ownership of the generated executable
    pub fn into_executable(self) -> HadesExecutable {
        self.executable
    }

    /// Write the executable to a buffer
    pub fn write_to_bytes(&mut self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.executable.write_to(&mut buffer)?;
        Ok(buffer)
    }
}

impl Default for ExecutableGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, Module, Function, Type as IRType, Linkage};
    use std::sync::Arc;

    #[test]
    fn test_executable_generation() {
        let mut generator = ExecutableGenerator::new();
        
        // Create a simple module with a main function
        let context = Arc::new(Context::new());
        let mut module = Module::new(context.clone(), "test".to_string());
        
        // Create main function
        let main_type = IRType::function(IRType::i32(), vec![], false);
        let main_function = Function::new("main".to_string(), main_type, Linkage::External).unwrap();
        
        module.add_function(main_function).unwrap();
        
        // Generate executable
        generator.set_entry_function("main".to_string());
        generator.generate_from_module(&module).unwrap();
        
        let executable = generator.get_executable();
        assert_eq!(executable.header.magic, crate::executable::HEX_MAGIC);
        assert!(executable.sections.len() > 0);
    }
} 