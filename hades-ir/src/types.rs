use std::fmt;
use std::sync::Arc;

/// The Type enum represents all possible types in the IR.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Void type (used for functions that return nothing)
    Void,
    
    /// Integer type with specified bit width (e.g., 8, 16, 32, 64)
    Integer(u32),
    
    /// Floating point type with specified bit width (32 or 64)
    Float(u32),
    
    /// Boolean type
    Boolean,
    
    /// Character type
    Char,
    
    /// String type
    String,
    
    /// Pointer to another type
    Pointer(Arc<Type>),
    
    /// Function type with return type and parameter types
    Function {
        /// Return type
        return_type: Arc<Type>,
        /// Parameter types
        param_types: Vec<Arc<Type>>,
        /// Whether the function is variadic
        is_variadic: bool,
    },
    
    /// Array type with element type and size
    Array {
        /// Element type
        element_type: Arc<Type>,
        /// Number of elements
        size: usize,
    },
    
    /// Structure type with named fields
    Struct {
        /// Name of the struct
        name: String,
        /// Field types
        field_types: Vec<Arc<Type>>,
        /// Field names
        field_names: Vec<String>,
    },
    
    /// Reference to a named type that will be resolved later
    Named(String),
}

impl Type {
    /// Create a new 32-bit integer type
    pub fn i32() -> Arc<Self> {
        Arc::new(Type::Integer(32))
    }
    
    /// Create a new 64-bit integer type
    pub fn i64() -> Arc<Self> {
        Arc::new(Type::Integer(64))
    }
    
    /// Create a new 32-bit floating point type
    pub fn f32() -> Arc<Self> {
        Arc::new(Type::Float(32))
    }
    
    /// Create a new 64-bit floating point type
    pub fn f64() -> Arc<Self> {
        Arc::new(Type::Float(64))
    }
    
    /// Create a new boolean type
    pub fn boolean() -> Arc<Self> {
        Arc::new(Type::Boolean)
    }
    
    /// Create a new character type
    pub fn char() -> Arc<Self> {
        Arc::new(Type::Char)
    }
    
    /// Create a new void type
    pub fn void() -> Arc<Self> {
        Arc::new(Type::Void)
    }
    
    /// Create a new pointer type
    pub fn pointer(pointee: Arc<Self>) -> Arc<Self> {
        Arc::new(Type::Pointer(pointee))
    }
    
    /// Create a new function type
    pub fn function(return_type: Arc<Self>, param_types: Vec<Arc<Self>>, is_variadic: bool) -> Arc<Self> {
        Arc::new(Type::Function {
            return_type,
            param_types,
            is_variadic,
        })
    }
    
    /// Create a new array type
    pub fn array(element_type: Arc<Self>, size: usize) -> Arc<Self> {
        Arc::new(Type::Array {
            element_type,
            size,
        })
    }
    
    /// Create a new struct type
    pub fn structure(name: &str, field_types: Vec<Arc<Self>>, field_names: Vec<String>) -> Arc<Self> {
        Arc::new(Type::Struct {
            name: name.to_string(),
            field_types,
            field_names,
        })
    }
    
    /// Create a new named type reference
    pub fn named(name: &str) -> Arc<Self> {
        Arc::new(Type::Named(name.to_string()))
    }
    
    /// Create a new string type
    pub fn string() -> Arc<Self> {
        Arc::new(Type::String)
    }
    
    /// Returns the size of the type in bytes
    pub fn size_in_bytes(&self) -> Option<usize> {
        match self {
            Type::Void => Some(0),
            Type::Integer(bits) => Some((bits / 8) as usize),
            Type::Float(bits) => Some((bits / 8) as usize),
            Type::Boolean => Some(1),
            Type::Char => Some(4), // Assuming Unicode character
            Type::String => None, // String size is dynamic
            Type::Pointer(_) => Some(8), // Assuming 64-bit pointers
            Type::Array { element_type, size } => {
                element_type.size_in_bytes().map(|s| s * size)
            }
            Type::Struct { field_types, .. } => {
                let mut total = 0;
                for field in field_types {
                    if let Some(size) = field.size_in_bytes() {
                        total += size;
                    } else {
                        return None;
                    }
                }
                Some(total)
            }
            // Function types and named types don't have a directly computable size
            Type::Function { .. } | Type::Named(_) => None,
        }
    }
    
    /// Checks if this type can be cast to another type
    pub fn can_cast_to(&self, target: &Type) -> bool {
        match (self, target) {
            // Same types can always be cast
            (a, b) if a == b => true,
            
            // Integer to integer casts
            (Type::Integer(_), Type::Integer(_)) => true,
            
            // Float to float casts
            (Type::Float(_), Type::Float(_)) => true,
            
            // Integer to float casts
            (Type::Integer(_), Type::Float(_)) => true,
            
            // Float to integer casts
            (Type::Float(_), Type::Integer(_)) => true,
            
            // Boolean to integer casts
            (Type::Boolean, Type::Integer(_)) => true,
            
            // Integer to boolean casts (0 = false, non-0 = true)
            (Type::Integer(_), Type::Boolean) => true,
            
            // Pointer to pointer casts
            (Type::Pointer(_), Type::Pointer(_)) => true,
            
            // Integer to pointer casts
            (Type::Integer(_), Type::Pointer(_)) => true,
            
            // Pointer to integer casts
            (Type::Pointer(_), Type::Integer(_)) => true,
            
            // String to string casts
            (Type::String, Type::String) => true,
            
            // String to pointer casts (for C-style strings)
            (Type::String, Type::Pointer(pointee)) => {
                matches!(**pointee, Type::Char)
            },
            
            // Anything else is not allowed
            _ => false,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Integer(bits) => write!(f, "i{}", bits),
            Type::Float(bits) => write!(f, "f{}", bits),
            Type::Boolean => write!(f, "bool"),
            Type::Char => write!(f, "char"),
            Type::String => write!(f, "string"),
            Type::Pointer(pointee) => write!(f, "{}*", pointee),
            Type::Function { return_type, param_types, is_variadic } => {
                write!(f, "{} (", return_type)?;
                for (i, param) in param_types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                if *is_variadic {
                    if !param_types.is_empty() {
                        write!(f, ", ")?;
                    }
                    write!(f, "...")?;
                }
                write!(f, ")")
            }
            Type::Array { element_type, size } => {
                write!(f, "[{} x {}]", size, element_type)
            }
            Type::Struct { name, .. } => {
                write!(f, "%{}", name)
            }
            Type::Named(name) => {
                write!(f, "%{}", name)
            }
        }
    }
} 