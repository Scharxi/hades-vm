//! Primitive Type Operations
//!
//! This module implements runtime support for primitive type methods
//! that are compiled into the bytecode by the nyx-compiler.

use crate::stack::{Stack, StackValue};
use std::f32::consts::{E, PI};

/// Error type for primitive operations
#[derive(Debug, Clone)]
pub enum PrimitiveOpError {
    InvalidArguments(String),
    TypeMismatch(String),
    RuntimeError(String),
}

impl std::fmt::Display for PrimitiveOpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimitiveOpError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            PrimitiveOpError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            PrimitiveOpError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for PrimitiveOpError {}

pub type PrimitiveOpResult<T> = Result<T, PrimitiveOpError>;

/// Primitive type operations handler
pub struct PrimitiveOps;

impl PrimitiveOps {
    /// Execute a primitive method call
    pub fn execute_method(
        method_name: &str,
        stack: &mut Stack,
    ) -> PrimitiveOpResult<()> {
        let parts: Vec<&str> = method_name.split('.').collect();
        if parts.len() != 2 {
            return Err(PrimitiveOpError::InvalidArguments(
                format!("Invalid method name format: {}", method_name)
            ));
        }

        let type_name = parts[0];
        let method = parts[1];

        match type_name {
            "int" => Self::execute_int_method(method, stack),
            "float" => Self::execute_float_method(method, stack),
            "bool" => Self::execute_bool_method(method, stack),
            "string" => Self::execute_string_method(method, stack),
            _ => Err(PrimitiveOpError::InvalidArguments(
                format!("Unknown primitive type: {}", type_name)
            )),
        }
    }

    /// Execute an Int method
    fn execute_int_method(method: &str, stack: &mut Stack) -> PrimitiveOpResult<()> {
        match method {
            "abs" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "abs requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Integer(n) = val {
                    stack.push(StackValue::Integer(n.abs()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected integer, got {:?}", val)
                    ))
                }
            }
            
