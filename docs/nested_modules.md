# Nested Modules & Module Hierarchies

This document describes the implementation of nested modules and module hierarchies in the Hades VM project.

## Overview

The Hades VM now supports a comprehensive module system with the following features:

- **Hierarchical Module Structure**: Modules can contain submodules, forming tree-like hierarchies
- **Multiple Visibility Levels**: Public, Internal, Protected, Private, and Restricted visibility
- **File-based and Inline Modules**: Support for both file-based module organization and inline module declarations
- **Qualified Name Resolution**: Access items using paths like `std::collections::vector::new()`
- **Module Path Operations**: Rich API for working with module paths and relationships

## Module Visibility Levels

### 1. Public (`pub`)
Accessible from anywhere, including external crates.

```nyx
pub mod math {
    pub fun add(a: Int, b: Int): Int {
        return a + b
    }
}
```

### 2. Internal (`internal`)
Accessible only within the same crate/package.

```nyx
internal mod database {
    internal fun connect(): Bool {
        return true
    }
}
```

### 3. Protected (`protected`)
Accessible from the parent module and all its descendants.

```nyx
pub mod ui {
    protected fun shared_utility(): Int {
        return 42
    }
    
    pub mod components {
        pub fun button(): Int {
            // Can access parent's protected function
            return super::shared_utility()
        }
    }
}
```

### 4. Private (default)
Only accessible within the defining module.

```nyx
mod internal_helpers {
    fun secret_function(): Int {  // Private by default
        return 123
    }
}
```

### 5. Restricted Visibility
Fine-grained visibility control with specific path restrictions.

```nyx
pub(crate) mod crate_internal {
    pub(super) fun parent_only(): Int {
        return 1
    }
    
    pub(in math::geometry) fun specific_path(): Int {
        return 2
    }
}
```

## Module Hierarchy Examples

### Inline Module Declarations

```nyx
pub mod math {
    pub fun add(a: Int, b: Int): Int {
        return a + b
    }
    
    pub mod geometry {
        pub fun area_rectangle(w: Int, h: Int): Int {
            return super::add(w, h)  // Access parent function
        }
        
        pub mod shapes {
            pub fun triangle_area(base: Int, height: Int): Int {
                let rect_area = super::area_rectangle(base, height)
                return super::super::divide_helper(rect_area, 2)
            }
            
            mod internal_calc {
                fun helper(): Int {
                    return 42
                }
            }
        }
    }
    
    pub mod statistics {
        pub fun average(values: [Int]): Float {
            return 0.0
        }
    }
}

fun main(): Int {
    let sum = math::add(10, 20)
    let area = math::geometry::area_rectangle(5, 4)
    let tri_area = math::geometry::shapes::triangle_area(6, 8)
    
    return sum + area + tri_area
}
```

### File-based Module Organization

```
project/
├── src/
│   ├── main.nyx
│   └── math/
│       ├── mod.nyx              // Main math module
│       ├── geometry.nyx         // math::geometry
│       ├── statistics.nyx       // math::statistics
│       └── geometry/
│           ├── shapes.nyx       // math::geometry::shapes
│           └── advanced.nyx     // math::geometry::advanced
```

**math/mod.nyx:**
```nyx
pub fun add(a: Int, b: Int): Int {
    return a + b
}

protected fun multiply_helper(a: Int, b: Int): Int {
    return a * b
}
```

**math/geometry.nyx:**
```nyx
pub fun area_rectangle(width: Int, height: Int): Int {
    return super::multiply_helper(width, height)
}

pub fun perimeter_rectangle(width: Int, height: Int): Int {
    let doubled_width = super::multiply_helper(width, 2)
    let doubled_height = super::multiply_helper(height, 2)
    return super::add(doubled_width, doubled_height)
}
```

**math/geometry/shapes.nyx:**
```nyx
pub fun triangle_area(base: Int, height: Int): Int {
    let rect_area = super::area_rectangle(base, height)
    return super::super::divide_helper(rect_area, 2)
}

pub fun complex_calculation(): Int {
    let rect = super::area_rectangle(10, 5)
    let tri = triangle_area(10, 5)
    return super::super::add(rect, tri)
}
```

## Implementation Details

### IR Module Structure

The `hades-ir` crate provides the `Module` struct with hierarchical support:

```rust
pub struct Module {
    path: ModulePath,                    // Hierarchical path
    submodules: HashMap<String, Module>, // Child modules
    parent_path: Option<ModulePath>,     // Parent reference
    visibility: ModuleVisibility,        // Access control
    functions: HashMap<String, Function>,
    global_variables: HashMap<String, GlobalVariable>,
    // ... other fields
}
```

Key methods:
- `create_submodule()`: Add a new submodule
- `find_module()`: Recursive module lookup
- `is_accessible_from()`: Visibility checking
- `get_all_modules()`: Depth-first traversal

