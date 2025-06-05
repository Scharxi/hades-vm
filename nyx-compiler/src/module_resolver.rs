use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::{ModulePath, ModuleAwareProgram, Function, ModuleDecl, Visibility, VisibilityRestriction};
use crate::parser::{Parser, ParseResult, ParseError};

/// Represents a resolved module with its functions and submodules
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub name: String,
    pub path: PathBuf,
    pub module_path: ModulePath,
    pub functions: Vec<Function>,
    pub submodules: HashMap<String, ResolvedModule>,
    pub visibility: crate::ast::Visibility,
    pub parent_path: Option<ModulePath>,
}

impl ResolvedModule {
    /// Create a new resolved module
    pub fn new(name: String, path: PathBuf, module_path: ModulePath, visibility: Visibility) -> Self {
        Self {
            name,
            path,
            module_path,
            functions: Vec::new(),
            submodules: HashMap::new(),
            visibility,
            parent_path: None,
        }
    }
    
    /// Add a submodule to this resolved module
    pub fn add_submodule(&mut self, mut submodule: ResolvedModule) -> Result<(), String> {
        if self.submodules.contains_key(&submodule.name) {
            return Err(format!("Submodule {} already exists", submodule.name));
        }
        
        // Set parent relationship
        submodule.parent_path = Some(self.module_path.clone());
        
        // Update submodule path to be relative to this module
        submodule.module_path = self.module_path.append(submodule.name.clone());
        
        self.submodules.insert(submodule.name.clone(), submodule);
        Ok(())
    }
    
    /// Find a submodule by path (recursive search)
    pub fn find_module(&self, target_path: &ModulePath) -> Option<&ResolvedModule> {
        // If this is the target, return self
        if self.module_path == *target_path {
            return Some(self);
        }
        
        // Check if target is under this module's hierarchy
        if target_path.is_child_of(&self.module_path) {
            // Get the next segment in the path
            if let Some(next_segment) = target_path.segments.get(self.module_path.segments.len()) {
                if let Some(submodule) = self.submodules.get(next_segment) {
                    return submodule.find_module(target_path);
                }
            }
        }
        
        None
    }
    
    /// Get all functions in this module and its accessible submodules
    pub fn get_all_accessible_functions(&self, context_path: &ModulePath) -> Vec<&Function> {
        let mut functions = Vec::new();
        
        // Add own functions
        functions.extend(&self.functions);
        
        // Add functions from accessible submodules
        for submodule in self.submodules.values() {
            if submodule.is_accessible_from(context_path) {
                functions.extend(submodule.get_all_accessible_functions(context_path));
            }
        }
        
        functions
    }
    
    /// Check if this module is accessible from the given context
    pub fn is_accessible_from(&self, context_path: &ModulePath) -> bool {
        match &self.visibility {
            Visibility::Public => true,
            Visibility::Private => {
                // Only accessible from the same module or parent
                if let Some(parent) = &self.parent_path {
                    context_path == parent || context_path == &self.module_path
                } else {
                    context_path == &self.module_path
                }
            }
            Visibility::Internal => {
                // Accessible within the same package/crate
                let context_root = context_path.segments.first();
                let self_root = self.module_path.segments.first();
                context_root == self_root
            }
            Visibility::Protected => {
                // Accessible from parent and all its descendants
                if let Some(parent) = &self.parent_path {
                    context_path == parent || parent.is_ancestor_of(context_path) || context_path == &self.module_path
                } else {
                    context_path == &self.module_path
                }
            }
            Visibility::Package => {
                // Accessible within the same package
                let context_root = context_path.segments.first();
                let self_root = self.module_path.segments.first();
                context_root == self_root
            }
            Visibility::Restricted { restriction } => {
                self.check_restricted_visibility(restriction, context_path)
            }
        }
    }
    
    /// Check restricted visibility rules
    fn check_restricted_visibility(&self, restriction: &VisibilityRestriction, context_path: &ModulePath) -> bool {
        match restriction {
            VisibilityRestriction::Crate => {
                let context_root = context_path.segments.first();
                let self_root = self.module_path.segments.first();
                context_root == self_root
            }
            VisibilityRestriction::Super => {
                // Check if context module is a child of this module's parent
                if let Some(parent) = &self.parent_path {
                    parent.is_parent_of(context_path)
                } else {
                    false
                }
            }
            VisibilityRestriction::Module => context_path == &self.module_path,
            VisibilityRestriction::Path(allowed_path) => {
                // Check if context module matches the allowed path or is a descendant
                *allowed_path == *context_path || allowed_path.is_ancestor_of(context_path)
            }
        }
    }
}

