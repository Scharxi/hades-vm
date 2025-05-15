use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::{
    instruction::Instruction,
    value::{Value, ValueId},
};

/// A BasicBlock is a sequence of instructions with a single entry and exit point.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// The ID of this basic block as a value
    id: ValueId,
    /// The name of this basic block (optional)
    name: Option<String>,
    /// The instructions in this basic block
    instructions: Vec<Instruction>,
    /// Metadata associated with this basic block
    metadata: HashMap<String, String>,
}

impl BasicBlock {
    /// Create a new basic block with the given name
    pub fn new(name: Option<String>) -> Self {
        Self {
            id: ValueId::new(),
            name,
            instructions: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new basic block with the given name (str version)
    pub fn named(name: &str) -> Self {
        Self::new(Some(name.to_string()))
    }
    
    /// Create a new unnamed basic block
    pub fn unnamed() -> Self {
        Self::new(None)
    }
    
    /// Get the ID of this basic block
    pub fn id(&self) -> ValueId {
        self.id
    }
    
    /// Get the name of this basic block
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    
    /// Set the name of this basic block
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }
    
    /// Get the instructions in this basic block
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
    
    /// Get mutable instructions in this basic block
    pub fn instructions_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.instructions
    }
    
    /// Add an instruction to this basic block
    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
    
    /// Add multiple instructions to this basic block
    pub fn add_instructions(&mut self, instructions: Vec<Instruction>) {
        self.instructions.extend(instructions);
    }
    
    /// Get the last instruction in this basic block
    pub fn get_terminator(&self) -> Option<&Instruction> {
        self.instructions.last()
    }
    
    /// Check if this basic block has a terminator instruction
    pub fn has_terminator(&self) -> bool {
        if let Some(inst) = self.get_terminator() {
            matches!(
                inst.operation(),
                crate::instruction::Operation::Ret
                    | crate::instruction::Operation::Br
                    | crate::instruction::Operation::CondBr
                    | crate::instruction::Operation::Switch
            )
        } else {
            false
        }
    }
    
    /// Add metadata to this basic block
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Get metadata from this basic block
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Convert this basic block to a value
    pub fn to_value(&self) -> Value {
        Value::BasicBlock {
            id: self.id,
            name: self.name.clone(),
        }
    }
    
    /// Get an instruction by index
    pub fn get_instruction(&self, index: usize) -> Option<&Instruction> {
        self.instructions.get(index)
    }
    
    /// Get a mutable instruction by index
    pub fn get_instruction_mut(&mut self, index: usize) -> Option<&mut Instruction> {
        self.instructions.get_mut(index)
    }
    
    /// Remove an instruction by index
    pub fn remove_instruction(&mut self, index: usize) -> Option<Instruction> {
        if index < self.instructions.len() {
            Some(self.instructions.remove(index))
        } else {
            None
        }
    }
    
    /// Check if this basic block is empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
    
    /// Get the number of instructions in this basic block
    pub fn len(&self) -> usize {
        self.instructions.len()
    }
    
    /// Get an iterator over the instructions in this basic block
    pub fn iter(&self) -> impl Iterator<Item = &Instruction> {
        self.instructions.iter()
    }
    
    /// Get a mutable iterator over the instructions in this basic block
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Instruction> {
        self.instructions.iter_mut()
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}:", name)?;
        } else {
            write!(f, "{}:", self.id)?;
        }
        
        if !self.instructions.is_empty() {
            writeln!(f)?;
            for instruction in &self.instructions {
                writeln!(f, "  {}", instruction)?;
            }
        }
        
        Ok(())
    }
} 