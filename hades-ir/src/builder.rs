use std::sync::Arc;
use std::cell::RefCell;

use crate::{
    basic_block::BasicBlock,
    context::Context,
    error::{Error, Result},
    function::Function,
    instruction::{Instruction, Operation},
    types::Type,
    value::Value,
};

/// The Builder provides a convenient interface for constructing IR.
#[derive(Debug, Clone)]
pub struct Builder {
    /// The context this builder belongs to
    context: Arc<Context>,
    /// The current function being built
    current_function: Option<Arc<RefCell<Function>>>,
    /// The current basic block being built
    current_block: Option<Arc<RefCell<BasicBlock>>>,
}

impl Builder {
    /// Create a new builder with the given context
    pub fn new(context: Arc<Context>) -> Self {
        Self {
            context,
            current_function: None,
            current_block: None,
        }
    }
    
    /// Get the context this builder belongs to
    pub fn context(&self) -> Arc<Context> {
        self.context.clone()
    }
    
    /// Set the current function
    pub fn set_function(&mut self, function: Arc<RefCell<Function>>) {
        self.current_function = Some(function);
        self.current_block = None;
    }
    
    /// Get the current function
    pub fn current_function(&self) -> Option<Arc<RefCell<Function>>> {
        self.current_function.clone()
    }
    
    /// Set the current basic block
    pub fn set_insertion_point(&mut self, block: Arc<RefCell<BasicBlock>>) {
        self.current_block = Some(block);
    }
    
    /// Get the current basic block
    pub fn current_block(&self) -> Option<Arc<RefCell<BasicBlock>>> {
        self.current_block.clone()
    }
    
    /// Create a new basic block and append it to the current function
    pub fn create_block(&mut self, name: Option<String>) -> Result<Arc<RefCell<BasicBlock>>> {
        if let Some(function) = &self.current_function {
            let block = BasicBlock::new(name);
            let block_id = block.id();
            
            // Add to the function
            {
                let mut function = function.borrow_mut();
                function.add_basic_block(block.clone());
            }
            
            // Create a reference cell for the block
            let block_ref = Arc::new(RefCell::new(block));
            
            Ok(block_ref)
        } else {
            Err(Error::ConstructionError(
                "No current function set for builder".to_string()
            ))
        }
    }
    
    /// Set the insertion point to the end of the given block
    pub fn position_at_end(&mut self, block: Arc<RefCell<BasicBlock>>) {
        self.current_block = Some(block);
    }
    
    /// Build an instruction and insert it at the current insertion point
    fn build_instruction(&mut self, instruction: Instruction) -> Result<Value> {
        if let Some(block) = &self.current_block {
            // Clone the instruction before adding metadata
            let result_type = instruction.result_type();
            let instruction_id = instruction.id();
            
            // Insert the instruction
            {
                let mut block = block.borrow_mut();
                block.add_instruction(instruction);
            }
            
            if let Some(ty) = result_type {
                Ok(Value::Instruction {
                    id: instruction_id,
                    ty,
                })
            } else {
                // Void instructions do not produce a value
                Err(Error::ConstructionError(
                    "Instruction does not produce a value".to_string()
                ))
            }
        } else {
            Err(Error::ConstructionError(
                "No insertion point set for builder".to_string()
            ))
        }
    }
    
    /// Build a binary operation instruction
    fn build_binary_op(&mut self, op: Operation, lhs: Value, rhs: Value, result_type: Arc<Type>) -> Result<Value> {
        let instruction = Instruction::binary_op(op, lhs, rhs, result_type);
        self.build_instruction(instruction)
    }
    
    /// Build an add instruction
    pub fn build_add(&mut self, lhs: Value, rhs: Value, name: Option<&str>) -> Result<Value> {
        // Get the type of the result
        let result_type = lhs.ty().ok_or_else(|| {
            Error::TypeError("Left operand of add has no type".to_string())
        })?;
        
        let mut instruction = Instruction::add(lhs, rhs, result_type);
        
        if let Some(name) = name {
            instruction.add_metadata("name", name);
        }
        
        self.build_instruction(instruction)
    }
    
