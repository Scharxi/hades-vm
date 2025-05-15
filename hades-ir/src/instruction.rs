use std::fmt;
use std::sync::Arc;
use std::collections::HashMap;

use crate::{
    types::Type,
    value::{Value, ValueId},
    error::Result,
};

/// Represents an operation in the IR.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operation {
    // Arithmetic operations
    /// Add two values
    Add,
    /// Subtract second value from first
    Sub,
    /// Multiply two values
    Mul,
    /// Divide first value by second
    Div,
    /// Calculate remainder of division
    Rem,
    /// Negate a value
    Neg,
    
    // Bitwise operations
    /// Bitwise AND
    And,
    /// Bitwise OR
    Or,
    /// Bitwise XOR
    Xor,
    /// Bitwise NOT
    Not,
    /// Shift left
    Shl,
    /// Shift right
    Shr,
    
    // Comparison operations
    /// Equal
    Eq,
    /// Not equal
    Ne,
    /// Less than
    Lt,
    /// Less than or equal
    Le,
    /// Greater than
    Gt,
    /// Greater than or equal
    Ge,
    
    // Memory operations
    /// Allocate memory on the stack
    Alloca,
    /// Load a value from memory
    Load,
    /// Store a value to memory
    Store,
    /// Get element pointer (for array/struct indexing)
    GetElementPtr,
    
    // Control flow
    /// Return a value from a function
    Ret,
    /// Unconditional branch to a basic block
    Br,
    /// Conditional branch based on a condition
    CondBr,
    /// Switch based on a value
    Switch,
    /// Call a function
    Call,
    
    // Conversion operations
    /// Convert between numeric types
    Cast,
    /// Zero-extend a value
    ZExt,
    /// Sign-extend a value
    SExt,
    /// Truncate a value
    Trunc,
    /// Convert integer to floating point
    IntToFloat,
    /// Convert floating point to integer
    FloatToInt,
    /// Bitcast between types of the same size
    Bitcast,
    
    // Other operations
    /// Get the address of a function
    GetFunctionAddr,
    /// Phi node for SSA form
    Phi,
    /// Select between two values based on a condition
    Select,
    /// Print a value (for debugging)
    Print,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Arithmetic
            Operation::Add => write!(f, "add"),
            Operation::Sub => write!(f, "sub"),
            Operation::Mul => write!(f, "mul"),
            Operation::Div => write!(f, "div"),
            Operation::Rem => write!(f, "rem"),
            Operation::Neg => write!(f, "neg"),
            
            // Bitwise
            Operation::And => write!(f, "and"),
            Operation::Or => write!(f, "or"),
            Operation::Xor => write!(f, "xor"),
            Operation::Not => write!(f, "not"),
            Operation::Shl => write!(f, "shl"),
            Operation::Shr => write!(f, "shr"),
            
            // Comparison
            Operation::Eq => write!(f, "eq"),
            Operation::Ne => write!(f, "ne"),
            Operation::Lt => write!(f, "lt"),
            Operation::Le => write!(f, "le"),
            Operation::Gt => write!(f, "gt"),
            Operation::Ge => write!(f, "ge"),
            
            // Memory
            Operation::Alloca => write!(f, "alloca"),
            Operation::Load => write!(f, "load"),
            Operation::Store => write!(f, "store"),
            Operation::GetElementPtr => write!(f, "getelementptr"),
            
            // Control flow
            Operation::Ret => write!(f, "ret"),
            Operation::Br => write!(f, "br"),
            Operation::CondBr => write!(f, "condbr"),
            Operation::Switch => write!(f, "switch"),
            Operation::Call => write!(f, "call"),
            
            // Conversion
            Operation::Cast => write!(f, "cast"),
            Operation::ZExt => write!(f, "zext"),
            Operation::SExt => write!(f, "sext"),
            Operation::Trunc => write!(f, "trunc"),
            Operation::IntToFloat => write!(f, "inttofl"),
            Operation::FloatToInt => write!(f, "fltoint"),
            Operation::Bitcast => write!(f, "bitcast"),
            
            // Other
            Operation::GetFunctionAddr => write!(f, "getfnaddr"),
            Operation::Phi => write!(f, "phi"),
            Operation::Select => write!(f, "select"),
            Operation::Print => write!(f, "print"),
        }
    }
}

/// An instruction in the IR.
#[derive(Debug, Clone)]
pub struct Instruction {
    /// The ID of this instruction as a value
    id: ValueId,
    /// The operation performed by this instruction
    operation: Operation,
    /// The operands for this instruction
    operands: Vec<Value>,
    /// The type of the result produced by this instruction
    result_type: Option<Arc<Type>>,
    /// Metadata associated with this instruction
    metadata: HashMap<String, String>,
}

