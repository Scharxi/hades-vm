# Guide: Implementing New Functionality in Hades IR

## 1. Preparation and Analysis

### 1.1 Requirements Analysis
- Define exactly what new functionality needs to be implemented
- Check if similar functionality already exists
- Identify affected components in the system

### 1.2 Design Considerations
- Ensure new functionality is compatible with the existing type system
- Consider impacts on existing components
- Plan integration with the existing error handling system

## 2. Implementation Steps

### 2.1 Type System Extensions
If new types are needed:
```rust
// In types.rs
impl Type {
    pub fn new_type_name() -> Arc<Self> {
        // Implementation
    }
}
```

### 2.2 Value Extensions
If new value types are needed:
```rust
// In value.rs
impl Value {
    pub fn new_value_type(/* parameters */) -> Self {
        // Implementation
    }
}
```

### 2.3 Instructions
For new operations:
```rust
// In instruction.rs
impl Instruction {
    pub fn new_instruction(/* parameters */) -> Self {
        // Implementation
    }
}
```

## 3. Test-Driven Development

### 3.1 Create Unit Tests
```rust
#[test]
fn test_new_functionality() {
    // Test Setup
    let context = create_test_context();
    
    // Test Execution
    
    // Assertions
}
```

### 3.2 Integration Tests
```rust
#[test]
fn test_integration_with_existing_components() {
    // Integration Test
}
```

## 4. Documentation

### 4.1 Code Documentation
```rust
/// Describes the new functionality
/// 
/// # Arguments
/// * `param1` - Description of first parameter
/// 
/// # Returns
/// Description of return value
/// 
/// # Errors
/// Description of possible errors
pub fn new_function(param1: Type) -> Result<Value, Error> {
    // Implementation
}
```

### 4.2 README Updates
- Add new functionality to feature list
- Update relevant code examples
- Document new best practices

## 5. Error Handling

### 5.1 New Error Types
```rust
pub enum Error {
    // Existing errors
    NewError(String),
}

impl Error {
    pub fn new_error(msg: impl Into<String>) -> Self {
        Error::NewError(msg.into())
    }
}
```

### 5.2 Validation
```rust
fn validate_new_functionality(/* parameters */) -> Result<(), Error> {
    // Validation logic
}
```

## 6. Best Practices

### 6.1 Coding Guidelines
- Use `Arc` for shared types
- Implement `Clone` where necessary
- Follow existing error handling patterns
- Adhere to Rust formatting guidelines

### 6.2 Performance Considerations
- Minimize unnecessary clones
- Optimize memory usage
- Consider runtime complexity

## 7. Checklist

- [ ] Functionality implemented
- [ ] Tests written and passing
- [ ] Documentation updated
- [ ] Error handling implemented
- [ ] Code formatted and linted
- [ ] Performance tested
- [ ] Existing functionality not impacted

## 8. Example: Implementing a New Operation

```rust
// 1. Define Operation
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    // Existing operations...
    NewOperation,
}

// 2. Implementation
impl Instruction {
    pub fn new_operation(
        value1: Value,
        value2: Value,
        result_type: Arc<Type>
    ) -> Result<Self, Error> {
        // Validation
        if !value1.type_matches(&value2) {
            return Err(Error::TypeError("Type mismatch".into()));
        }

        // Construction
        Ok(Self {
            operation: Operation::NewOperation,
            operands: vec![value1, value2],
            result_type,
        })
    }
}

// 3. Tests
#[cfg(test)]
mod tests {
    #[test]
    fn test_new_operation() {
        let context = create_test_context();
        let i32_type = Type::i32();
        let val1 = Value::integer_constant(1, i32_type.clone());
        let val2 = Value::integer_constant(2, i32_type.clone());
        
        let result = Instruction::new_operation(
            val1,
            val2,
            i32_type
        );
        
        assert!(result.is_ok());
    }
}
```

## 9. Useful Tools

- `cargo test`: Run all tests
- `cargo fmt`: Format code
- `cargo clippy`: Run additional lint checks
- `cargo doc`: Generate documentation

## 10. Support and Help

When you have questions or issues:
1. Check existing documentation
2. Look at similar implementations in the code
3. Use test utilities for new tests
4. Follow established patterns in the codebase

## 11. Common Patterns

### 11.1 Type System Integration
- Always validate type compatibility
- Use Arc for type sharing
- Implement proper type comparison

### 11.2 Value Management
- Ensure proper ownership
- Handle cloning efficiently
- Maintain value uniqueness

### 11.3 Instruction Creation
- Validate operands
- Check type compatibility
- Maintain SSA form

### 11.4 Module Integration
- Register new functionality with modules
- Update verification logic
- Maintain module consistency

## 12. Testing Strategy

### 12.1 Unit Testing
- Test individual components
- Cover edge cases
- Test error conditions

### 12.2 Integration Testing
- Test interaction with existing components
- Verify module-level functionality
- Test end-to-end scenarios

### 12.3 Performance Testing
- Benchmark critical operations
- Compare with existing implementations
- Test with large datasets

## 13. Documentation Requirements

### 13.1 Code Comments
- Document public interfaces
- Explain complex algorithms
- Document assumptions

### 13.2 Examples
- Provide usage examples
- Show common patterns
- Demonstrate error handling

### 13.3 API Documentation
- Document all public functions
- Describe parameters and return values
- Document error conditions

## 14. Optimization Guidelines

### 14.1 Memory Optimization
- Minimize allocations
- Use references where appropriate
- Implement proper cleanup

### 14.2 Performance Optimization
- Profile critical paths
- Optimize hot loops
- Consider caching strategies

### 14.3 Code Organization
- Keep related functionality together
- Use appropriate module structure
- Maintain clear dependencies

This guide should provide a comprehensive overview of how to implement new functionality in Hades IR. Remember that code quality and maintainability are important, so take time for proper testing and documentation. 