    /// Build a subtract instruction
    pub fn build_sub(&mut self, lhs: Value, rhs: Value, name: Option<&str>) -> Result<Value> {
        // Get the type of the result
        let result_type = lhs.ty().ok_or_else(|| {
            Error::TypeError("Left operand of sub has no type".to_string())
        })?;
        
        let mut instruction = Instruction::sub(lhs, rhs, result_type);
        
        if let Some(name) = name {
            instruction.add_metadata("name", name);
        }
        
        self.build_instruction(instruction)
    }
    
    /// Build a multiply instruction
    pub fn build_mul(&mut self, lhs: Value, rhs: Value, name: Option<&str>) -> Result<Value> {
        // Get the type of the result
        let result_type = lhs.ty().ok_or_else(|| {
            Error::TypeError("Left operand of mul has no type".to_string())
        })?;
        
        let mut instruction = Instruction::mul(lhs, rhs, result_type);
        
        if let Some(name) = name {
            instruction.add_metadata("name", name);
        }
        
        self.build_instruction(instruction)
    }
    
    /// Build a divide instruction
    pub fn build_div(&mut self, lhs: Value, rhs: Value, name: Option<&str>) -> Result<Value> {
        // Get the type of the result
        let result_type = lhs.ty().ok_or_else(|| {
            Error::TypeError("Left operand of div has no type".to_string())
        })?;
        
        let mut instruction = Instruction::div(lhs, rhs, result_type);
        
        if let Some(name) = name {
            instruction.add_metadata("name", name);
        }
        
        self.build_instruction(instruction)
    }
    
    /// Build a comparison instruction
    pub fn build_cmp(&mut self, op: Operation, lhs: Value, rhs: Value, name: Option<&str>) -> Result<Value> {
        // Verify that the operation is a comparison operation
        match op {
            Operation::Eq | Operation::Ne | Operation::Lt | Operation::Le | Operation::Gt | Operation::Ge => {},
            _ => {
                return Err(Error::ConstructionError(format!(
                    "Operation {:?} is not a comparison operation",
                    op
                )));
            }
        }
        
        let mut instruction = Instruction::compare(op, lhs, rhs);
        
        if let Some(name) = name {
            instruction.add_metadata("name", name);
        }
        
        self.build_instruction(instruction)
    }
    
    /// Build a return instruction
    pub fn build_ret(&mut self, value: Option<Value>) -> Result<()> {
        let instruction = Instruction::ret(value);
        
        if let Some(block) = &self.current_block {
            let mut block = block.borrow_mut();
            block.add_instruction(instruction);
            Ok(())
        } else {
            Err(Error::ConstructionError(
                "No insertion point set for builder".to_string()
            ))
        }
    }
    
    /// Build a branch instruction
    pub fn build_br(&mut self, target: Value) -> Result<()> {
        let instruction = Instruction::br(target);
        
        if let Some(block) = &self.current_block {
            let mut block = block.borrow_mut();
            block.add_instruction(instruction);
            Ok(())
        } else {
            Err(Error::ConstructionError(
                "No insertion point set for builder".to_string()
            ))
        }
    }
    
    /// Build a conditional branch instruction
    pub fn build_cond_br(&mut self, condition: Value, true_target: Value, false_target: Value) -> Result<()> {
        let instruction = Instruction::cond_br(condition, true_target, false_target);
        
        if let Some(block) = &self.current_block {
            let mut block = block.borrow_mut();
            block.add_instruction(instruction);
            Ok(())
        } else {
            Err(Error::ConstructionError(
                "No insertion point set for builder".to_string()
            ))
        }
    }
    
