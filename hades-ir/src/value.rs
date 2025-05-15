use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::types::Type;

/// Global counter for generating unique value IDs
static VALUE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A unique identifier for values in the IR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(usize);

impl ValueId {
    /// Create a new unique value ID
    pub fn new() -> Self {
        let id = VALUE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(id)
    }
    
    /// Get the raw ID value
    pub fn raw(&self) -> usize {
        self.0
    }
}

impl Default for ValueId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ValueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// A value in the IR.
#[derive(Debug, Clone)]
pub enum Value {
    /// An integer constant
    IntegerConstant {
        /// The type of the constant
        ty: Arc<Type>,
        /// The value of the constant
        value: i64,
    },
    
    /// A floating point constant
    FloatConstant {
        /// The type of the constant
        ty: Arc<Type>,
        /// The value of the constant
        value: f64,
    },
    
    /// A boolean constant
    BooleanConstant {
        /// The value of the constant
        value: bool,
    },
    
    /// A character constant
    CharConstant {
        /// The value of the constant
        value: char,
    },
    
    /// A null pointer constant
    NullPointer {
        /// The type of the null pointer
        ty: Arc<Type>,
    },
    
    /// A reference to a global variable
    GlobalVariable {
        /// The ID of the value
        id: ValueId,
        /// The name of the global variable
        name: String,
        /// The type of the global variable
        ty: Arc<Type>,
    },
    
    /// A function argument
    Argument {
        /// The ID of the value
        id: ValueId,
        /// The index of the argument
        index: usize,
        /// The name of the argument (if any)
        name: Option<String>,
        /// The type of the argument
        ty: Arc<Type>,
    },
    
    /// A reference to a function
    Function {
        /// The ID of the value
        id: ValueId,
        /// The name of the function
        name: String,
        /// The type of the function
        ty: Arc<Type>,
    },
    
    /// A reference to an instruction
    Instruction {
        /// The ID of the value
        id: ValueId,
        /// The type of the instruction result
        ty: Arc<Type>,
    },
    
    /// A basic block
    BasicBlock {
        /// The ID of the value
        id: ValueId,
        /// The name of the basic block (if any)
        name: Option<String>,
    },
    
    /// An undefined value
    Undefined {
        /// The ID of the value
        id: ValueId,
        /// The type of the undefined value
        ty: Arc<Type>,
    },
}

impl Value {
    /// Get the ID of the value
    pub fn id(&self) -> ValueId {
        match self {
            Value::IntegerConstant { .. } => ValueId::new(), // Constants get new IDs each time
            Value::FloatConstant { .. } => ValueId::new(),
            Value::BooleanConstant { .. } => ValueId::new(),
            Value::CharConstant { .. } => ValueId::new(),
            Value::NullPointer { .. } => ValueId::new(),
            Value::GlobalVariable { id, .. } => *id,
            Value::Argument { id, .. } => *id,
            Value::Function { id, .. } => *id,
            Value::Instruction { id, .. } => *id,
            Value::BasicBlock { id, .. } => *id,
            Value::Undefined { id, .. } => *id,
        }
    }
    
    /// Get the type of the value
    pub fn ty(&self) -> Option<Arc<Type>> {
        match self {
            Value::IntegerConstant { ty, .. } => Some(ty.clone()),
            Value::FloatConstant { ty, .. } => Some(ty.clone()),
            Value::BooleanConstant { .. } => Some(Type::boolean()),
            Value::CharConstant { .. } => Some(Type::char()),
            Value::NullPointer { ty } => Some(ty.clone()),
            Value::GlobalVariable { ty, .. } => Some(ty.clone()),
            Value::Argument { ty, .. } => Some(ty.clone()),
            Value::Function { ty, .. } => Some(ty.clone()),
            Value::Instruction { ty, .. } => Some(ty.clone()),
            Value::BasicBlock { .. } => None, // Basic blocks don't have a proper type
            Value::Undefined { ty, .. } => Some(ty.clone()),
        }
    }
    