impl Instruction {
    /// Create a new instruction
    pub fn new(operation: Operation, operands: Vec<Value>, result_type: Option<Arc<Type>>) -> Self {
        Self {
            id: ValueId::new(),
            operation,
            operands,
            result_type,
            metadata: HashMap::new(),
        }
    }
    
    /// Get the ID of this instruction
    pub fn id(&self) -> ValueId {
        self.id
    }
    
    /// Get the operation of this instruction
    pub fn operation(&self) -> &Operation {
        &self.operation
    }
    
    /// Get the operands of this instruction
    pub fn operands(&self) -> &[Value] {
        &self.operands
    }
    
    /// Get the type of the result produced by this instruction
    pub fn result_type(&self) -> Option<Arc<Type>> {
        self.result_type.clone()
    }
    
    /// Add metadata to this instruction
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Get metadata from this instruction
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Convert this instruction to a value
    pub fn to_value(&self) -> Value {
        if let Some(ty) = &self.result_type {
            Value::Instruction {
                id: self.id,
                ty: ty.clone(),
            }
        } else {
            panic!("Cannot convert void instruction to value")
        }
    }
    
    /// Creates a binary operation instruction
    pub fn binary_op(operation: Operation, lhs: Value, rhs: Value, result_type: Arc<Type>) -> Self {
        Self::new(operation, vec![lhs, rhs], Some(result_type))
    }
    
    /// Creates an add instruction
    pub fn add(lhs: Value, rhs: Value, result_type: Arc<Type>) -> Self {
        Self::binary_op(Operation::Add, lhs, rhs, result_type)
    }
    
    /// Creates a subtract instruction
    pub fn sub(lhs: Value, rhs: Value, result_type: Arc<Type>) -> Self {
        Self::binary_op(Operation::Sub, lhs, rhs, result_type)
    }
    
    /// Creates a multiply instruction
    pub fn mul(lhs: Value, rhs: Value, result_type: Arc<Type>) -> Self {
        Self::binary_op(Operation::Mul, lhs, rhs, result_type)
    }
    
    /// Creates a divide instruction
    pub fn div(lhs: Value, rhs: Value, result_type: Arc<Type>) -> Self {
        Self::binary_op(Operation::Div, lhs, rhs, result_type)
    }
    
    /// Creates a comparison instruction
    pub fn compare(operation: Operation, lhs: Value, rhs: Value) -> Self {
        Self::binary_op(operation, lhs, rhs, Type::boolean())
    }
    
    /// Creates a return instruction
    pub fn ret(value: Option<Value>) -> Self {
        match value {
            Some(v) => Self::new(Operation::Ret, vec![v], None),
            None => Self::new(Operation::Ret, vec![], None),
        }
    }
    
    /// Creates a call instruction
    pub fn call(function: Value, args: Vec<Value>, result_type: Option<Arc<Type>>) -> Self {
        let mut operands = vec![function];
        operands.extend(args);
        Self::new(Operation::Call, operands, result_type)
    }
    
    /// Creates an alloca instruction
    pub fn alloca(ty: Arc<Type>) -> Self {
        let ptr_ty = Type::pointer(ty);
        Self::new(Operation::Alloca, vec![], Some(ptr_ty))
    }
    
    /// Creates a load instruction
    pub fn load(ptr: Value, ty: Arc<Type>) -> Self {
        Self::new(Operation::Load, vec![ptr], Some(ty))
    }
    
    /// Creates a store instruction
    pub fn store(value: Value, ptr: Value) -> Self {
        Self::new(Operation::Store, vec![value, ptr], None)
    }
    
    /// Creates a branch instruction
    pub fn br(target: Value) -> Self {
        Self::new(Operation::Br, vec![target], None)
    }
    
    /// Creates a conditional branch instruction
    pub fn cond_br(condition: Value, true_target: Value, false_target: Value) -> Self {
        Self::new(Operation::CondBr, vec![condition, true_target, false_target], None)
    }
    
    /// Creates a phi instruction
    pub fn phi(ty: Arc<Type>, values_and_blocks: Vec<(Value, Value)>) -> Self {
        let mut operands = Vec::with_capacity(values_and_blocks.len() * 2);
        for (value, block) in values_and_blocks {
            operands.push(value);
            operands.push(block);
        }
        Self::new(Operation::Phi, operands, Some(ty))
    }
    
    /// Creates a print instruction (for debugging)
    pub fn print(value: Value) -> Self {
        Self::new(Operation::Print, vec![value], None)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ty) = &self.result_type {
            write!(f, "{} = {} {}", self.id, self.operation, ty)?;
        } else {
            write!(f, "{}", self.operation)?;
        }
        
        if !self.operands.is_empty() {
            write!(f, " ")?;
            for (i, operand) in self.operands.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", operand)?;
            }
        }
        
        Ok(())
    }
} 