# Hades IR

An LLVM-like intermediate representation for compiling higher-level languages to Hades VM bytecode.

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