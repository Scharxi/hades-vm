use thiserror::Error;

/// Errors that can occur during IR operations
#[derive(Error, Debug)]
pub enum Error {
    /// An error in type checking
    #[error("Type error: {0}")]
    TypeError(String),
    
    /// An error in IR validation
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// An error in IR construction
    #[error("Construction error: {0}")]
    ConstructionError(String),
    
    /// An error in IR optimization
    #[error("Optimization error: {0}")]
    OptimizationError(String),
    
    /// An error in bytecode generation
    #[error("Bytecode generation error: {0}")]
    BytecodeGenerationError(String),
    
    /// An error in the VM
    #[error("VM error: {0}")]
    VMError(String),
    
    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for IR operations
pub type Result<T> = std::result::Result<T, Error>;

/// Create a new type error
pub fn type_error(msg: impl Into<String>) -> Error {
    Error::TypeError(msg.into())
}

/// Create a new validation error
pub fn validation_error(msg: impl Into<String>) -> Error {
    Error::ValidationError(msg.into())
}

/// Create a new construction error
pub fn construction_error(msg: impl Into<String>) -> Error {
    Error::ConstructionError(msg.into())
} 