    /// Create a new integer constant
    pub fn integer_constant(value: i64, ty: Arc<Type>) -> Self {
        Self::IntegerConstant { ty, value }
    }
    
    /// Create a new floating point constant
    pub fn float_constant(value: f64, ty: Arc<Type>) -> Self {
        Self::FloatConstant { ty, value }
    }
    
    /// Create a new boolean constant
    pub fn boolean_constant(value: bool) -> Self {
        Self::BooleanConstant { value }
    }
    
    /// Create a new character constant
    pub fn char_constant(value: char) -> Self {
        Self::CharConstant { value }
    }
    
    /// Create a new null pointer constant
    pub fn null_pointer(pointee_ty: Arc<Type>) -> Self {
        let ty = Type::pointer(pointee_ty);
        Self::NullPointer { ty }
    }
    
    /// Create a new global variable
    pub fn global_variable(name: &str, ty: Arc<Type>) -> Self {
        Self::GlobalVariable {
            id: ValueId::new(),
            name: name.to_string(),
            ty,
        }
    }
    
    /// Create a new function argument
    pub fn argument(index: usize, name: Option<&str>, ty: Arc<Type>) -> Self {
        Self::Argument {
            id: ValueId::new(),
            index,
            name: name.map(|s| s.to_string()),
            ty,
        }
    }
    
    /// Create a new function reference
    pub fn function(name: &str, ty: Arc<Type>) -> Self {
        Self::Function {
            id: ValueId::new(),
            name: name.to_string(),
            ty,
        }
    }
    
    /// Create a new instruction reference
    pub fn instruction(ty: Arc<Type>) -> Self {
        Self::Instruction {
            id: ValueId::new(),
            ty,
        }
    }
    
    /// Create a new basic block
    pub fn basic_block(name: Option<&str>) -> Self {
        Self::BasicBlock {
            id: ValueId::new(),
            name: name.map(|s| s.to_string()),
        }
    }
    
    /// Create a new undefined value
    pub fn undefined(ty: Arc<Type>) -> Self {
        Self::Undefined {
            id: ValueId::new(),
            ty,
        }
    }
    
    /// Check if this value is a constant
    pub fn is_constant(&self) -> bool {
        matches!(
            self,
            Value::IntegerConstant { .. }
                | Value::FloatConstant { .. }
                | Value::BooleanConstant { .. }
                | Value::CharConstant { .. }
                | Value::NullPointer { .. }
        )
    }
    
    /// Check if this value is a global
    pub fn is_global(&self) -> bool {
        matches!(self, Value::GlobalVariable { .. } | Value::Function { .. })
    }
    
    /// Check if this value is a function
    pub fn is_function(&self) -> bool {
        matches!(self, Value::Function { .. })
    }
    
    /// Check if this value is an instruction
    pub fn is_instruction(&self) -> bool {
        matches!(self, Value::Instruction { .. })
    }
    
    /// Check if this value is a basic block
    pub fn is_basic_block(&self) -> bool {
        matches!(self, Value::BasicBlock { .. })
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::IntegerConstant { value, .. } => write!(f, "{}", value),
            Value::FloatConstant { value, .. } => write!(f, "{}", value),
            Value::BooleanConstant { value } => write!(f, "{}", value),
            Value::CharConstant { value } => write!(f, "'{}'", value),
            Value::NullPointer { .. } => write!(f, "null"),
            Value::GlobalVariable { name, .. } => write!(f, "@{}", name),
            Value::Argument { name, id, .. } => {
                if let Some(name) = name {
                    write!(f, "%{}", name)
                } else {
                    write!(f, "{}", id)
                }
            }
            Value::Function { name, .. } => write!(f, "@{}", name),
            Value::Instruction { id, .. } => write!(f, "{}", id),
            Value::BasicBlock { name, id } => {
                if let Some(name) = name {
                    write!(f, "label %{}", name)
                } else {
                    write!(f, "label {}", id)
                }
            }
            Value::Undefined { .. } => write!(f, "undef"),
        }
    }
} 