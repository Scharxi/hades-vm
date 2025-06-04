use thiserror::Error;

/// Errors that can occur during IR operations
#[derive(Debug, Error)]
pub enum Error {
    /// Type checking error
    #[error("Type error: {0}")]
    TypeError(String),
    
    /// Error during IR construction
    #[error("Construction error: {0}")]
    ConstructionError(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Optimization error
    #[error("Optimization error: {0}")]
    OptimizationError(String),
    
    /// Bytecode generation error
    #[error("Bytecode generation error: {0}")]
    BytecodeGenerationError(String),
    
    /// Code generation error
    #[error("Code generation error: {0}")]
    CodeGenError(String),
    
    /// VM error
    #[error("VM error: {0}")]
    VMError(String),
    
    /// Executable format error
    #[error("Executable error: {0}")]
    ExecutableError(String),
    
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Other error
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

/// Create a new executable error
pub fn executable_error(msg: impl Into<String>) -> Error {
    Error::ExecutableError(msg.into())
} 