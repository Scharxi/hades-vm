use std::sync::Arc;
use crate::error::{Error, Result};
use crate::Type;
use crate::Value;
use crate::Function;
use crate::instruction::{Instruction, Operation};
use crate::basic_block::BasicBlock;

/// Type checker for the Hades IR
pub struct TypeChecker;

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        Self
    }

    /// Check if two types are compatible for assignment
    pub fn check_assignment_compatible(&self, target_type: &Type, value_type: &Type) -> Result<()> {
        match (target_type, value_type) {
            // Same types are always compatible
            (a, b) if a == b => Ok(()),

            // Integer type compatibility
            (Type::Integer(target_bits), Type::Integer(value_bits)) => {
                if target_bits >= value_bits {
                    Ok(())
                } else {
                    Err(Error::TypeError(format!(
                        "Cannot assign i{} to i{} - possible data loss",
                        value_bits, target_bits
                    )))
                }
            }

            // Float type compatibility
            (Type::Float(target_bits), Type::Float(value_bits)) => {
                if target_bits >= value_bits {
                    Ok(())
                } else {
                    Err(Error::TypeError(format!(
                        "Cannot assign f{} to f{} - possible data loss",
                        value_bits, target_bits
                    )))
                }
            }

            // Pointer type compatibility
            (Type::Pointer(target_pointee), Type::Pointer(value_pointee)) => {
                // Allow void* to be assigned to any pointer type
                if matches!(&**value_pointee, Type::Void) {
                    Ok(())
                } else {
                    self.check_assignment_compatible(target_pointee, value_pointee)
                }
            }

            // Array type compatibility
            (Type::Array { element_type: target_elem, size: target_size },
             Type::Array { element_type: value_elem, size: value_size }) => {
                if target_size != value_size {
                    return Err(Error::TypeError(format!(
                        "Array size mismatch: expected {}, got {}",
                        target_size, value_size
                    )));
                }
                self.check_assignment_compatible(target_elem, value_elem)
            }

            // Function type compatibility
            (Type::Function { return_type: target_ret, param_types: target_params, is_variadic: target_var },
             Type::Function { return_type: value_ret, param_types: value_params, is_variadic: value_var }) => {
                if target_var != value_var {
                    return Err(Error::TypeError(
                        "Function variadic mismatch".to_string()
                    ));
                }
                if target_params.len() != value_params.len() {
                    return Err(Error::TypeError(format!(
                        "Function parameter count mismatch: expected {}, got {}",
                        target_params.len(), value_params.len()
                    )));
                }
                // Return type is covariant (value_ret must be assignable to target_ret)
                self.check_assignment_compatible(target_ret, value_ret)?;
                // Parameter types are contravariant (target_param must be assignable to value_param)
                for (target_param, value_param) in target_params.iter().zip(value_params.iter()) {
                    self.check_assignment_compatible(value_param, target_param)?;
                }
                Ok(())
            }

            // Struct type compatibility
            (Type::Struct { name: target_name, field_types: target_fields, .. },
             Type::Struct { name: value_name, field_types: value_fields, .. }) => {
                if target_name != value_name {
                    return Err(Error::TypeError(format!(
                        "Struct type mismatch: {} vs {}",
                        target_name, value_name
                    )));
                }
                if target_fields.len() != value_fields.len() {
                    return Err(Error::TypeError(format!(
                        "Struct field count mismatch: expected {}, got {}",
                        target_fields.len(), value_fields.len()
                    )));
                }
                for (target_field, value_field) in target_fields.iter().zip(value_fields.iter()) {
                    self.check_assignment_compatible(target_field, value_field)?;
                }
                Ok(())
            }

            // Named type compatibility
            (Type::Named(target_name), Type::Named(value_name)) => {
                if target_name == value_name {
                    Ok(())
                } else {
                    Err(Error::TypeError(format!(
                        "Named type mismatch: {} vs {}",
                        target_name, value_name
                    )))
                }
            }

            // Incompatible types
            _ => Err(Error::TypeError(format!(
                "Type mismatch: cannot assign {} to {}",
                value_type, target_type
            )))
        }
    }

    /// Check if a value has the expected type
    pub fn check_value_type(&self, value: &Value, expected_type: &Type) -> Result<()> {
        let value_type = value.ty().ok_or_else(|| {
            Error::TypeError("Value has no type".to_string())
        })?;
        
        self.check_assignment_compatible(expected_type, &value_type)
    }

    /// Check if an instruction's operands have valid types
    pub fn check_instruction(&self, instruction: &Instruction) -> Result<()> {
        match instruction.operation() {
            Operation::Add | Operation::Sub | Operation::Mul | Operation::Div => {
                let operands = instruction.operands();
                if operands.len() != 2 {
                    return Err(Error::TypeError(format!(
                        "Binary operation requires 2 operands, got {}",
                        operands.len()
                    )));
                }
                // Check that both operands have numeric types
                self.check_numeric_operands(&operands[0], &operands[1])?;
                // Check that the result type matches the operands
                if let Some(result_type) = instruction.result_type() {
                    self.check_value_type(&operands[0], &result_type)?;
                    self.check_value_type(&operands[1], &result_type)
                } else {
                    Err(Error::TypeError("Binary operation must have a result type".to_string()))
                }
            }

            Operation::Load => {
                let operands = instruction.operands();
                if operands.len() != 1 {
                    return Err(Error::TypeError(format!(
                        "Load instruction requires 1 operand, got {}",
                        operands.len()
                    )));
                }
                // Check that operand is a pointer type
                let operand_type = operands[0].ty().ok_or_else(|| {
                    Error::TypeError("Load operand has no type".to_string())
                })?;
                
                if let Type::Pointer(pointee) = &*operand_type {
                    // Check that the pointee type matches the result type
                    if let Some(result_type) = instruction.result_type() {
                        self.check_assignment_compatible(&result_type, pointee)
                    } else {
                        Err(Error::TypeError("Load instruction must have a result type".to_string()))
                    }
                } else {
                    Err(Error::TypeError("Load instruction requires pointer operand".to_string()))
                }
            }

            Operation::Store => {
                let operands = instruction.operands();
                if operands.len() != 2 {
                    return Err(Error::TypeError(format!(
                        "Store instruction requires 2 operands, got {}",
                        operands.len()
                    )));
                }
                let value = &operands[0];
                let ptr = &operands[1];
                
                let ptr_type = ptr.ty().ok_or_else(|| {
                    Error::TypeError("Store target has no type".to_string())
                })?;
                
                if let Type::Pointer(pointee) = &*ptr_type {
                    self.check_value_type(value, pointee)
                } else {
                    Err(Error::TypeError("Store instruction requires pointer target".to_string()))
                }
            }

            Operation::Ret => {
                let operands = instruction.operands();
                if operands.len() > 1 {
                    return Err(Error::TypeError(format!(
                        "Return instruction requires 0 or 1 operands, got {}",
                        operands.len()
                    )));
                }
                
                if let Some(value) = operands.first() {
                    let value_type = value.ty().ok_or_else(|| {
                        Error::TypeError("Return value has no type".to_string())
                    })?;
                    // The function's return type should be checked at a higher level
                    Ok(())
                } else {
                    Ok(()) // Void return
                }
            }

            // Add more instruction type checking as needed...
            _ => Ok(()) // Placeholder for other instructions
        }
    }

    /// Check if a function's type signature is valid
    pub fn check_function(&self, function: &Function) -> Result<()> {
        // Check function type
        let fn_type = function.ty();
        if let Type::Function { return_type, param_types, .. } = &*fn_type {
            // Check return type
            if let Some(last_block) = function.basic_blocks().back() {
                if let Some(terminator) = last_block.get_terminator() {
                    if let Operation::Ret = terminator.operation() {
                        let operands = terminator.operands();
                        match operands.first() {
                            Some(value) => self.check_value_type(value, return_type)?,
                            None => {
                                if !matches!(&**return_type, Type::Void) {
                                    return Err(Error::TypeError(
                                        "Function must return a value".to_string()
                                    ));
                                }
                            }
                        }
                    } else {
                        return Err(Error::TypeError(
                            "Function must end with return instruction".to_string()
                        ));
                    }
                }
            }

            // Check parameters
            for (param, param_type) in function.parameters().iter().zip(param_types.iter()) {
                self.check_value_type(&param.to_value(), param_type)?;
            }

            // Check all instructions in all basic blocks
            for block in function.basic_blocks() {
                for instruction in block.instructions() {
                    self.check_instruction(instruction)?;
                }
            }
        } else {
            return Err(Error::TypeError("Invalid function type".to_string()));
        }

        Ok(())
    }

    /// Helper function to check if operands are numeric types
    pub fn check_numeric_operands(&self, lhs: &Value, rhs: &Value) -> Result<()> {
        let lhs_type = lhs.ty().ok_or_else(|| {
            Error::TypeError("Left operand has no type".to_string())
        })?;
        
        let rhs_type = rhs.ty().ok_or_else(|| {
            Error::TypeError("Right operand has no type".to_string())
        })?;

        match (&*lhs_type, &*rhs_type) {
            (Type::Integer(_), Type::Integer(_)) |
            (Type::Float(_), Type::Float(_)) => Ok(()),
            _ => Err(Error::TypeError(format!(
                "Invalid operand types for arithmetic operation: {} and {}",
                lhs_type, rhs_type
            )))
        }
    }
}