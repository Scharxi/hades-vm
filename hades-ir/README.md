# Hades IR Documentation

Hades IR is a strongly-typed intermediate representation (IR) designed for program analysis and optimization. It provides a robust foundation for representing and manipulating programs in a compiler pipeline.

## Core Features

### Type System

The IR implements a rich type system that includes:

- **Primitive Types**
  - Integer types (i32, i64)
  - Floating-point types (f32, f64)
  - Boolean type
  - Character type
  - Void type

- **Complex Types**
  - Pointer types (`Type::pointer(pointee_type)`)
  - Array types with fixed size (`Type::array(element_type, size)`)
  - Structure types with named fields (`Type::structure(name, field_types, field_names)`)
  - Function types (`Type::function(return_type, param_types, is_variadic)`)
  - Named types for forward declarations

### Values

The IR supports various kinds of values:

- **Constants**
  - Integer constants
  - Floating-point constants
  - Boolean constants
  - Character constants
  - Null pointer constants

- **Variables and References**
  - Global variables
  - Function arguments
  - Function references
  - Basic block references
  - Instruction results

### Instructions

Instructions represent operations in the IR:

- **Arithmetic Operations**
  - Add, Sub, Mul, Div, Rem
  - Neg (negation)

- **Bitwise Operations**
  - And, Or, Xor, Not
  - Shl (shift left), Shr (shift right)

- **Comparison Operations**
  - Eq, Ne (equality/inequality)
  - Lt, Le, Gt, Ge (ordering)

- **Memory Operations**
  - Alloca (stack allocation)
  - Load (read from memory)
  - Store (write to memory)
  - GetElementPtr (array/struct element addressing)

- **Control Flow**
  - Ret (return from function)
  - Br (unconditional branch)
  - CondBr (conditional branch)
  - Switch (multi-way branch)
  - Call (function invocation)

- **Type Conversion**
  - Cast (generic type conversion)
  - ZExt, SExt (zero/sign extension)
  - Trunc (truncation)
  - IntToFloat, FloatToInt
  - Bitcast (reinterpret bits)

### Basic Blocks

Basic blocks are sequences of instructions with the following properties:

- Can be named or unnamed
- Contain a list of instructions
- Must end with a terminator instruction (e.g., Ret, Br, CondBr)
- Support metadata attachments
- Can be referenced as values

### Functions

Functions are the primary unit of code organization:

- **Properties**
  - Name
  - Type (including return type and parameter types)
  - Linkage type (External, Internal, InlineOnly, LinkOnceODR)
  - Parameters with optional names
  - Basic blocks
  - Metadata

- **Verification**
  - Functions must have at least one basic block
  - All basic blocks must end with a terminator
  - Parameter types must match function type
  - Basic blocks must be properly linked

### Modules

Modules are the top-level containers in the IR:

- **Properties**
  - Name
  - Source file information
  - Global variables
  - Functions
  - Type definitions
  - Metadata

- **Capabilities**
  - Function management (add, remove, lookup)
  - Global variable management
  - Type definition management
  - Module-level verification
  - Cross-function optimization

### Error Handling

The IR uses a robust error handling system:

- **Error Types**
  - `TypeError`: Type mismatch or invalid type usage
  - `ValidationError`: IR structure validation failures
  - `ConstructionError`: Issues during IR construction
  - `OptimizationError`: Problems during optimization
  - `BytecodeGenerationError`: Issues in bytecode generation
  - `VMError`: Runtime/VM-related errors

- **Error Context**
  - Detailed error messages
  - Location information when available
  - Suggestions for resolution
  - Error propagation through Result types

- **Common Error Cases**
  ```rust
  // Type mismatch
  fn type_error(msg: impl Into<String>) -> Error {
      Error::TypeError(msg.into())
  }

  // Validation failure
  fn validation_error(msg: impl Into<String>) -> Error {
      Error::ValidationError(msg.into())
  }

  // Construction issue
  fn construction_error(msg: impl Into<String>) -> Error {
      Error::ConstructionError(msg.into())
  }
  ```

### Metadata

The IR supports attaching metadata to various entities:

- Functions can have metadata (e.g., author, version)
- Basic blocks can have metadata
- Instructions can have metadata
- Metadata is stored as key-value pairs of strings

## Usage Examples

### Creating a Function

```rust
let i32_type = Type::i32();
let fn_type = Type::function(i32_type.clone(), vec![i32_type.clone()], false);
let mut function = Function::new("example", fn_type, Linkage::External).unwrap();

// Add parameters
let param = function.add_parameter(Some("x".to_string()), i32_type.clone()).unwrap();

// Create basic blocks
let mut entry = BasicBlock::named("entry");
let val = Value::integer_constant(42, i32_type.clone());
let ret_inst = Instruction::ret(Some(val));
entry.add_instruction(ret_inst);

function.add_basic_block(entry);
```

### Creating and Using Instructions

