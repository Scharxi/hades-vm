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

/// A module in the IR, representing a compilation unit.
#[derive(Debug, Clone)]
pub struct Module {
    /// The context this module belongs to
    context: Arc<Context>,
    /// The name of this module
    name: String,
    /// The source file this module was compiled from, if any
    source_file: Option<String>,
    /// The functions in this module
    functions: HashMap<String, Function>,
    /// The global variables in this module
    global_variables: HashMap<String, GlobalVariable>,
    /// The target triple for this module
    target_triple: Option<String>,
    /// The data layout for this module
    data_layout: Option<String>,
}

impl Module {
    /// Create a new module with the given name
    pub fn new(context: Arc<Context>, name: String) -> Self {
        Self {
            context,
            name,
            source_file: None,
            functions: HashMap::new(),
            global_variables: HashMap::new(),
            target_triple: None,
            data_layout: None,
        }
    }
    
    /// Get the context this module belongs to
    pub fn context(&self) -> Arc<Context> {
        self.context.clone()
    }
    
    /// Get the name of this module
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Set the name of this module
    pub fn set_name(&mut self, name: String) {
        self.name = name;
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
                name, self.name
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
                name, self.name
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
                name, self.name
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
                name, self.name
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
    
    /// Verify that the module is well-formed
    pub fn verify(&self) -> Result<()> {
        // Verify all functions
        for (name, function) in &self.functions {
            function.verify().map_err(|e| {
                Error::ValidationError(format!(
                    "Function {} in module {} failed verification: {}",
                    name, self.name, e
                ))
            })?;
        }
        
        Ok(())
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write module metadata
        writeln!(f, "; Module: {}", self.name)?;
        
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
        
        Ok(())
    }
} 