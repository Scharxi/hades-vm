use std::collections::{HashMap, LinkedList};
use std::fmt;
use std::sync::Arc;

use crate::{
    basic_block::BasicBlock,
    context::Context,
    error::{Error, Result},
    types::Type,
    value::{Value, ValueId},
};

/// A function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    /// The value ID of this parameter
    id: ValueId,
    /// The name of the parameter (optional)
    name: Option<String>,
    /// The type of the parameter
    ty: Arc<Type>,
    /// The index of the parameter
    index: usize,
}

impl Parameter {
    /// Create a new parameter
    pub fn new(ty: Arc<Type>, index: usize, name: Option<String>) -> Self {
        Self {
            id: ValueId::new(),
            name,
            ty,
            index,
        }
    }
    
    /// Get the ID of this parameter
    pub fn id(&self) -> ValueId {
        self.id
    }
    
    /// Get the name of this parameter
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    
    /// Get the type of this parameter
    pub fn ty(&self) -> Arc<Type> {
        self.ty.clone()
    }
    
    /// Get the index of this parameter
    pub fn index(&self) -> usize {
        self.index
    }
    
    /// Convert this parameter to a value
    pub fn to_value(&self) -> Value {
        Value::Argument {
            id: self.id,
            index: self.index,
            name: self.name.clone(),
            ty: self.ty.clone(),
        }
    }
}

/// Linkage type for a function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Linkage {
    /// Function is visible outside of the module and can be linked against
    External,
    /// Function is only visible within the module
    Internal,
    /// Function is visible outside of the module but will be inlined into the calling code
    InlineOnly,
    /// Function is visible outside of the module, but can be optimized based on its usage
    LinkOnceODR,
}

impl Default for Linkage {
    fn default() -> Self {
        Self::External
    }
}

/// Function represents a function in the IR.
#[derive(Debug, Clone)]
pub struct Function {
    /// The ID of this function as a value
    id: ValueId,
    /// The name of this function
    name: String,
    /// The type of this function
    ty: Arc<Type>,
    /// The linkage type of this function
    linkage: Linkage,
    /// The parameters of this function
    parameters: Vec<Parameter>,
    /// The basic blocks in this function
    basic_blocks: LinkedList<BasicBlock>,
    /// The entry basic block of this function
    entry_block: Option<ValueId>,
    /// Metadata associated with this function
    metadata: HashMap<String, String>,
}

impl Function {
    /// Create a new function with the given name and type
    pub fn new(name: String, ty: Arc<Type>, linkage: Linkage) -> Result<Self> {
        // Clone ty before the pattern match to avoid borrowing issues
        let ty_ref = ty.clone();
        
        // Validate that the type is a function type
        if let Type::Function { return_type, param_types, .. } = &*ty_ref {
            Ok(Self {
                id: ValueId::new(),
                name,
                ty,
                linkage,
                parameters: Vec::with_capacity(param_types.len()),
                basic_blocks: LinkedList::new(),
                entry_block: None,
                metadata: HashMap::new(),
            })
        } else {
            Err(Error::ConstructionError(format!(
                "Expected function type, got {}",
                ty
            )))
        }
    }
    
    /// Get the ID of this function
    pub fn id(&self) -> ValueId {
        self.id
    }
    
    /// Get the name of this function
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the type of this function
    pub fn ty(&self) -> Arc<Type> {
        self.ty.clone()
    }
    
    /// Get the return type of this function
    pub fn return_type(&self) -> Arc<Type> {
        if let Type::Function { return_type, .. } = &*self.ty {
            return_type.clone()
        } else {
            unreachable!("Function type was validated during construction")
        }
    }
    
    /// Get the linkage type of this function
    pub fn linkage(&self) -> Linkage {
        self.linkage
    }
    
    /// Set the linkage type of this function
    pub fn set_linkage(&mut self, linkage: Linkage) {
        self.linkage = linkage;
    }
    