### Module Path Operations

The `ModulePath` struct supports rich path operations:

```rust
pub struct ModulePath {
    pub segments: Vec<String>,
}

impl ModulePath {
    pub fn append(&self, segment: String) -> Self
    pub fn parent(&self) -> Option<Self>
    pub fn is_parent_of(&self, other: &ModulePath) -> bool
    pub fn is_child_of(&self, other: &ModulePath) -> bool
    pub fn is_ancestor_of(&self, other: &ModulePath) -> bool
}
```

### Module Resolution

The `ModuleResolver` handles loading and caching of modules:

```rust
impl ModuleResolver {
    pub fn load_module(&mut self, path: &ModulePath) -> ParseResult<ResolvedModule>
    pub fn find_nested_modules(&self, base_path: &Path) -> Vec<(String, PathBuf)>
    pub fn get_module_tree(&self) -> HashMap<ModulePath, Vec<ModulePath>>
    pub fn is_module_accessible(&mut self, path: &ModulePath, context: &ModulePath) -> bool
}
```

## Usage Examples

### Qualified Function Calls

```nyx
// Using :: syntax (preferred)
let result1 = std::collections::vector::new()
let result2 = math::geometry::shapes::triangle_area(10, 5)

// Using . syntax (alternative)
let result3 = std.collections.vector.length()
let result4 = math.geometry.area_rectangle(8, 6)
```

### Importing from Nested Modules

```nyx
// Import specific functions
import std::collections::vector::{new, push, pop}
import math::geometry::shapes::triangle_area

// Import entire modules
import std::collections::vector
import math::geometry

// Import with aliases
import std::collections::vector as vec
import math::geometry::shapes as geom

fun main(): Int {
    let v = vec::new()
    let area = geom::triangle_area(5, 3)
    return area
}
```

### Module Re-exports

```nyx
pub mod utils {
    // Re-export from nested modules
    pub use math::add
    pub use math::geometry::area_rectangle as area
    
    // Re-export entire submodules
    pub use string_processing::*
    
    // Create module aliases
    pub mod geom = math::geometry
}

fun test(): Int {
    let sum = utils::add(5, 3)
    let area = utils::area(10, 20)
    let perimeter = utils::geom::perimeter_rectangle(10, 20)
    
    return sum + area + perimeter
}
```

## Advanced Features

### Super References

Access parent module items using `super::`:

```nyx
pub mod parent {
    pub fun parent_function(): Int { return 1 }
    
    pub mod child {
        pub fun child_function(): Int {
            return super::parent_function() + 1
        }
        
        pub mod grandchild {
            pub fun grandchild_function(): Int {
                // Access grandparent
                return super::super::parent_function() + 2
            }
        }
    }
}
```

### Conditional Module Compilation

```nyx
#[cfg(feature = "advanced")]
pub mod advanced_features {
    pub fun experimental_function(): Int {
        return 42
    }
}

#[cfg(debug)]
mod debug_utils {
    pub fun debug_print(msg: String) {
        // Debug-only functionality
    }
}
```

### Module Documentation

```nyx
/// Math utilities module
/// 
/// This module provides basic mathematical operations
/// and geometric calculations.
pub mod math {
    /// Adds two integers
    /// 
    /// # Arguments
    /// * `a` - First integer
    /// * `b` - Second integer
    /// 
    /// # Returns
    /// The sum of a and b
    pub fun add(a: Int, b: Int): Int {
        return a + b
    }
}
```

## Best Practices

### 1. Module Organization
- Use file-based modules for large codebases
- Group related functionality in the same module
- Keep module nesting depth reasonable (≤ 4 levels)

### 2. Visibility Design
- Default to private, expose only what's necessary
- Use `internal` for crate-wide utilities
- Use `protected` for extensible module hierarchies
- Reserve `public` for true public APIs

### 3. Naming Conventions
- Use snake_case for module names
- Use descriptive, hierarchical names
- Avoid deep nesting with generic names

### 4. Import Organization
- Group imports by scope (std, external, local)
- Use specific imports over wildcard imports
- Prefer qualified calls for disambiguation

## Testing Module Hierarchies

The implementation includes comprehensive tests covering:

- Module path operations and relationships
- Visibility checking across different contexts
- Recursive module loading and caching
- Nested module parsing and AST generation
- End-to-end compilation with module resolution

Run tests with:
```bash
cargo test test_nested_module_parsing
cargo test test_module_path_operations
cargo test test_ir_module_hierarchy
cargo test test_module_visibility
```

## Future Enhancements

Planned improvements include:
- Macro support across module boundaries
- Trait and type definitions in modules
- Module-level constants and static variables
- Advanced module attributes and metadata
- Performance optimizations for large module trees
- IDE support for module navigation and refactoring 