use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::{
    context::Context,
    error::{Error, Result},
    function::{Function, Linkage},
    types::Type,
    value::{Value, ValueId},
};

/// A global variable in the module
#[derive(Debug, Clone)]
pub struct GlobalVariable {
    /// The ID of this global variable as a value
    id: ValueId,
    /// The name of this global variable
    name: String,
    /// The type of this global variable
    ty: Arc<Type>,
    /// Whether this global variable is constant
    is_constant: bool,
    /// The initial value of this global variable, if any
    initializer: Option<Value>,
    /// The linkage type of this global variable
    linkage: Linkage,
    /// Alignment of this global variable in bytes
    alignment: usize,
}

impl GlobalVariable {
    /// Create a new global variable
    pub fn new(
        name: String,
        ty: Arc<Type>,
        is_constant: bool,
        initializer: Option<Value>,
        linkage: Linkage,
        alignment: usize,
    ) -> Self {
        Self {
            id: ValueId::new(),
            name,
            ty,
            is_constant,
            initializer,
            linkage,
            alignment,
        }
    }
    
    /// Get the ID of this global variable
    pub fn id(&self) -> ValueId {
        self.id
    }
    
    /// Get the name of this global variable
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the type of this global variable
    pub fn ty(&self) -> Arc<Type> {
        self.ty.clone()
    }
    
    /// Check if this global variable is constant
    pub fn is_constant(&self) -> bool {
        self.is_constant
    }
    
    /// Get the initializer of this global variable
    pub fn initializer(&self) -> Option<&Value> {
        self.initializer.as_ref()
    }
    
    /// Set the initializer of this global variable
    pub fn set_initializer(&mut self, initializer: Value) {
        self.initializer = Some(initializer);
    }
    
    /// Get the linkage type of this global variable
    pub fn linkage(&self) -> Linkage {
        self.linkage
    }
    
    /// Set the linkage type of this global variable
    pub fn set_linkage(&mut self, linkage: Linkage) {
        self.linkage = linkage;
    }
    
    /// Get the alignment of this global variable
    pub fn alignment(&self) -> usize {
        self.alignment
    }
    
    /// Set the alignment of this global variable
    pub fn set_alignment(&mut self, alignment: usize) {
        self.alignment = alignment;
    }
    
    /// Convert this global variable to a value
    pub fn to_value(&self) -> Value {
        Value::GlobalVariable {
            id: self.id,
            name: self.name.clone(),
            ty: self.ty.clone(),
        }
    }
}

impl fmt::Display for GlobalVariable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "@{} = ", self.name)?;
        
        // Write linkage
        match self.linkage {
            Linkage::External => {} // Default, no prefix needed
            Linkage::Internal => write!(f, "internal ")?,
            Linkage::InlineOnly => write!(f, "inlinehint ")?,
            Linkage::LinkOnceODR => write!(f, "linkonce_odr ")?,
        }
        
        if self.is_constant {
            write!(f, "constant ")?;
        } else {
            write!(f, "global ")?;
        }
        
        write!(f, "{}", self.ty)?;
        
        if let Some(initializer) = &self.initializer {
            write!(f, " {}", initializer)?;
        } else {
            write!(f, " zeroinitializer")?;
        }
        
        if self.alignment > 0 {
            write!(f, ", align {}", self.alignment)?;
        }
        
        Ok(())
    }
}

/// Module visibility levels for controlling access
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleVisibility {
    /// Publicly visible to all modules and external code
    Public,
    /// Only visible within the current package/crate
    Internal,
    /// Only visible within the parent module and its submodules
    Protected,
    /// Private to the defining module
    Private,
}

/// A module path representing the hierarchical location of a module
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePath {
    /// The path segments (e.g., ["std", "collections", "vector"])
    pub segments: Vec<String>,
}

impl ModulePath {
    /// Create a new module path from segments
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }
    
    /// Create a root module path
    pub fn root() -> Self {
        Self { segments: vec![] }
    }
    
    /// Create a single-segment module path
    pub fn single(name: String) -> Self {
        Self { segments: vec![name] }
    }
    
    /// Append a segment to this path
    pub fn append(&self, segment: String) -> Self {
        let mut new_segments = self.segments.clone();
        new_segments.push(segment);
        Self { segments: new_segments }
    }
    
    /// Get the parent path (all segments except the last)
    pub fn parent(&self) -> Option<Self> {
        if self.segments.is_empty() {
            None
        } else {
            Some(Self { segments: self.segments[..self.segments.len() - 1].to_vec() })
        }
    }
    
    /// Get the module name (last segment)
    pub fn name(&self) -> Option<&str> {
        self.segments.last().map(|s| s.as_str())
    }
    
    /// Check if this path is a parent of the other path
    pub fn is_parent_of(&self, other: &ModulePath) -> bool {
        if self.segments.len() >= other.segments.len() {
            return false;
        }
        
        for (i, segment) in self.segments.iter().enumerate() {
            if other.segments.get(i) != Some(segment) {
                return false;
            }
        }
        
        true
    }
    
    /// Check if this path is a child of the other path
    pub fn is_child_of(&self, other: &ModulePath) -> bool {
        other.is_parent_of(self)
    }
    
    /// Check if this path is an ancestor of the other path
    pub fn is_ancestor_of(&self, other: &ModulePath) -> bool {
        self.is_parent_of(other) || self.segments == other.segments[..self.segments.len().min(other.segments.len())]
    }
}

