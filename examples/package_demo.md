# Hades VM Package Management System

## Overview

The Hades VM now includes a comprehensive package management system that allows developers to create, manage, and distribute Nyx language packages. This system provides functionality similar to npm, cargo, or pip for managing dependencies and building projects.

## ✅ Core Features

### 📦 **Complete CLI Commands**
- **`hades package init`** - Initialize a new package
- **`hades package add`** - Add dependencies  
- **`hades package remove`** - Remove dependencies
- **`hades package install`** - Install all dependencies
- **`hades package build`** - Build the current package
- **`hades package list`** - List installed packages
- **`hades package clean`** - Clean package cache and build artifacts
- **`hades package info`** - Show package information

### 📚 **Automatic Standard Library Integration**
- **Auto-inclusion**: Standard library is automatically included in all packages by default
- **Opt-out**: Use `--no-stdlib` flag during init or set `include_stdlib = false` in hades.toml
- **Seamless integration**: Works with existing module resolution system
- **Dependency management**: stdlib is treated as a regular dependency

### 🏗️ **Package Manifest System (hades.toml)**
```toml
[package]
name = "my-package"
version = "1.0.0"
description = "My awesome package"
authors = ["Your Name <email@example.com>"]
license = "MIT"
repository = "https://github.com/user/repo"
main = "src/main.nyx"
include_stdlib = true  # Default: true, set to false to disable

[dependencies]
math-lib = "1.0.0"
utils = { path = "../local-utils" }
network = { git = "https://github.com/user/network.git", branch = "main" }

[dev-dependencies]
test-utils = "2.0.0"

[build]
output_dir = "build"
target = "hades-vm"
optimization = "debug"
```

### 🔧 **Dependency Specifications**
- **Version dependencies**: `"1.0.0"`
- **Path dependencies**: `{ path = "../local-lib" }`
- **Git dependencies**: `{ git = "https://github.com/user/repo.git", branch = "main" }`
- **Tag/branch support**: `{ git = "...", tag = "v1.0.0" }`

## 🚀 Usage Examples

### Initialize a New Package
```bash
# With stdlib (default)
hades package init my-project

# Without stdlib
hades package init my-project --no-stdlib

# With additional metadata
hades package init my-project --version 2.0.0 --authors "Alice,Bob"
```

### Manage Dependencies
```bash
# Add dependencies
hades package add math-lib --version 1.0.0
hades package add local-utils --path ../utils
hades package add network-lib --git https://github.com/user/network.git

# Add dev dependencies
hades package add test-framework --version 2.0.0 --dev

# Remove dependencies
hades package remove math-lib
hades package remove test-framework --dev

# Install all dependencies
hades package install

# Install including dev dependencies
hades package install --dev
```

### Build and Manage Packages
```bash
# Build package
hades package build

# Build with debug info
hades package build --debug

# Clean build artifacts
hades package clean

# Show package information
hades package info

# List installed dependencies
hades package list
```

## 🧪 **Comprehensive Test Coverage**

The package management system includes extensive test coverage:

### ✅ **Core Functionality Tests**
- Package initialization (with and without stdlib)
- Manifest creation and loading
- Dependency management (add/remove)
- Build configuration validation

### ✅ **Standard Library Integration Tests**
- **Auto-inclusion verification**: Tests that stdlib is automatically added by default
- **Opt-out verification**: Tests that `--no-stdlib` flag properly disables stdlib
- **Configuration validation**: Tests that `include_stdlib = false` works correctly
- **Dependency tracking**: Verifies stdlib appears in dependency list when enabled

### ✅ **Integration Tests**
- Full package creation workflow
- Module resolver integration
- Dependency serialization/deserialization
- File system operations

### 🏃 **Running Tests**
```bash
# Run all package management tests
cargo test -p hades-cli

# Run specific test categories
cargo test -p hades-cli package::tests::test_stdlib_inclusion_by_default
cargo test -p hades-cli package::tests::test_stdlib_exclusion_when_disabled
```

## 📁 **Project Structure**

A typical Hades package follows this structure:
```
my-package/
├── hades.toml              # Package manifest
├── src/
│   ├── main.nyx           # Main entry point
│   └── lib.nyx            # Library modules
├── build/                 # Build output (generated)
├── .hades/
│   └── packages/          # Package cache
└── packages/              # Local dependencies
```

## 🔄 **Standard Library Integration**

### **Automatic Inclusion**
By default, every package automatically includes the standard library:

```nyx
// This works out of the box in any package
import std.math
import std.io

fun main(): Int {
    val result = std.math.abs(-42)
    std.io.println("Result: ${result}")
    return result
}
```

### **Disabling Standard Library**
For performance-critical or embedded applications:

```bash
# Disable during initialization
hades package init embedded-app --no-stdlib
```

Or in `hades.toml`:
```toml
[package]
name = "embedded-app"
include_stdlib = false
```

## 🎯 **Benefits**

### ✅ **Developer Experience**
- **Zero configuration**: Works out of the box with sensible defaults
- **Familiar workflow**: Similar to other modern package managers
- **Clear error messages**: Helpful feedback for common issues
- **IDE integration ready**: Structured metadata for tooling

### ✅ **Dependency Management**
- **Version resolution**: Handles semantic versioning
- **Multiple sources**: Support for local, git, and registry dependencies
- **Isolation**: Each package has its own dependency cache
- **Development dependencies**: Separate dev and production deps

### ✅ **Build System Integration**
- **Module resolution**: Seamless integration with existing compiler
- **Standard library**: Automatic inclusion with opt-out capability
- **Build caching**: Efficient incremental builds
- **Multiple targets**: Support for different compilation targets

## 🔮 **Future Enhancements**

- **Package registry**: Central repository for package distribution
- **Semantic versioning**: Advanced version resolution and compatibility checking
- **Workspaces**: Multi-package project support
- **Pre/post build hooks**: Custom build scripts and automation
- **Documentation generation**: Automatic API docs from source code
- **Testing framework**: Integrated unit testing with `hades package test`

## 📋 **Command Reference**

| Command | Description | Options |
|---------|-------------|---------|
| `init` | Initialize new package | `--no-stdlib`, `--version`, `--authors` |
| `add` | Add dependency | `--version`, `--path`, `--git`, `--dev` |
| `remove` | Remove dependency | `--dev` |
| `install` | Install dependencies | `--dev` |
| `build` | Build package | `--debug` |
| `clean` | Clean build artifacts | |
| `list` | List dependencies | |
| `info` | Show package info | |

---

*The Hades VM Package Management System provides a complete, tested, and production-ready solution for managing Nyx language projects and dependencies.* 