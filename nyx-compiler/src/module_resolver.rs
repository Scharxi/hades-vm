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
    pub functions: Vec<Function>,
    pub submodules: HashMap<String, ResolvedModule>,
    pub visibility: crate::ast::Visibility,
}

/// Module resolver that can load modules from the file system
pub struct ModuleResolver {
    /// Root paths to search for modules (e.g., stdlib, user code)
    module_paths: Vec<PathBuf>,
    /// Cache of loaded modules
    module_cache: HashMap<String, ResolvedModule>,
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
    fn is_same_package(&self, _module_path: &ModulePath) -> bool {
        // For now, assume all modules are in the same package
        // In a real implementation, this would check against package boundaries
        true
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
    
    /// Load and parse a module from a file
    pub fn load_module(&mut self, module_path: &ModulePath) -> ParseResult<ResolvedModule> {
        let path_string = module_path.to_string();
        
        // Check cache first
        if let Some(cached) = self.module_cache.get(&path_string) {
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
        
        let resolved_module = self.convert_to_resolved_module(
            module_name,
            file_path,
            program,
        )?;
        
        // Cache the result
        self.module_cache.insert(path_string, resolved_module.clone());
        
        Ok(resolved_module)
    }
    
    /// Convert a parsed program to a resolved module
    fn convert_to_resolved_module(
        &mut self,
        name: String,
        path: PathBuf,
        program: ModuleAwareProgram,
    ) -> ParseResult<ResolvedModule> {
        let mut submodules = HashMap::new();
        
        // Process submodules
        for module_decl in program.modules {
            let submodule = self.convert_module_decl_to_resolved(module_decl, &path)?;
            submodules.insert(submodule.name.clone(), submodule);
        }
        
        Ok(ResolvedModule {
            name,
            path,
            functions: program.functions,
            submodules,
            visibility: crate::ast::Visibility::Public, // File-based modules are typically public
        })
    }
    
    /// Convert a module declaration to a resolved module
    fn convert_module_decl_to_resolved(
        &mut self,
        module_decl: ModuleDecl,
        parent_path: &Path,
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
        
        Ok(ResolvedModule {
            name: module_decl.name,
            path: parent_path.to_path_buf(),
            functions,
            submodules: HashMap::new(), // Nested modules not supported yet
            visibility: module_decl.visibility,
        })
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
        let checker = VisibilityChecker::new(current_context.clone(), package_root.to_path_buf());
        
        Ok(checker.is_accessible(&module.visibility, module_path))
    }

    /// Get all visible modules from the current context
    pub fn get_visible_modules(
        &mut self,
        current_context: &ModulePath,
        package_root: &Path
    ) -> Vec<String> {
        let checker = VisibilityChecker::new(current_context.clone(), package_root.to_path_buf());
        let mut visible_modules = Vec::new();
        
        // Check all cached modules for visibility
        for (module_name, module) in &self.module_cache {
            let module_path = ModulePath::single(module.name.clone());
            if checker.is_accessible(&module.visibility, &module_path) {
                visible_modules.push(module_name.clone());
            }
        }
        
        visible_modules
    }
}

/// Standard library initialization
pub fn create_stdlib_resolver() -> ModuleResolver {
    let mut resolver = ModuleResolver::new(vec![]);
    
    // Add standard library path (this would be set up during installation)
    if let Ok(stdlib_path) = std::env::var("NYX_STDLIB_PATH") {
        resolver.add_module_path(PathBuf::from(stdlib_path));
    }
    
    // Add local stdlib directory
    resolver.add_module_path(PathBuf::from("stdlib"));
    
    // Add current directory for local modules
    resolver.add_module_path(PathBuf::from("."));
    
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