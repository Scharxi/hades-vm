use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::hash::{Hash, Hasher};

use crate::types::Type;

/// Global counter for generating unique value IDs
static VALUE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A unique identifier for values in the IR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(u64);

impl ValueId {
    /// Create a new unique value ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
    
    /// Get the raw ID value
    pub fn raw(&self) -> usize {
        self.0 as usize
    }

    /// Convert to bytes in little-endian format
    pub fn to_le_bytes(&self) -> [u8; 8] {
        self.0.to_le_bytes()
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
    
    /// A string constant
    StringConstant {
        /// The value of the constant
        value: String,
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
            Value::StringConstant { .. } => ValueId::new(),
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
            Value::StringConstant { .. } => Some(Type::string()),
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
    
    /// Create a new string constant
    pub fn string_constant(value: &str) -> Self {
        Self::StringConstant {
            value: value.to_string(),
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
                | Value::StringConstant { .. }
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
            Value::CharConstant { value } => write!(f, "'{}'", value.escape_debug()),
            Value::NullPointer { .. } => write!(f, "null"),
            Value::GlobalVariable { name, .. } => write!(f, "@{}", name),
            Value::Argument { id, name, .. } => {
                if let Some(name) = name {
                    write!(f, "%{}", name)
                } else {
                    write!(f, "{}", id)
                }
            }
            Value::Function { name, .. } => write!(f, "@{}", name),
            Value::Instruction { id, .. } => write!(f, "{}", id),
            Value::BasicBlock { id, name } => {
                if let Some(name) = name {
                    write!(f, "label %{}", name)
                } else {
                    write!(f, "label {}", id)
                }
            }
            Value::Undefined { id, .. } => write!(f, "{}", id),
            Value::StringConstant { value } => write!(f, "\"{}\"", value.escape_debug()),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::IntegerConstant { value: v1, .. }, Value::IntegerConstant { value: v2, .. }) => v1 == v2,
            (Value::FloatConstant { value: v1, .. }, Value::FloatConstant { value: v2, .. }) => v1 == v2,
            (Value::BooleanConstant { value: v1 }, Value::BooleanConstant { value: v2 }) => v1 == v2,
            (Value::CharConstant { value: v1 }, Value::CharConstant { value: v2 }) => v1 == v2,
            (Value::StringConstant { value: v1 }, Value::StringConstant { value: v2 }) => v1 == v2,
            (Value::NullPointer { .. }, Value::NullPointer { .. }) => true,
            (Value::GlobalVariable { id: id1, .. }, Value::GlobalVariable { id: id2, .. }) => id1 == id2,
            (Value::Argument { id: id1, .. }, Value::Argument { id: id2, .. }) => id1 == id2,
            (Value::Function { id: id1, .. }, Value::Function { id: id2, .. }) => id1 == id2,
            (Value::Instruction { id: id1, .. }, Value::Instruction { id: id2, .. }) => id1 == id2,
            (Value::BasicBlock { id: id1, .. }, Value::BasicBlock { id: id2, .. }) => id1 == id2,
            (Value::Undefined { id: id1, .. }, Value::Undefined { id: id2, .. }) => id1 == id2,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::IntegerConstant { value, .. } => {
                0u8.hash(state);
                value.hash(state);
            },
            Value::FloatConstant { value, .. } => {
                1u8.hash(state);
                value.to_bits().hash(state);
            },
            Value::BooleanConstant { value } => {
                2u8.hash(state);
                value.hash(state);
            },
            Value::CharConstant { value } => {
                3u8.hash(state);
                value.hash(state);
            },
            Value::StringConstant { value } => {
                4u8.hash(state);
                value.hash(state);
            },
            Value::NullPointer { .. } => {
                5u8.hash(state);
            },
            Value::GlobalVariable { id, .. } => {
                6u8.hash(state);
                id.hash(state);
            },
            Value::Argument { id, .. } => {
                7u8.hash(state);
                id.hash(state);
            },
            Value::Function { id, .. } => {
                8u8.hash(state);
                id.hash(state);
            },
            Value::Instruction { id, .. } => {
                9u8.hash(state);
                id.hash(state);
            },
            Value::BasicBlock { id, .. } => {
                10u8.hash(state);
                id.hash(state);
            },
            Value::Undefined { id, .. } => {
                11u8.hash(state);
                id.hash(state);
            },
        }
    }
} 