/// Module resolver that can load modules from the file system
pub struct ModuleResolver {
    /// Root paths to search for modules (e.g., stdlib, user code)
    module_paths: Vec<PathBuf>,
    /// Cache of loaded modules
    module_cache: HashMap<String, ResolvedModule>,
    /// Map of module paths to resolved modules for hierarchy tracking
    path_cache: HashMap<ModulePath, ResolvedModule>,
}

/// Visibility checker for module access control
#[derive(Debug)]
pub struct VisibilityChecker {
    /// Current module context for visibility checks
    current_module_path: ModulePath,
    /// Package/crate root path for package-level visibility
    package_root: PathBuf,
}

impl VisibilityChecker {
    /// Create a new visibility checker for the given module context
    pub fn new(current_module_path: ModulePath, package_root: PathBuf) -> Self {
        Self {
            current_module_path,
            package_root,
        }
    }

    /// Check if an item with the given visibility is accessible from the current context
    pub fn is_accessible(&self, item_visibility: &Visibility, item_module_path: &ModulePath) -> bool {
        match item_visibility {
            Visibility::Public => true,
            Visibility::Private => self.is_same_module(item_module_path),
            Visibility::Internal => self.is_same_package(item_module_path),
            Visibility::Protected => self.is_submodule_or_same(item_module_path),
            Visibility::Package => self.is_same_package(item_module_path),
            Visibility::Restricted { restriction } => {
                self.check_restricted_visibility(restriction, item_module_path)
            }
        }
    }

    /// Check if the given module is the same as the current module
    fn is_same_module(&self, module_path: &ModulePath) -> bool {
        self.current_module_path == *module_path
    }

    /// Check if the given module is in the same package/crate
    fn is_same_package(&self, module_path: &ModulePath) -> bool {
        // For single-segment modules, assume they're in the same package
        // This handles common cases where modules don't have explicit package names
        if self.current_module_path.segments.len() == 1 && module_path.segments.len() == 1 {
            return true;
        }
        
        // For multi-segment modules, compare the first segment (package/crate name)
        let current_root = self.current_module_path.segments.first();
        let target_root = module_path.segments.first();
        current_root == target_root
    }

