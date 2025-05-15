use std::sync::Arc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::error::Result;

/// The Context is the main container for an IR compilation session.
/// It manages global data like type definitions and interned strings,
/// and serves as a factory for creating IR elements.
pub struct Context {
    /// Internal state of the context
    state: Arc<RefCell<ContextState>>,
}

/// Internal state of the Context, held behind Rc<RefCell<>> to allow
/// sharing and mutation from different places while preventing
/// multiple simultaneous borrows.
struct ContextState {
    /// String interning table - reduces memory usage by storing each unique string once
    string_table: HashMap<String, usize>,
    /// Next string ID
    next_string_id: usize,
    /// Type table - stores each unique type definition
    type_table: HashMap<usize, TypeEntry>,
    /// Next type ID
    next_type_id: usize,
}

/// A type in the type table
struct TypeEntry {
    /// Unique ID
    id: usize,
    /// Name of the type
    name: String,
    /// Size in bytes
    size: usize,
    /// Alignment in bytes
    alignment: usize,
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        let state = ContextState {
            string_table: HashMap::new(),
            next_string_id: 0,
            type_table: HashMap::new(),
            next_type_id: 0,
        };

        Self {
            state: Arc::new(RefCell::new(state)),
        }
    }

    /// Create a new module in this context
    pub fn create_module(&self, name: &str) -> Result<()> {
        // For now, just intern the module name to verify the context is working
        let _module_name_id = self.intern_string(name);
        
        // The actual Module type will be implemented later
        Ok(())
    }

    /// Intern a string, returning a unique ID for it
    pub fn intern_string(&self, s: &str) -> usize {
        let mut state = self.state.borrow_mut();
        if let Some(&id) = state.string_table.get(s) {
            return id;
        }

        let id = state.next_string_id;
        state.next_string_id += 1;
        state.string_table.insert(s.to_string(), id);
        id
    }

    /// Get a string from its interned ID
    pub fn get_string(&self, id: usize) -> Option<String> {
        let state = self.state.borrow();
        for (s, &s_id) in state.string_table.iter() {
            if s_id == id {
                return Some(s.clone());
            }
        }
        None
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
} 