    /// Get the parameters of this function
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }
    
    /// Add a parameter to this function
    pub fn add_parameter(&mut self, name: Option<String>, ty: Arc<Type>) -> Result<Parameter> {
        // Validate against the function type
        if let Type::Function { param_types, .. } = &*self.ty {
            let index = self.parameters.len();
            if index >= param_types.len() {
                return Err(Error::ConstructionError(format!(
                    "Too many parameters added to function {}",
                    self.name
                )));
            }
            
            if *param_types[index] != *ty {
                return Err(Error::TypeError(format!(
                    "Parameter type mismatch: expected {}, got {}",
                    param_types[index], ty
                )));
            }
            
            let param = Parameter::new(ty, index, name);
            self.parameters.push(param.clone());
            Ok(param)
        } else {
            unreachable!("Function type was validated during construction")
        }
    }
    
    /// Get the basic blocks in this function
    pub fn basic_blocks(&self) -> &LinkedList<BasicBlock> {
        &self.basic_blocks
    }
    
    /// Get mutable basic blocks in this function
    pub fn basic_blocks_mut(&mut self) -> &mut LinkedList<BasicBlock> {
        &mut self.basic_blocks
    }
    
    /// Add a basic block to this function
    pub fn add_basic_block(&mut self, basic_block: BasicBlock) -> ValueId {
        let id = basic_block.id();
        
        // If this is the first block, make it the entry block
        if self.basic_blocks.is_empty() {
            self.entry_block = Some(id);
        }
        
        self.basic_blocks.push_back(basic_block);
        id
    }
    
    /// Create and add a new basic block to this function
    pub fn create_basic_block(&mut self, name: Option<String>) -> BasicBlock {
        let block = BasicBlock::new(name);
        let id = block.id();
        
        // If this is the first block, make it the entry block
        if self.basic_blocks.is_empty() {
            self.entry_block = Some(id);
        }
        
        self.basic_blocks.push_back(block.clone());
        block
    }
    
    /// Get the entry basic block of this function
    pub fn entry_block(&self) -> Option<&BasicBlock> {
        if let Some(entry_id) = self.entry_block {
            self.basic_blocks.iter().find(|bb| bb.id() == entry_id)
        } else {
            None
        }
    }
    
    /// Get a mutable reference to the entry basic block of this function
    pub fn entry_block_mut(&mut self) -> Option<&mut BasicBlock> {
        if let Some(entry_id) = self.entry_block {
            self.basic_blocks.iter_mut().find(|bb| bb.id() == entry_id)
        } else {
            None
        }
    }
    
    /// Set the entry basic block of this function
    pub fn set_entry_block(&mut self, id: ValueId) -> Result<()> {
        // Verify that the block exists in this function
        if !self.basic_blocks.iter().any(|bb| bb.id() == id) {
            return Err(Error::ConstructionError(format!(
                "Basic block with ID {} not found in function {}",
                id, self.name
            )));
        }
        
        self.entry_block = Some(id);
        Ok(())
    }
    
    /// Add metadata to this function
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Get metadata from this function
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Convert this function to a value
    pub fn to_value(&self) -> Value {
        Value::Function {
            id: self.id,
            name: self.name.clone(),
            ty: self.ty.clone(),
        }
    }
    
    /// Verify that the function is well-formed
    pub fn verify(&self) -> Result<()> {
        // Check that all parameters have been added
        if let Type::Function { param_types, .. } = &*self.ty {
            if self.parameters.len() != param_types.len() {
                return Err(Error::ValidationError(format!(
                    "Function {} has {} parameters, but its type requires {}",
                    self.name, self.parameters.len(), param_types.len()
                )));
            }
        }
        
        // Check that the function has an entry block
        if self.entry_block.is_none() {
            return Err(Error::ValidationError(format!(
                "Function {} has no entry block",
                self.name
            )));
        }
        
        // Check that all blocks except the last one have terminators
        for block in self.basic_blocks.iter() {
            if !block.has_terminator() && block.id() != self.basic_blocks.back().unwrap().id() {
                return Err(Error::ValidationError(format!(
                    "Basic block {} in function {} has no terminator instruction",
                    block.id(), self.name
                )));
            }
        }
        
        Ok(())
    }
    
    /// Find a basic block by ID
    pub fn find_basic_block(&self, id: ValueId) -> Option<&BasicBlock> {
        self.basic_blocks.iter().find(|bb| bb.id() == id)
    }
    
    /// Find a mutable basic block by ID
    pub fn find_basic_block_mut(&mut self, id: ValueId) -> Option<&mut BasicBlock> {
        self.basic_blocks.iter_mut().find(|bb| bb.id() == id)
    }
    
    /// Find a basic block by name
    pub fn find_basic_block_by_name(&self, name: &str) -> Option<&BasicBlock> {
        self.basic_blocks.iter().find(|bb| bb.name() == Some(name))
    }
    
    /// Find a mutable basic block by name
    pub fn find_basic_block_by_name_mut(&mut self, name: &str) -> Option<&mut BasicBlock> {
        self.basic_blocks.iter_mut().find(|bb| bb.name() == Some(name))
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write function signature
        write!(f, "define ")?;
        
        // Write linkage
        match self.linkage {
            Linkage::External => {} // Default, no prefix needed
            Linkage::Internal => write!(f, "internal ")?,
            Linkage::InlineOnly => write!(f, "inlinehint ")?,
            Linkage::LinkOnceODR => write!(f, "linkonce_odr ")?,
        }
        
        // Write return type and name
        if let Type::Function { return_type, .. } = &*self.ty {
            write!(f, "{} @{}(", return_type, self.name)?;
        } else {
            unreachable!("Function type was validated during construction");
        }
        
        // Write parameters
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", param.ty())?;
            if let Some(name) = param.name() {
                write!(f, " %{}", name)?;
            } else {
                write!(f, " %{}", param.id())?;
            }
        }
        
        // End signature and start body
        writeln!(f, ") {{")?;
        
        // Write basic blocks
        for block in &self.basic_blocks {
            writeln!(f, "{}", block)?;
        }
        
        // End function body
        writeln!(f, "}}")?;
        
        Ok(())
    }
} 