# Nyx Compiler

A modern compiler implementation for the Nyx programming language, combining the best features of Kotlin and Rust.

## Features

- Modern syntax inspired by Kotlin
- Rust-like ownership system
- Pattern matching with `when` expressions
- First-class async/await support
- Trait-based polymorphism
- Generic type parameters
- Strong type system with null safety

## Project Structure

```
nyx-compiler/
├── src/
│   ├── lexer/      # Lexical analysis
│   ├── parser/     # Syntax analysis
│   └── ast/        # Abstract Syntax Tree definitions
```

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 