            "sign" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "sign requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Integer(n) = val {
                    let sign = if n > 0 { 1 } else if n < 0 { -1 } else { 0 };
                    stack.push(StackValue::Integer(sign));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected integer, got {:?}", val)
                    ))
                }
            }
            
            "pow" => {
                let exponent = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "pow requires 2 arguments".to_string()
                ))?;
                let base = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "pow requires 2 arguments".to_string()
                ))?;
                
                if let (StackValue::Integer(b), StackValue::Integer(e)) = (base, exponent) {
                    if e < 0 {
                        return Err(PrimitiveOpError::RuntimeError(
                            "Negative exponents not supported for integer pow".to_string()
                        ));
                    }
                    let result = b.pow(e as u32);
                    stack.push(StackValue::Integer(result));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        "pow requires two integers".to_string()
                    ))
                }
            }
            
            "isEven" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "isEven requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Integer(n) = val {
                    stack.push(StackValue::Boolean(n % 2 == 0));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected integer, got {:?}", val)
                    ))
                }
            }
            
            "isOdd" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "isOdd requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Integer(n) = val {
                    stack.push(StackValue::Boolean(n % 2 != 0));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected integer, got {:?}", val)
                    ))
                }
            }
            
            "isPrime" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "isPrime requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Integer(n) = val {
                    let is_prime = Self::is_prime(n);
                    stack.push(StackValue::Boolean(is_prime));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected integer, got {:?}", val)
                    ))
                }
            }
            
            "toFloat" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "toFloat requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Integer(n) = val {
                    stack.push(StackValue::Float(n as f32));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected integer, got {:?}", val)
                    ))
                }
            }
            
            "toString" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "toString requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Integer(n) = val {
                    stack.push(StackValue::String(n.to_string()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected integer, got {:?}", val)
                    ))
                }
            }
            
            _ => Err(PrimitiveOpError::InvalidArguments(
                format!("Unknown Int method: {}", method)
            )),
        }
    }

    /// Execute a Float method
    fn execute_float_method(method: &str, stack: &mut Stack) -> PrimitiveOpResult<()> {
        match method {
            "abs" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "abs requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    stack.push(StackValue::Float(f.abs()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            "sin" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "sin requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    stack.push(StackValue::Float(f.sin()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            "cos" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "cos requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    stack.push(StackValue::Float(f.cos()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            "tan" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "tan requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    stack.push(StackValue::Float(f.tan()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            "sqrt" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "sqrt requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    if f < 0.0 {
                        return Err(PrimitiveOpError::RuntimeError(
                            "Cannot take square root of negative number".to_string()
                        ));
                    }
                    stack.push(StackValue::Float(f.sqrt()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            "ln" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "ln requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    if f <= 0.0 {
                        return Err(PrimitiveOpError::RuntimeError(
                            "Cannot take natural log of non-positive number".to_string()
                        ));
                    }
                    stack.push(StackValue::Float(f.ln()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            "exp" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "exp requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    stack.push(StackValue::Float(f.exp()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            "round" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "round requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    stack.push(StackValue::Integer(f.round() as i32));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            "floor" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "floor requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    stack.push(StackValue::Integer(f.floor() as i32));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            "ceil" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "ceil requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    stack.push(StackValue::Integer(f.ceil() as i32));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            "isNaN" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "isNaN requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    stack.push(StackValue::Boolean(f.is_nan()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            "isInfinite" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "isInfinite requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    stack.push(StackValue::Boolean(f.is_infinite()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            "toInt" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "toInt requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    stack.push(StackValue::Integer(f as i32));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            "toString" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "toString requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Float(f) = val {
                    stack.push(StackValue::String(f.to_string()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected float, got {:?}", val)
                    ))
                }
            }
            
            _ => Err(PrimitiveOpError::InvalidArguments(
                format!("Unknown Float method: {}", method)
            )),
        }
    }

    /// Execute a Bool method
    fn execute_bool_method(method: &str, stack: &mut Stack) -> PrimitiveOpResult<()> {
        match method {
            "and" => {
                let rhs = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "and requires 2 arguments".to_string()
                ))?;
                let lhs = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "and requires 2 arguments".to_string()
                ))?;
                
                if let (StackValue::Boolean(a), StackValue::Boolean(b)) = (lhs, rhs) {
                    stack.push(StackValue::Boolean(a && b));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        "and requires two booleans".to_string()
                    ))
                }
            }
            
            "or" => {
                let rhs = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "or requires 2 arguments".to_string()
                ))?;
                let lhs = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "or requires 2 arguments".to_string()
                ))?;
                
                if let (StackValue::Boolean(a), StackValue::Boolean(b)) = (lhs, rhs) {
                    stack.push(StackValue::Boolean(a || b));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        "or requires two booleans".to_string()
                    ))
                }
            }
            
            "xor" => {
                let rhs = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "xor requires 2 arguments".to_string()
                ))?;
                let lhs = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "xor requires 2 arguments".to_string()
                ))?;
                
                if let (StackValue::Boolean(a), StackValue::Boolean(b)) = (lhs, rhs) {
                    stack.push(StackValue::Boolean(a ^ b));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        "xor requires two booleans".to_string()
                    ))
                }
            }
            
            "not" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "not requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Boolean(b) = val {
                    stack.push(StackValue::Boolean(!b));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected boolean, got {:?}", val)
                    ))
                }
            }
            
            "toString" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "toString requires 1 argument".to_string()
                ))?;
                
                if let StackValue::Boolean(b) = val {
                    stack.push(StackValue::String(b.to_string()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected boolean, got {:?}", val)
                    ))
                }
            }
            
            _ => Err(PrimitiveOpError::InvalidArguments(
                format!("Unknown Bool method: {}", method)
            )),
        }
    }

    /// Execute a String method
    fn execute_string_method(method: &str, stack: &mut Stack) -> PrimitiveOpResult<()> {
        match method {
            "length" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "length requires 1 argument".to_string()
                ))?;
                
                if let StackValue::String(s) = val {
                    stack.push(StackValue::Integer(s.len() as i32));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected string, got {:?}", val)
                    ))
                }
            }
            
            "substring" => {
                let length = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "substring requires 3 arguments".to_string()
                ))?;
                let start = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "substring requires 3 arguments".to_string()
                ))?;
                let string = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "substring requires 3 arguments".to_string()
                ))?;
                
                if let (StackValue::String(s), StackValue::Integer(start), StackValue::Integer(len)) = 
                    (string, start, length) {
                    let start_idx = start as usize;
                    let end_idx = (start + len) as usize;
                    
                    if start_idx <= s.len() && end_idx <= s.len() && start_idx <= end_idx {
                        let substring = s.chars().skip(start_idx).take(len as usize).collect::<String>();
                        stack.push(StackValue::String(substring));
                        Ok(())
                    } else {
                        Err(PrimitiveOpError::RuntimeError(
                            "String index out of bounds".to_string()
                        ))
                    }
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        "substring requires string and two integers".to_string()
                    ))
                }
            }
            
            "concat" => {
                let rhs = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "concat requires 2 arguments".to_string()
                ))?;
                let lhs = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "concat requires 2 arguments".to_string()
                ))?;
                
                if let (StackValue::String(s1), StackValue::String(s2)) = (lhs, rhs) {
                    stack.push(StackValue::String(format!("{}{}", s1, s2)));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        "concat requires two strings".to_string()
                    ))
                }
            }
            
            "toLowerCase" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "toLowerCase requires 1 argument".to_string()
                ))?;
                
                if let StackValue::String(s) = val {
                    stack.push(StackValue::String(s.to_lowercase()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected string, got {:?}", val)
                    ))
                }
            }
            
            "toUpperCase" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "toUpperCase requires 1 argument".to_string()
                ))?;
                
                if let StackValue::String(s) = val {
                    stack.push(StackValue::String(s.to_uppercase()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected string, got {:?}", val)
                    ))
                }
            }
            
            "trim" => {
                let val = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "trim requires 1 argument".to_string()
                ))?;
                
                if let StackValue::String(s) = val {
                    stack.push(StackValue::String(s.trim().to_string()));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        format!("Expected string, got {:?}", val)
                    ))
                }
            }
            
            "contains" => {
                let needle = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "contains requires 2 arguments".to_string()
                ))?;
                let haystack = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "contains requires 2 arguments".to_string()
                ))?;
                
                if let (StackValue::String(s), StackValue::String(sub)) = (haystack, needle) {
                    stack.push(StackValue::Boolean(s.contains(&sub)));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        "contains requires two strings".to_string()
                    ))
                }
            }
            
            "startsWith" => {
                let prefix = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "startsWith requires 2 arguments".to_string()
                ))?;
                let string = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "startsWith requires 2 arguments".to_string()
                ))?;
                
                if let (StackValue::String(s), StackValue::String(prefix)) = (string, prefix) {
                    stack.push(StackValue::Boolean(s.starts_with(&prefix)));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        "startsWith requires two strings".to_string()
                    ))
                }
            }
            
            "endsWith" => {
                let suffix = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "endsWith requires 2 arguments".to_string()
                ))?;
                let string = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "endsWith requires 2 arguments".to_string()
                ))?;
                
                if let (StackValue::String(s), StackValue::String(suffix)) = (string, suffix) {
                    stack.push(StackValue::Boolean(s.ends_with(&suffix)));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        "endsWith requires two strings".to_string()
                    ))
                }
            }
            
            "indexOf" => {
                let needle = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "indexOf requires 2 arguments".to_string()
                ))?;
                let haystack = stack.pop().ok_or_else(|| PrimitiveOpError::InvalidArguments(
                    "indexOf requires 2 arguments".to_string()
                ))?;
                
                if let (StackValue::String(s), StackValue::String(sub)) = (haystack, needle) {
                    let index = s.find(&sub).map(|i| i as i32).unwrap_or(-1);
                    stack.push(StackValue::Integer(index));
                    Ok(())
                } else {
                    Err(PrimitiveOpError::TypeMismatch(
                        "indexOf requires two strings".to_string()
                    ))
                }
            }
            
            _ => Err(PrimitiveOpError::InvalidArguments(
                format!("Unknown String method: {}", method)
            )),
        }
    }

    /// Helper function to check if a number is prime
    fn is_prime(n: i32) -> bool {
        if n < 2 {
            return false;
        }
        if n == 2 {
            return true;
        }
        if n % 2 == 0 {
            return false;
        }
        
        let sqrt_n = (n as f64).sqrt() as i32;
        for i in (3..=sqrt_n).step_by(2) {
            if n % i == 0 {
                return false;
            }
        }
        true
    }

    /// Get static constants for primitive types
    pub fn get_static_constant(type_name: &str, constant_name: &str) -> Option<StackValue> {
        match (type_name, constant_name) {
            ("int", "MIN_VALUE") => Some(StackValue::Integer(i32::MIN)),
            ("int", "MAX_VALUE") => Some(StackValue::Integer(i32::MAX)),
            ("int", "ZERO") => Some(StackValue::Integer(0)),
            ("int", "ONE") => Some(StackValue::Integer(1)),
            ("float", "NaN") => Some(StackValue::Float(f32::NAN)),
            ("float", "POSITIVE_INFINITY") => Some(StackValue::Float(f32::INFINITY)),
            ("float", "NEGATIVE_INFINITY") => Some(StackValue::Float(f32::NEG_INFINITY)),
            ("float", "PI") => Some(StackValue::Float(PI)),
            ("float", "E") => Some(StackValue::Float(E)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack::{Stack, StackValue};

    #[test]
    fn test_int_abs() {
        let mut stack = Stack::new(10);
        stack.push(StackValue::Integer(-42));
        
        assert!(PrimitiveOps::execute_method("int.abs", &mut stack).is_ok());
        
        let result = stack.pop().unwrap();
        assert_eq!(result, StackValue::Integer(42));
    }

    #[test]
    fn test_int_pow() {
        let mut stack = Stack::new(10);
        stack.push(StackValue::Integer(2));  // base
        stack.push(StackValue::Integer(3));  // exponent
        
        assert!(PrimitiveOps::execute_method("int.pow", &mut stack).is_ok());
        
        let result = stack.pop().unwrap();
        assert_eq!(result, StackValue::Integer(8));
    }

    #[test]
    fn test_int_is_even() {
        let mut stack = Stack::new(10);
        stack.push(StackValue::Integer(42));
        
        assert!(PrimitiveOps::execute_method("int.isEven", &mut stack).is_ok());
        
        let result = stack.pop().unwrap();
        assert_eq!(result, StackValue::Boolean(true));
    }

    #[test]
    fn test_float_sin() {
        let mut stack = Stack::new(10);
        stack.push(StackValue::Float(0.0));
        
        assert!(PrimitiveOps::execute_method("float.sin", &mut stack).is_ok());
        
        let result = stack.pop().unwrap();
        if let StackValue::Float(f) = result {
            assert!((f - 0.0).abs() < 1e-7); // Use f32 precision
        } else {
            panic!("Expected float result");
        }
    }

    #[test]
    fn test_string_length() {
        let mut stack = Stack::new(10);
        stack.push(StackValue::String("Hello".to_string()));
        
        assert!(PrimitiveOps::execute_method("string.length", &mut stack).is_ok());
        
        let result = stack.pop().unwrap();
        assert_eq!(result, StackValue::Integer(5));
    }

    #[test]
    fn test_string_concat() {
        let mut stack = Stack::new(10);
        stack.push(StackValue::String("Hello".to_string()));
        stack.push(StackValue::String(" World".to_string()));
        
        assert!(PrimitiveOps::execute_method("string.concat", &mut stack).is_ok());
        
        let result = stack.pop().unwrap();
        assert_eq!(result, StackValue::String("Hello World".to_string()));
    }

    #[test]
    fn test_bool_and() {
        let mut stack = Stack::new(10);
        stack.push(StackValue::Boolean(true));
        stack.push(StackValue::Boolean(false));
        
        assert!(PrimitiveOps::execute_method("bool.and", &mut stack).is_ok());
        
        let result = stack.pop().unwrap();
        assert_eq!(result, StackValue::Boolean(false));
    }
} 