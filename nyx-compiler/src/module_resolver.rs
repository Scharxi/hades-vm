use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ast::{ModulePath, ModuleAwareProgram, Function, ModuleDecl};
use crate::parser::{Parser, ParseResult, ParseError};

/// Represents a resolved module with its functions and submodules
#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub name: String,
    pub path: PathBuf,
    pub functions: Vec<Function>,
    pub submodules: HashMap<String, ResolvedModule>,
    pub is_public: bool,
}

/// Module resolver that can load modules from the file system
pub struct ModuleResolver {
    /// Root paths to search for modules (e.g., stdlib, user code)
    module_paths: Vec<PathBuf>,
    /// Cache of loaded modules
    module_cache: HashMap<String, ResolvedModule>,
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
            is_public: true, // File-based modules are typically public
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
            is_public: matches!(module_decl.visibility, crate::ast::Visibility::Public),
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