```rust
// Binary operation
let add_inst = Instruction::binary_op(
    Operation::Add,
    val1.clone(),
    val2.clone(),
    i32_type.clone()
);

// Comparison
let cond = Instruction::compare(Operation::Gt, val1.clone(), val2.clone());

// Control flow
let br_inst = Instruction::br(target_block.to_value());
let cond_br = Instruction::cond_br(
    condition.to_value(),
    then_block.to_value(),
    else_block.to_value()
);
```

### Working with Types

```rust
// Create array type
let array_type = Type::array(Type::i32(), 10);

// Create struct type
let field_types = vec![Type::i32(), Type::boolean()];
let field_names = vec!["x".to_string(), "y".to_string()];
let struct_type = Type::structure("Point", field_types, field_names);

// Create pointer type
let ptr_type = Type::pointer(Type::i32());
```

### Working with Modules

```rust
// Create a new module
let context = Context::new();
let mut module = Module::new(context.clone(), "example_module".to_string());
module.set_source_file("example.rs".to_string());

// Add a function to the module
let function = create_function();
module.add_function(function).unwrap();

// Verify the entire module
assert!(module.verify().is_ok());
```

## Best Practices

1. **Memory Management**
   - Use Arc for type sharing
   - Clone values when needed for multiple uses
   - Clean up basic blocks when removing from functions

2. **Error Handling**
   - Check function verification before use
   - Handle all error cases from function and instruction creation
   - Validate types match before operations
   - Use appropriate error types for different failure cases
   - Provide detailed error messages

3. **Type Safety**
   - Always verify type compatibility in operations
   - Use proper type conversions when needed
   - Maintain SSA form in basic blocks
   - Check pointer types before dereferencing
   - Validate array indices and struct field access

4. **Metadata**
   - Use consistent metadata keys across the program
   - Document metadata conventions
   - Don't rely on metadata for critical program logic
   - Consider metadata scope (function, block, instruction)

5. **Module Organization**
   - Group related functions together
   - Use meaningful names for functions and blocks
   - Maintain clear ownership of values
   - Document module-level invariants
   - Keep modules focused and cohesive

## Implementation Notes

- The IR uses Rust's type system to enforce correctness
- Values maintain unique IDs for tracking in the IR
- Arc is used for efficient type sharing
- Basic blocks maintain proper terminator invariants
- Functions enforce parameter and block consistency
- Modules provide a container for program organization
- Error handling is comprehensive and type-safe

## Overview

Hades IR is a compiler infrastructure that sits between high-level programming languages and the Hades VM. It provides a rich intermediate representation (IR) that allows for:

- Sophisticated type checking and type inference
- Powerful optimizations
- Platform-independent code representation
- Code generation for the Hades VM

The design is inspired by LLVM, but tailored specifically for the Hades VM architecture.

## Core Components

- **Context**: The global environment for IR construction and compilation
- **Module**: The top-level container for IR code (similar to an LLVM module)
- **Function**: Represents a function with basic blocks and parameters
- **BasicBlock**: A sequence of instructions with a single entry and exit point
- **Instruction**: An IR-level instruction that will be lowered to VM bytecode
- **Type**: A rich type system for IR-level type checking
- **Value**: A typed value in the IR, representing constants, variables, etc.
- **Builder**: A helper for constructing IR programmatically
- **Pass**: Transformations and optimizations that can be applied to the IR
- **BytecodeGenerator**: Converts IR to Hades VM bytecode

## Example

Here's a simple example that generates bytecode for adding two integers:

```rust
use hades_ir::{
    context::Context,
    types::Type,
    value::Value,
    bytecode::BytecodeGenerator,
};

// Create a new IR context
let context = Context::new();

// Create constants in the IR
let i32_type = Type::i32();
let const_3 = Value::integer_constant(3, i32_type.clone());
let const_4 = Value::integer_constant(4, i32_type);

// Create a bytecode generator
let mut gen = BytecodeGenerator::new();

// Generate bytecode for adding constants and printing the result
gen.gen_constant(&const_3)?;
gen.gen_constant(&const_4)?;
gen.gen_add()?;
gen.gen_print()?;

// Get the final bytecode
let bytecode = gen.get_bytecode();
```

When run on the Hades VM, this will push 3 and 4 onto the stack, add them, and print the result (7).

## Future Development

This IR infrastructure is intended to grow along with the Hades VM and support the development of high-level programming languages:

1. **Module System**: Full support for modules, imports, and exports
2. **Optimization Passes**: Common subexpression elimination, constant folding, etc.
3. **Advanced Types**: Support for user-defined types, generics, etc.
4. **Serialization**: Save and load IR for separate compilation
5. **Verification**: Ensures IR is well-formed and type-safe

## Usage with High-Level Languages

To use the Hades IR for a new programming language:

1. Implement a parser for your language's syntax
2. Create an AST (Abstract Syntax Tree) from the parsed code
3. Implement a frontend that converts your AST to Hades IR
4. Apply desired optimizations to the IR
5. Generate Hades VM bytecode from the optimized IR
6. Execute the bytecode on the Hades VM

## License

This project is part of the Hades VM ecosystem, with the same licensing terms. 