    /// Build a call instruction
    pub fn build_call(&mut self, function: Value, args: Vec<Value>, name: Option<&str>) -> Result<Value> {
        // Get the function type
        let fn_ty = function.ty().ok_or_else(|| {
            Error::TypeError("Function has no type".to_string())
        })?;
        
        // Get the return type and clone it for later use
        let mut maybe_return_type = None;
        let mut return_type_for_instruction = None;
        
        if let Type::Function { return_type, .. } = &*fn_ty {
            if **return_type == Type::Void {
                // Void function (no return value)
                maybe_return_type = None;
                return_type_for_instruction = None;
            } else {
                // Function returns a value
                maybe_return_type = Some(return_type.clone());
                return_type_for_instruction = Some(return_type.clone());
            }
        } else {
            return Err(Error::TypeError(format!(
                "Expected function type, got {}",
                fn_ty
            )));
        }
        
        let mut instruction = Instruction::call(function, args, return_type_for_instruction);
        
        if let Some(name) = name {
            instruction.add_metadata("name", name);
        }
        
        if let Some(return_type) = maybe_return_type {
            // Function returns a value
            let value = self.build_instruction(instruction)?;
            Ok(value)
        } else {
            // Void function (no return value)
            if let Some(block) = &self.current_block {
                let mut block = block.borrow_mut();
                block.add_instruction(instruction);
                Ok(Value::undefined(Type::void()))
            } else {
                Err(Error::ConstructionError(
                    "No insertion point set for builder".to_string()
                ))
            }
        }
    }
    
    /// Build an alloca instruction
    pub fn build_alloca(&mut self, ty: Arc<Type>, name: Option<&str>) -> Result<Value> {
        let mut instruction = Instruction::alloca(ty);
        
        if let Some(name) = name {
            instruction.add_metadata("name", name);
        }
        
        self.build_instruction(instruction)
    }
    
    /// Build a load instruction
    pub fn build_load(&mut self, ptr: Value, name: Option<&str>) -> Result<Value> {
        // Get the pointer type
        let ptr_ty = ptr.ty().ok_or_else(|| {
            Error::TypeError("Pointer has no type".to_string())
        })?;
        
        // Get the pointee type
        let pointee_ty = if let Type::Pointer(pointee) = &*ptr_ty {
            pointee.clone()
        } else {
            return Err(Error::TypeError(format!(
                "Expected pointer type, got {}",
                ptr_ty
            )));
        };
        
        let mut instruction = Instruction::load(ptr, pointee_ty);
        
        if let Some(name) = name {
            instruction.add_metadata("name", name);
        }
        
        self.build_instruction(instruction)
    }
    
    /// Build a store instruction
    pub fn build_store(&mut self, value: Value, ptr: Value) -> Result<()> {
        // Get the pointer type
        let ptr_ty = ptr.ty().ok_or_else(|| {
            Error::TypeError("Pointer has no type".to_string())
        })?;
        
        // Get the value type
        let value_ty = value.ty().ok_or_else(|| {
            Error::TypeError("Value has no type".to_string())
        })?;
        
        // Get the pointee type
        let pointee_ty = if let Type::Pointer(pointee) = &*ptr_ty {
            pointee.clone()
        } else {
            return Err(Error::TypeError(format!(
                "Expected pointer type, got {}",
                ptr_ty
            )));
        };
        
        // Check that the value type matches the pointee type
        if *value_ty != *pointee_ty {
            return Err(Error::TypeError(format!(
                "Cannot store value of type {} in pointer to {}",
                value_ty, pointee_ty
            )));
        }
        
        let instruction = Instruction::store(value, ptr);
        
        if let Some(block) = &self.current_block {
            let mut block = block.borrow_mut();
            block.add_instruction(instruction);
            Ok(())
        } else {
            Err(Error::ConstructionError(
                "No insertion point set for builder".to_string()
            ))
        }
    }
    
    /// Build a phi instruction
    pub fn build_phi(&mut self, ty: Arc<Type>, values_and_blocks: Vec<(Value, Value)>, name: Option<&str>) -> Result<Value> {
        let mut instruction = Instruction::phi(ty, values_and_blocks);
        
        if let Some(name) = name {
            instruction.add_metadata("name", name);
        }
        
        self.build_instruction(instruction)
    }
    
    /// Build a print instruction (for debugging)
    pub fn build_print(&mut self, value: Value) -> Result<()> {
        let instruction = Instruction::print(value);
        
        if let Some(block) = &self.current_block {
            let mut block = block.borrow_mut();
            block.add_instruction(instruction);
            Ok(())
        } else {
            Err(Error::ConstructionError(
                "No insertion point set for builder".to_string()
            ))
        }
    }
} 