impl fmt::Display for ModulePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.segments.join("::"))
    }
}

/// A module in the IR, representing a compilation unit with hierarchical structure.
#[derive(Debug, Clone)]
pub struct Module {
    /// The context this module belongs to
    context: Arc<Context>,
    /// The hierarchical path of this module
    path: ModulePath,
    /// The source file this module was compiled from, if any
    source_file: Option<String>,
    /// The functions in this module
    functions: HashMap<String, Function>,
    /// The global variables in this module
    global_variables: HashMap<String, GlobalVariable>,
    /// Child submodules
    submodules: HashMap<String, Module>,
    /// Module visibility
    visibility: ModuleVisibility,
    /// The target triple for this module
    target_triple: Option<String>,
    /// The data layout for this module
    data_layout: Option<String>,
    /// Parent module reference (weak to avoid cycles)
    parent_path: Option<ModulePath>,
}

impl Module {
    /// Create a new module with the given path
    pub fn new(context: Arc<Context>, path: ModulePath) -> Self {
        Self {
            context,
            path,
            source_file: None,
            functions: HashMap::new(),
            global_variables: HashMap::new(),
            submodules: HashMap::new(),
            visibility: ModuleVisibility::Public,
            target_triple: None,
            data_layout: None,
            parent_path: None,
        }
    }
    
    /// Create a new root module
    pub fn new_root(context: Arc<Context>, name: String) -> Self {
        Self::new(context, ModulePath::single(name))
    }
    
    /// Get the context this module belongs to
    pub fn context(&self) -> Arc<Context> {
        self.context.clone()
    }
    
    /// Get the full path of this module
    pub fn path(&self) -> &ModulePath {
        &self.path
    }
    
    /// Get the name of this module (last segment of path)
    pub fn name(&self) -> &str {
        self.path.name().unwrap_or("root")
    }
    
    /// Get the parent path of this module
    pub fn parent_path(&self) -> Option<&ModulePath> {
        self.parent_path.as_ref()
    }
    
    /// Set the parent path of this module
    pub fn set_parent_path(&mut self, parent: ModulePath) {
        self.parent_path = Some(parent);
    }
    
    /// Get the visibility of this module
    pub fn visibility(&self) -> &ModuleVisibility {
        &self.visibility
    }
    
    /// Set the visibility of this module
    pub fn set_visibility(&mut self, visibility: ModuleVisibility) {
        self.visibility = visibility;
    }
    
    /// Get the source file this module was compiled from
    pub fn source_file(&self) -> Option<&str> {
        self.source_file.as_deref()
    }
    
    /// Set the source file this module was compiled from
    pub fn set_source_file(&mut self, source_file: String) {
        self.source_file = Some(source_file);
    }
    
    /// Get the target triple for this module
    pub fn target_triple(&self) -> Option<&str> {
        self.target_triple.as_deref()
    }
    
    /// Set the target triple for this module
    pub fn set_target_triple(&mut self, target_triple: String) {
        self.target_triple = Some(target_triple);
    }
    
    /// Get the data layout for this module
    pub fn data_layout(&self) -> Option<&str> {
        self.data_layout.as_deref()
    }
    
    /// Set the data layout for this module
    pub fn set_data_layout(&mut self, data_layout: String) {
        self.data_layout = Some(data_layout);
    }
    
    /// Get the functions in this module
    pub fn functions(&self) -> &HashMap<String, Function> {
        &self.functions
    }
    
    /// Get mutable functions in this module
    pub fn functions_mut(&mut self) -> &mut HashMap<String, Function> {
        &mut self.functions
    }
    
    /// Add a function to this module
    pub fn add_function(&mut self, function: Function) -> Result<()> {
        let name = function.name().to_string();
        if self.functions.contains_key(&name) {
            return Err(Error::ConstructionError(format!(
                "Function {} already exists in module {}",
                name, self.path
            )));
        }
        
        self.functions.insert(name, function);
        Ok(())
    }
    