    /// Check if the given module is a submodule or the same module
    fn is_submodule_or_same(&self, module_path: &ModulePath) -> bool {
        // Check if the item's module is a parent or the same module
        if self.current_module_path.segments.len() >= module_path.segments.len() {
            for (i, segment) in module_path.segments.iter().enumerate() {
                if self.current_module_path.segments.get(i) != Some(segment) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Check restricted visibility rules
    fn check_restricted_visibility(&self, restriction: &VisibilityRestriction, item_module_path: &ModulePath) -> bool {
        match restriction {
            VisibilityRestriction::Crate => self.is_same_package(item_module_path),
            VisibilityRestriction::Super => {
                // Check if current module is a child of the item's module
                if item_module_path.segments.len() + 1 == self.current_module_path.segments.len() {
                    for (i, segment) in item_module_path.segments.iter().enumerate() {
                        if self.current_module_path.segments.get(i) != Some(segment) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            VisibilityRestriction::Module => self.is_same_module(item_module_path),
            VisibilityRestriction::Path(allowed_path) => {
                // Check if current module matches the allowed path
                *allowed_path == self.current_module_path
            }
        }
    }
}

impl ModuleResolver {
    /// Create a new module resolver with the given search paths
    pub fn new(module_paths: Vec<PathBuf>) -> Self {
        Self {
            module_paths,
            module_cache: HashMap::new(),
            path_cache: HashMap::new(),
        }
    }
    
    /// Add a new search path for modules
    pub fn add_module_path(&mut self, path: PathBuf) {
        self.module_paths.push(path);
    }
    
    /// Resolve a module path to an actual file path
    pub fn find_module_file(&self, module_path: &ModulePath) -> Option<PathBuf> {
        for base_path in &self.module_paths {
            let mut file_path = base_path.clone();
            
            // Convert module path segments to file path
            for segment in &module_path.segments {
                file_path.push(segment);
            }
            
            // Try different file extensions
            for extension in &["nyx", "ny"] {
                let mut path_with_ext = file_path.clone();
                path_with_ext.set_extension(extension);
                
                if path_with_ext.exists() {
                    return Some(path_with_ext);
                }
            }
            
            // Also try as a directory with mod.nyx or mod.ny
            for mod_file in &["mod.nyx", "mod.ny"] {
                let mod_path = file_path.join(mod_file);
                if mod_path.exists() {
                    return Some(mod_path);
                }
            }
        }
        
        None
    }
    
    /// Find all nested module files in a directory
    pub fn find_nested_modules(&self, base_path: &Path) -> Vec<(String, PathBuf)> {
        let mut modules = Vec::new();
        
        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                
                if path.is_dir() {
                    // Check for mod.nyx or mod.ny in subdirectory
                    for mod_file in &["mod.nyx", "mod.ny"] {
                        let mod_path = path.join(mod_file);
                        if mod_path.exists() {
                            modules.push((name.clone(), mod_path));
                            break;
                        }
                    }
                } else if path.is_file() {
                    // Check for .nyx or .ny files (but not mod.nyx/mod.ny which we already handle)
                    if let Some(extension) = path.extension() {
                        if (extension == "nyx" || extension == "ny") && !name.starts_with("mod.") {
                            if let Some(stem) = path.file_stem() {
                                modules.push((stem.to_string_lossy().to_string(), path));
                            }
                        }
                    }
                }
            }
        }
        
        modules
    }
    
    /// Load and parse a module from a file with nested module support
    pub fn load_module(&mut self, module_path: &ModulePath) -> ParseResult<ResolvedModule> {
        let path_string = module_path.to_string();
        
        // Check cache first
        if let Some(cached) = self.module_cache.get(&path_string) {
            return Ok(cached.clone());
        }
        
        // Check path cache
        if let Some(cached) = self.path_cache.get(module_path) {
            return Ok(cached.clone());
        }
        
        // Find the module file
        let file_path = self.find_module_file(module_path)
            .ok_or_else(|| ParseError {
                message: format!("Module not found: {}", path_string),
                position: 0,
            })?;
        
        // Read and parse the file
        let source = fs::read_to_string(&file_path)
            .map_err(|e| ParseError {
                message: format!("Failed to read module file {}: {}", file_path.display(), e),
                position: 0,
            })?;
        
        let mut parser = Parser::new(&source);
        
        // Try parsing as a module-aware program first, but fall back to simple program
        let program = if source.contains("import") || source.contains("mod ") {
            parser.parse_module_aware_program()?
        } else {
            // For simple module files with just functions, parse as regular program
            let simple_program = parser.parse_program()?;
            ModuleAwareProgram {
                imports: vec![],
                functions: simple_program.functions,
                modules: vec![],
            }
        };
        
        // Convert to resolved module
        let module_name = module_path.segments.last()
            .ok_or_else(|| ParseError {
                message: "Empty module path".to_string(),
                position: 0,
            })?
            .clone();
        
        let mut resolved_module = self.convert_to_resolved_module(
            module_name,
            file_path.clone(),
            module_path.clone(),
            program,
        )?;
        
        // Load nested modules from the same directory
        if let Some(parent_dir) = file_path.parent() {
            let nested_modules = self.find_nested_modules(parent_dir);
            
            for (nested_name, nested_path) in nested_modules {
                // Skip the current module file
                if nested_path == file_path {
                    continue;
                }
                
                // Create module path for nested module
                let nested_module_path = module_path.append(nested_name.clone());
                
                // Recursively load nested module
                match self.load_module(&nested_module_path) {
                    Ok(nested_module) => {
                        if let Err(e) = resolved_module.add_submodule(nested_module) {
                            eprintln!("Warning: Failed to add submodule {}: {}", nested_name, e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load nested module {}: {}", nested_name, e);
                    }
                }
            }
        }
        
        // Cache the result
        self.module_cache.insert(path_string, resolved_module.clone());
        self.path_cache.insert(module_path.clone(), resolved_module.clone());
        
        Ok(resolved_module)
    }
    
    /// Convert a parsed program to a resolved module
    fn convert_to_resolved_module(
        &mut self,
        name: String,
        path: PathBuf,
        module_path: ModulePath,
        program: ModuleAwareProgram,
    ) -> ParseResult<ResolvedModule> {
        let mut resolved_module = ResolvedModule::new(
            name,
            path.clone(),
            module_path,
            Visibility::Public, // File-based modules are typically public
        );
        
        // Add functions to the module
        resolved_module.functions = program.functions;
        
        // Process inline module declarations
        for module_decl in program.modules {
            let submodule = self.convert_module_decl_to_resolved(module_decl, &path, &resolved_module.module_path)?;
            if let Err(e) = resolved_module.add_submodule(submodule) {
                return Err(ParseError {
                    message: format!("Failed to add submodule: {}", e),
                    position: 0,
                });
            }
        }
        
        Ok(resolved_module)
    }
    
    /// Convert a module declaration to a resolved module
    fn convert_module_decl_to_resolved(
        &mut self,
        module_decl: ModuleDecl,
        parent_path: &Path,
        parent_module_path: &ModulePath,
    ) -> ParseResult<ResolvedModule> {
        let mut functions = Vec::new();
        
        // Extract functions from the module declaration
        for item in module_decl.items {
            match item {
                crate::ast::Item::Function(function) => {
                    functions.push(function);
                }
                _ => {
                    // For now, only support functions in modules
                    // Later we can add support for structs, etc.
                }
            }
        }
        
        let submodule_path = parent_module_path.append(module_decl.name.clone());
        let mut resolved_module = ResolvedModule::new(
            module_decl.name,
            parent_path.to_path_buf(),
            submodule_path,
            module_decl.visibility,
        );
        
        resolved_module.functions = functions;
        
        Ok(resolved_module)
    }
    
    /// Get all available functions from a module path
    pub fn get_module_functions(&mut self, module_path: &ModulePath) -> ParseResult<Vec<Function>> {
        let module = self.load_module(module_path)?;
        Ok(module.functions)
    }
    
    /// Check if a module exists
    pub fn module_exists(&self, module_path: &ModulePath) -> bool {
        self.find_module_file(module_path).is_some()
    }

    /// Get all available functions from a module path with visibility checks
    pub fn get_accessible_functions(
        &mut self, 
        module_path: &ModulePath,
        current_context: &ModulePath,
        package_root: &Path
    ) -> ParseResult<Vec<Function>> {
        let module = self.load_module(module_path)?;
        let checker = VisibilityChecker::new(current_context.clone(), package_root.to_path_buf());
        
        // Filter functions based on visibility
        let accessible_functions = module.functions.into_iter()
            .filter(|function| checker.is_accessible(&function.visibility, module_path))
            .collect();
        
        Ok(accessible_functions)
    }

    /// Check if a module is accessible from the current context
    pub fn is_module_accessible(
        &mut self,
        module_path: &ModulePath,
        current_context: &ModulePath,
        package_root: &Path
    ) -> ParseResult<bool> {
        let module = self.load_module(module_path)?;
        Ok(module.is_accessible_from(current_context))
    }

    /// Get all visible modules from the current context
    pub fn get_visible_modules(
        &mut self,
        current_context: &ModulePath,
        package_root: &Path
    ) -> Vec<String> {
        let mut visible_modules = Vec::new();
        
        // Get all cached modules and check visibility
        for (path, module) in &self.path_cache {
            if module.is_accessible_from(current_context) {
                visible_modules.push(path.to_string());
            }
        }
        
        visible_modules
    }
    
    /// Find a module in the loaded hierarchy
    pub fn find_module(&self, target_path: &ModulePath) -> Option<&ResolvedModule> {
        // Check path cache first
        if let Some(module) = self.path_cache.get(target_path) {
            return Some(module);
        }
        
        // Search through loaded modules
        for module in self.path_cache.values() {
            if let Some(found) = module.find_module(target_path) {
                return Some(found);
            }
        }
        
        None
    }
    
    /// Get the full module hierarchy as a tree structure
    pub fn get_module_tree(&self) -> HashMap<ModulePath, Vec<ModulePath>> {
        let mut tree = HashMap::new();
        
        for (path, module) in &self.path_cache {
            let mut children = Vec::new();
            
            for submodule in module.submodules.values() {
                children.push(submodule.module_path.clone());
            }
            
            tree.insert(path.clone(), children);
        }
        
        tree
    }
}

pub fn create_stdlib_resolver() -> ModuleResolver {
    let mut resolver = ModuleResolver::new(vec![]);
    
    // Add standard library paths
    if let Ok(current_dir) = std::env::current_dir() {
        let stdlib_path = current_dir.join("stdlib");
        if stdlib_path.exists() {
            resolver.add_module_path(stdlib_path);
        }
    }
    
    // Add common stdlib locations
    let stdlib_paths = vec![
        "/usr/local/lib/nyx/stdlib",
        "/usr/lib/nyx/stdlib",
        "./stdlib",
        "../stdlib",
    ];
    
    for path_str in stdlib_paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            resolver.add_module_path(path);
        }
    }
    
    resolver
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_module_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let module_content = r#"
            fun hello(): Int {
                return 42
            }
        "#;
        
        // Create a test module file
        let module_path = temp_dir.path().join("test_module.nyx");
        fs::write(&module_path, module_content).unwrap();
        
        // Set up resolver
        let mut resolver = ModuleResolver::new(vec![temp_dir.path().to_path_buf()]);
        
        // Test module resolution
        let module_path = ModulePath::single("test_module".to_string());
        assert!(resolver.module_exists(&module_path));
        
        let resolved = resolver.load_module(&module_path).unwrap();
        assert_eq!(resolved.name, "test_module");
        assert_eq!(resolved.functions.len(), 1);
        assert_eq!(resolved.functions[0].name, "hello");
    }
} 