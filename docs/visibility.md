# Extended Visibility Modifiers in Nyx

This document describes the extended visibility system implemented in the Nyx programming language, providing fine-grained access control for modules, functions, and other language items.

## Overview

Nyx supports multiple visibility levels that control how items (functions, modules, structs, etc.) can be accessed from different parts of your codebase. The visibility system is inspired by Rust's visibility model but adapted for Nyx's specific needs.

## Visibility Modifiers

### 1. `pub` - Public Visibility
Items marked as `pub` are visible from anywhere in the codebase and to external consumers.

```nyx
pub fun public_function(): Int {
    return 42
}

pub mod public_module {
    // Contents are accessible based on their own visibility
}
```

### 2. `private` - Private Visibility (Default)
Items without an explicit visibility modifier are private by default. They are only accessible within the same module.

```nyx
fun private_function(): Int {  // Implicitly private
    return 42
}

mod private_module {  // Implicitly private
    // Only accessible within the current module
}
```

### 3. `internal` - Package/Crate Visibility
Items marked as `internal` are visible within the current package/crate but not to external consumers.

```nyx
internal fun package_function(): Int {
    return 42
}

internal mod package_module {
    // Accessible throughout the package
}
```

### 4. `protected` - Protected Visibility
Items marked as `protected` are visible to the current module and all its submodules.

```nyx
protected fun base_function(): Int {
    return 42
}

protected mod base_module {
    // Accessible to submodules
}
```

### 5. `package` - Package-Level Visibility
Similar to `internal`, but with specific package-level semantics.

```nyx
package fun utility_function(): Int {
    return 42
}

package mod utilities {
    // Package-level access
}
```

### 6. Restricted Visibility - `pub(restriction)`
Provides fine-grained control over visibility scope.

#### `pub(crate)` - Crate-wide Visibility
```nyx
pub(crate) fun crate_function(): Int {
    return 42
}
```

#### `pub(super)` - Parent Module Visibility
```nyx
pub(super) fun parent_visible(): Int {
    return 42
}
```

#### `pub(self)` - Current Module Only
```nyx
pub(self) fun module_only(): Int {
    return 42
}
```

#### `pub(in path::to::module)` - Specific Path Visibility
```nyx
pub(in core.math) fun math_specific(): Int {
    return 42
}
```

## Visibility Rules

### Access Control Matrix

| Visibility | Same Module | Submodule | Parent Module | Package | External |
|------------|-------------|-----------|---------------|---------|----------|
| `pub` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `private` | ✅ | ❌ | ❌ | ❌ | ❌ |
| `internal` | ✅ | ✅ | ✅ | ✅ | ❌ |
| `protected` | ✅ | ✅ | ❌* | ❌* | ❌ |
| `package` | ✅ | ✅ | ✅ | ✅ | ❌ |
| `pub(crate)` | ✅ | ✅ | ✅ | ✅ | ❌ |
| `pub(super)` | ✅ | ❌ | ✅ | ❌ | ❌ |
| `pub(self)` | ✅ | ❌ | ❌ | ❌ | ❌ |

*Protected items are accessible to submodules but not necessarily to parent modules.

### Module Hierarchy Example

```nyx
// Root module
pub mod app {
    internal mod core {
        protected fun base_logic(): Int {
            return 1
        }
        
        pub mod math {
            // Can access base_logic (protected from parent)
            pub fun calculate(): Int {
                return base_logic() * 2
            }
            
            pub(super) fun internal_calc(): Int {
                return 10
            }
        }
        
        pub mod utils {
            // Can access base_logic (protected from parent)
            internal fun helper(): Int {
                return base_logic() + 5
            }
            
            // Can access internal_calc from sibling module
            // Only if they share the same parent scope
        }
    }
    
    pub mod ui {
        // Cannot access core.base_logic (protected in different branch)
        // Can access core.math.calculate (public)
        pub fun display(): Int {
            return core.math.calculate()
        }
    }
}
```

## Implementation Details

### Parser Support
The Nyx parser recognizes all visibility modifiers and constructs appropriate AST nodes:

```rust
// AST representation
pub enum Visibility {
    Public,
    Private,
    Internal,
    Protected,
    Package,
    Restricted { restriction: VisibilityRestriction },
}

pub enum VisibilityRestriction {
    Crate,
    Super,
    Module,
    Path(ModulePath),
}
```

### Visibility Checking
The `VisibilityChecker` component validates access permissions:

```rust
let checker = VisibilityChecker::new(current_module, package_root);
let is_accessible = checker.is_accessible(&item_visibility, &item_module_path);
```

## Best Practices

### 1. Start with Private, Expose as Needed
Begin with private visibility and only make items public when necessary:

```nyx
mod my_module {
    // Private by default - good starting point
    fun internal_helper(): Int { return 42 }
    
    // Expose only what's needed
    pub fun public_api(): Int {
        return internal_helper()
    }
}
```

### 2. Use `internal` for Package APIs
For items that should be available throughout your package but not externally:

```nyx
internal mod config {
    internal fun load_settings(): Settings { ... }
}
```

### 3. Use `protected` for Extensible Hierarchies
When building modular systems that can be extended:

```nyx
protected mod base {
    protected fun common_functionality(): Int { ... }
}

mod extended {
    // Can access and build upon base functionality
    pub fun enhanced_feature(): Int {
        return base.common_functionality() + 10
    }
}
```

### 4. Use Restricted Visibility for Fine Control
When you need precise control over access:

```nyx
pub(crate) fun testing_utility(): Int { ... }  // Available for tests
pub(super) fun callback(): Int { ... }         // Only for parent module
pub(in core.math) fun math_helper(): Int { ... }  // Specific module only
```

## Compilation and Error Handling

The compiler enforces visibility rules at compile time:

```nyx
mod a {
    fun private_func(): Int { return 1 }
}

mod b {
    pub fun test(): Int {
        return a.private_func()  // ERROR: private_func is not accessible
    }
}
```

Error messages provide clear information about visibility violations:
```
Error: Cannot access private function 'private_func' from module 'b'
  --> src/main.nyx:8:16
   |
8  |         return a.private_func()
   |                ^^^^^^^^^^^^^^^^ private function not accessible
   |
   = help: Consider making the function 'pub' or 'internal' if it should be accessible
```

## Migration Guide

### From Basic to Extended Visibility

1. **Audit existing code**: Review current `pub` and private items
2. **Identify package boundaries**: Use `internal` for package-internal APIs
3. **Establish module hierarchies**: Use `protected` for extensible systems
4. **Apply restricted visibility**: Use `pub(...)` for fine-grained control

### Common Patterns

```nyx
// Before: Too permissive
pub mod utils {
    pub fun helper(): Int { ... }
}

// After: More precise
internal mod utils {
    pub(crate) fun helper(): Int { ... }  // Available in crate only
}
```

This extended visibility system provides the flexibility needed for large-scale Nyx applications while maintaining clear access control and encapsulation principles. 