    /// Create a new function in this module
    pub fn create_function(
        &mut self,
        name: &str,
        return_type: Arc<Type>,
        param_types: Vec<Arc<Type>>,
        linkage: Linkage,
    ) -> Result<&mut Function> {
        // Check if a function with this name already exists
        if self.functions.contains_key(name) {
            return Err(Error::ConstructionError(format!(
                "Function {} already exists in module {}",
                name, self.path
            )));
        }
        
        // Create the function type
        let function_type = Type::function(return_type, param_types, false);
        
        // Create the function
        let function = Function::new(name.to_string(), function_type, linkage)?;
        
        // Add the function to the module
        self.functions.insert(name.to_string(), function);
        
        // Return a mutable reference to the function
        Ok(self.functions.get_mut(name).unwrap())
    }
    
    /// Get a function by name
    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.functions.get(name)
    }
    
    /// Get a mutable function by name
    pub fn get_function_mut(&mut self, name: &str) -> Option<&mut Function> {
        self.functions.get_mut(name)
    }
    
    /// Get the global variables in this module
    pub fn global_variables(&self) -> &HashMap<String, GlobalVariable> {
        &self.global_variables
    }
    
    /// Get mutable global variables in this module
    pub fn global_variables_mut(&mut self) -> &mut HashMap<String, GlobalVariable> {
        &mut self.global_variables
    }
    
    /// Add a global variable to this module
    pub fn add_global_variable(&mut self, global_variable: GlobalVariable) -> Result<()> {
        let name = global_variable.name().to_string();
        if self.global_variables.contains_key(&name) {
            return Err(Error::ConstructionError(format!(
                "Global variable {} already exists in module {}",
                name, self.path
            )));
        }
        
        self.global_variables.insert(name, global_variable);
        Ok(())
    }
    
    /// Create a new global variable in this module
    pub fn create_global_variable(
        &mut self,
        name: &str,
        ty: Arc<Type>,
        is_constant: bool,
        initializer: Option<Value>,
        linkage: Linkage,
        alignment: usize,
    ) -> Result<&mut GlobalVariable> {
        // Check if a global variable with this name already exists
        if self.global_variables.contains_key(name) {
            return Err(Error::ConstructionError(format!(
                "Global variable {} already exists in module {}",
                name, self.path
            )));
        }
        
        // Create the global variable
        let global_variable = GlobalVariable::new(
            name.to_string(),
            ty,
            is_constant,
            initializer,
            linkage,
            alignment,
        );
        
        // Add the global variable to the module
        self.global_variables.insert(name.to_string(), global_variable);
        
        // Return a mutable reference to the global variable
        Ok(self.global_variables.get_mut(name).unwrap())
    }
    
    /// Get a global variable by name
    pub fn get_global_variable(&self, name: &str) -> Option<&GlobalVariable> {
        self.global_variables.get(name)
    }
    
    /// Get a mutable global variable by name
    pub fn get_global_variable_mut(&mut self, name: &str) -> Option<&mut GlobalVariable> {
        self.global_variables.get_mut(name)
    }
    
    /// Get all submodules
    pub fn submodules(&self) -> &HashMap<String, Module> {
        &self.submodules
    }
    
    /// Get mutable submodules
    pub fn submodules_mut(&mut self) -> &mut HashMap<String, Module> {
        &mut self.submodules
    }
    
    /// Add a submodule to this module
    pub fn add_submodule(&mut self, mut submodule: Module) -> Result<()> {
        let name = submodule.name().to_string();
        
        if self.submodules.contains_key(&name) {
            return Err(Error::ConstructionError(format!(
                "Submodule {} already exists in module {}",
                name, self.path
            )));
        }
        
        // Set parent relationship
        submodule.set_parent_path(self.path.clone());
        
        // Update the submodule's path to be relative to this module
        let new_path = self.path.append(name.clone());
        submodule.path = new_path;
        
        self.submodules.insert(name, submodule);
        Ok(())
    }
    
    /// Create a new submodule
    pub fn create_submodule(&mut self, name: &str, visibility: ModuleVisibility) -> Result<&mut Module> {
        if self.submodules.contains_key(name) {
            return Err(Error::ConstructionError(format!(
                "Submodule {} already exists in module {}",
                name, self.path
            )));
        }
        
        let submodule_path = self.path.append(name.to_string());
        let mut submodule = Module::new(self.context.clone(), submodule_path);
        submodule.set_visibility(visibility);
        submodule.set_parent_path(self.path.clone());
        
        self.submodules.insert(name.to_string(), submodule);
        Ok(self.submodules.get_mut(name).unwrap())
    }
    
    /// Get a submodule by name
    pub fn get_submodule(&self, name: &str) -> Option<&Module> {
        self.submodules.get(name)
    }
    
    /// Get a mutable submodule by name
    pub fn get_submodule_mut(&mut self, name: &str) -> Option<&mut Module> {
        self.submodules.get_mut(name)
    }
    
    /// Find a module by path (recursive search through hierarchy)
    pub fn find_module(&self, path: &ModulePath) -> Option<&Module> {
        if path.segments.is_empty() {
            return Some(self);
        }
        
        // If this is the target path, return self
        if self.path == *path {
            return Some(self);
        }
        
        // Check if path starts with our path
        if path.segments.len() > self.path.segments.len() {
            let mut matches = true;
            for (i, segment) in self.path.segments.iter().enumerate() {
                if path.segments.get(i) != Some(segment) {
                    matches = false;
                    break;
                }
            }
            
            if matches {
                // This path is under our hierarchy, search submodules
                let next_segment = &path.segments[self.path.segments.len()];
                if let Some(submodule) = self.submodules.get(next_segment) {
                    return submodule.find_module(path);
                }
            }
        }
        
        None
    }
    
    /// Find a mutable module by path
    pub fn find_module_mut(&mut self, path: &ModulePath) -> Option<&mut Module> {
        if path.segments.is_empty() {
            return Some(self);
        }
        
        // If this is the target path, return self
        if self.path == *path {
            return Some(self);
        }
        
        // Check if path starts with our path
        if path.segments.len() > self.path.segments.len() {
            let mut matches = true;
            for (i, segment) in self.path.segments.iter().enumerate() {
                if path.segments.get(i) != Some(segment) {
                    matches = false;
                    break;
                }
            }
            
            if matches {
                // This path is under our hierarchy, search submodules
                let next_segment = &path.segments[self.path.segments.len()];
                if let Some(submodule) = self.submodules.get_mut(next_segment) {
                    return submodule.find_module_mut(path);
                }
            }
        }
        
        None
    }
    
    /// Get all modules in the hierarchy (depth-first traversal)
    pub fn get_all_modules(&self) -> Vec<&Module> {
        let mut modules = vec![self];
        
        for submodule in self.submodules.values() {
            modules.extend(submodule.get_all_modules());
        }
        
        modules
    }
    
    /// Check if this module is accessible from the given context
    pub fn is_accessible_from(&self, context_path: &ModulePath) -> bool {
        match &self.visibility {
            ModuleVisibility::Public => true,
            ModuleVisibility::Private => {
                // Only accessible from the same module or parent
                if let Some(parent) = &self.parent_path {
                    context_path == parent || context_path == &self.path
                } else {
                    context_path == &self.path
                }
            }
            ModuleVisibility::Protected => {
                // Accessible from parent and all its descendants
                if let Some(parent) = &self.parent_path {
                    context_path == parent || parent.is_ancestor_of(context_path) || context_path == &self.path
                } else {
                    context_path == &self.path
                }
            }
            ModuleVisibility::Internal => {
                // Accessible within the same top-level module/crate
                let context_root = context_path.segments.first();
                let self_root = self.path.segments.first();
                context_root == self_root
            }
        }
    }
    
    /// Verify that the module is well-formed
    pub fn verify(&self) -> Result<()> {
        // Verify all functions
        for (name, function) in &self.functions {
            function.verify().map_err(|e| {
                Error::ValidationError(format!(
                    "Function {} in module {} failed verification: {}",
                    name, self.path, e
                ))
            })?;
        }
        
        // Verify all submodules
        for (name, submodule) in &self.submodules {
            submodule.verify().map_err(|e| {
                Error::ValidationError(format!(
                    "Submodule {} in module {} failed verification: {}",
                    name, self.path, e
                ))
            })?;
        }
        
        Ok(())
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write module metadata
        writeln!(f, "; Module: {}", self.path)?;
        
        if let Some(source_file) = &self.source_file {
            writeln!(f, "; Source file: {}", source_file)?;
        }
        
        if let Some(target_triple) = &self.target_triple {
            writeln!(f, "target triple = \"{}\"", target_triple)?;
        }
        
        if let Some(data_layout) = &self.data_layout {
            writeln!(f, "target datalayout = \"{}\"", data_layout)?;
        }
        
        // Write global variables
        if !self.global_variables.is_empty() {
            writeln!(f)?;
            for global_variable in self.global_variables.values() {
                writeln!(f, "{}", global_variable)?;
            }
        }
        
        // Write functions
        if !self.functions.is_empty() {
            writeln!(f)?;
            for function in self.functions.values() {
                writeln!(f, "{}", function)?;
            }
        }
        
        // Write submodules
        if !self.submodules.is_empty() {
            writeln!(f)?;
            writeln!(f, "; Submodules:")?;
            for (name, submodule) in &self.submodules {
                writeln!(f, "; - {}: {}", name, submodule.path)?;
            }
        }
        
        Ok(())
    }
} 