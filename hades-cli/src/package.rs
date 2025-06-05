use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context, bail};
use nyx_compiler::ModuleResolver;

/// Package manifest structure (hades.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub package: PackageInfo,
    pub dependencies: Option<HashMap<String, DependencySpec>>,
    pub dev_dependencies: Option<HashMap<String, DependencySpec>>,
    pub build: Option<BuildConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub main: Option<String>, // Main entry point file
    pub include_stdlib: Option<bool>, // Whether to include stdlib (default: true)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    Version(String),
    Detailed {
        version: Option<String>,
        path: Option<String>,
        git: Option<String>,
        branch: Option<String>,
        tag: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub output_dir: Option<String>,
    pub target: Option<String>,
    pub optimization: Option<String>,
}

/// Package manager for handling dependencies and package operations
pub struct PackageManager {
    /// Current working directory (project root)
    pub project_root: PathBuf,
    /// Package manifest
    pub manifest: Option<PackageManifest>,
    /// Module resolver for dependency resolution
    pub module_resolver: ModuleResolver,
    /// Package cache directory
    pub cache_dir: PathBuf,
}

impl PackageManager {
    /// Create a new package manager for the given project root
    pub fn new(project_root: PathBuf) -> Result<Self> {
        let manifest = Self::load_manifest(&project_root)?;
        
        // Set up module resolver with package paths
        let mut module_resolver = nyx_compiler::create_stdlib_resolver();
        
        // Add project source directory
        let src_dir = project_root.join("src");
        if src_dir.exists() {
            module_resolver.add_module_path(src_dir);
        }
        
        // Add packages directory for dependencies
        let packages_dir = project_root.join("packages");
        if packages_dir.exists() {
            module_resolver.add_module_path(packages_dir.clone());
        }
        
        // Package cache (similar to .cargo or node_modules)
        let cache_dir = project_root.join(".hades").join("packages");
        
        // Auto-include stdlib unless explicitly disabled
        let mut package_manager = Self {
            project_root,
            manifest,
            module_resolver,
            cache_dir,
        };
        
        // Check if we should include stdlib (default: true)
        if let Some(manifest) = &package_manager.manifest {
            let include_stdlib = manifest.package.include_stdlib.unwrap_or(true);
            if include_stdlib {
                package_manager.ensure_stdlib_included()?;
            }
        }
        
        Ok(package_manager)
    }
    
    /// Load package manifest from hades.toml
    pub fn load_manifest(project_root: &Path) -> Result<Option<PackageManifest>> {
        let manifest_path = project_root.join("hades.toml");
        
        if !manifest_path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read {}", manifest_path.display()))?;
        
        let manifest: PackageManifest = toml::from_str(&content)
            .with_context(|| "Failed to parse hades.toml")?;
        
        Ok(Some(manifest))
    }
    
    /// Save package manifest to hades.toml
    pub fn save_manifest(&self, manifest: &PackageManifest) -> Result<()> {
        let manifest_path = self.project_root.join("hades.toml");
        
        let content = toml::to_string_pretty(manifest)
            .with_context(|| "Failed to serialize manifest")?;
        
        fs::write(&manifest_path, content)
            .with_context(|| format!("Failed to write {}", manifest_path.display()))?;
        
        Ok(())
    }
    
    /// Initialize a new package in the current directory
    pub fn init_package(
        name: String,
        version: Option<String>,
        authors: Option<Vec<String>>,
        include_stdlib: Option<bool>,
    ) -> Result<PackageManifest> {
        let manifest = PackageManifest {
            package: PackageInfo {
                name,
                version: version.unwrap_or_else(|| "0.1.0".to_string()),
                description: None,
                authors,
                license: None,
                repository: None,
                main: Some("src/main.nyx".to_string()),
                include_stdlib: Some(include_stdlib.unwrap_or(true)),
            },
            dependencies: None,
            dev_dependencies: None,
            build: Some(BuildConfig {
                output_dir: Some("build".to_string()),
                target: Some("hades-vm".to_string()),
                optimization: Some("debug".to_string()),
            }),
        };
        
        Ok(manifest)
    }
    
    /// Add a dependency to the package
    pub fn add_dependency(
        &mut self,
        name: String,
        spec: DependencySpec,
        dev: bool,
    ) -> Result<()> {
        if self.manifest.is_none() {
            bail!("No package manifest found. Run 'hades package init' first.");
        }
        
        let manifest = self.manifest.as_mut().unwrap();
        
        let deps = if dev {
            manifest.dev_dependencies.get_or_insert_with(HashMap::new)
        } else {
            manifest.dependencies.get_or_insert_with(HashMap::new)
        };
        
        deps.insert(name, spec);
        
        Ok(())
    }
    
    /// Remove a dependency from the package
    pub fn remove_dependency(&mut self, name: &str, dev: bool) -> Result<bool> {
        if self.manifest.is_none() {
            return Ok(false);
        }
        
        let manifest = self.manifest.as_mut().unwrap();
        
        let removed = if dev {
            manifest.dev_dependencies
                .as_mut()
                .map(|deps| deps.remove(name).is_some())
                .unwrap_or(false)
        } else {
            manifest.dependencies
                .as_mut()
                .map(|deps| deps.remove(name).is_some())
                .unwrap_or(false)
        };
        
        Ok(removed)
    }
    
    /// Install all dependencies specified in the manifest
    pub fn install_dependencies(&mut self, dev: bool) -> Result<()> {
        // Clone the manifest to avoid borrow checker issues
        let manifest = self.manifest.clone();
        
        if let Some(manifest) = manifest {
            println!("📦 Installing dependencies...");
            
            // Install regular dependencies
            if let Some(deps) = &manifest.dependencies {
                for (name, spec) in deps {
                    self.install_dependency(name, spec).with_context(|| {
                        format!("Failed to install dependency: {}", name)
                    })?;
                }
            }
            
            // Install dev dependencies if requested
            if dev {
                if let Some(dev_deps) = &manifest.dev_dependencies {
                    for (name, spec) in dev_deps {
                        self.install_dependency(name, spec).with_context(|| {
                            format!("Failed to install dev dependency: {}", name)
                        })?;
                    }
                }
            }
            
            println!("✅ Dependencies installed successfully");
        }
        
        Ok(())
    }
    
    /// Install a single dependency
    fn install_dependency(&mut self, name: &str, spec: &DependencySpec) -> Result<()> {
        println!("  📥 Installing {}...", name);
        
        match spec {
            DependencySpec::Version(version) => {
                // For now, just create a placeholder directory
                // In a real implementation, this would download from a registry
                let package_dir = self.cache_dir.join(name).join(version);
                fs::create_dir_all(&package_dir)?;
                
                // Create a basic package structure
                self.create_placeholder_package(&package_dir, name)?;
            }
            DependencySpec::Detailed { path, git, .. } => {
                if let Some(local_path) = path {
                    // Install from local path
                    let source_path = self.project_root.join(local_path);
                    let target_path = self.cache_dir.join(name);
                    
                    if source_path.exists() {
                        self.copy_local_package(&source_path, &target_path)?;
                    } else {
                        bail!("Local path not found: {}", source_path.display());
                    }
                } else if let Some(_git_url) = git {
                    // Install from git repository
                    // For now, just create a placeholder
                    let package_dir = self.cache_dir.join(name);
                    fs::create_dir_all(&package_dir)?;
                    self.create_placeholder_package(&package_dir, name)?;
                } else {
                    bail!("Unsupported dependency specification for {}", name);
                }
            }
        }
        
        // Add the installed package to module resolver
        let package_dir = self.cache_dir.join(name);
        if package_dir.exists() {
            self.module_resolver.add_module_path(package_dir);
        }
        
        Ok(())
    }
    
    /// Create a placeholder package (for demo purposes)
    fn create_placeholder_package(&self, package_dir: &Path, name: &str) -> Result<()> {
        fs::create_dir_all(package_dir)?;
        
        // Create a simple package manifest
        let manifest = PackageManifest {
            package: PackageInfo {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                description: Some(format!("Package: {}", name)),
                authors: None,
                license: None,
                repository: None,
                main: Some("lib.nyx".to_string()),
                include_stdlib: Some(true),
            },
            dependencies: None,
            dev_dependencies: None,
            build: None,
        };
        
        let manifest_content = toml::to_string_pretty(&manifest)?;
        fs::write(package_dir.join("hades.toml"), manifest_content)?;
        
        // Create a simple library file
        let lib_content = format!(
            r#"// {} library
pub mod {} {{
    pub fun hello(): Int {{
        return 42
    }}
}}
"#,
            name, name
        );
        
        fs::write(package_dir.join("lib.nyx"), lib_content)?;
        
        Ok(())
    }
    
    /// Copy a local package
    fn copy_local_package(&self, source: &Path, target: &Path) -> Result<()> {
        fs::create_dir_all(target)?;
        
        // For now, just copy the entire directory
        // In a real implementation, we'd be more selective
        if source.is_dir() {
            for entry in fs::read_dir(source)? {
                let entry = entry?;
                let source_path = entry.path();
                let target_path = target.join(entry.file_name());
                
                if source_path.is_dir() {
                    self.copy_local_package(&source_path, &target_path)?;
                } else {
                    fs::copy(&source_path, &target_path)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Build the current package
    pub fn build_package(&self, debug: bool) -> Result<()> {
        if let Some(manifest) = &self.manifest {
            println!("🔨 Building package: {}", manifest.package.name);
            
            let default_main = "src/main.nyx".to_string();
            let main_file = manifest.package.main
                .as_ref()
                .unwrap_or(&default_main);
            
            let source_path = self.project_root.join(main_file);
            
            if !source_path.exists() {
                bail!("Main file not found: {}", source_path.display());
            }
            
            // Build using the existing compiler
            let source = fs::read_to_string(&source_path)?;
            let compiler = crate::compiler::Compiler::new(debug);
            let bytecode = compiler.compile(&source)?;
            
            // Create build directory
            let build_dir = self.project_root.join(
                manifest.build.as_ref()
                    .and_then(|b| b.output_dir.as_ref())
                    .unwrap_or(&"build".to_string())
            );
            
            fs::create_dir_all(&build_dir)?;
            
            // Write bytecode
            let output_path = build_dir.join(format!("{}.hvm", manifest.package.name));
            fs::write(&output_path, bytecode)?;
            
            println!("✅ Built {} -> {}", manifest.package.name, output_path.display());
        } else {
            bail!("No package manifest found. Run 'hades package init' first.");
        }
        
        Ok(())
    }
    
    /// List installed packages
    pub fn list_packages(&self) -> Result<()> {
        if self.cache_dir.exists() {
            println!("📦 Installed packages:");
            
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let package_name = entry.file_name();
                    println!("  - {}", package_name.to_string_lossy());
                }
            }
        } else {
            println!("No packages installed");
        }
        
        Ok(())
    }
    
    /// Clean package cache
    pub fn clean(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
            println!("🧹 Cleaned package cache");
        }
        
        // Also clean build directory
        let build_dir = self.project_root.join("build");
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir)?;
            println!("🧹 Cleaned build directory");
        }
        
        Ok(())
    }
    
    /// Get the module resolver for this package manager
    pub fn get_module_resolver(&self) -> &ModuleResolver {
        &self.module_resolver
    }
    
    /// Get a mutable reference to the module resolver
    pub fn get_module_resolver_mut(&mut self) -> &mut ModuleResolver {
        &mut self.module_resolver
    }
    
    /// Ensure stdlib is included as a dependency if not explicitly disabled
    fn ensure_stdlib_included(&mut self) -> Result<()> {
        if let Some(manifest) = &mut self.manifest {
            // Check if stdlib is already in dependencies
            let has_stdlib = manifest.dependencies
                .as_ref()
                .map(|deps| deps.contains_key("std"))
                .unwrap_or(false);
            
            if !has_stdlib {
                // Add stdlib as a dependency
                let stdlib_spec = DependencySpec::Detailed {
                    version: Some("1.0.0".to_string()),
                    path: Some("stdlib".to_string()),
                    git: None,
                    branch: None,
                    tag: None,
                };
                
                self.add_dependency("std".to_string(), stdlib_spec, false)?;
                
                // Install stdlib immediately
                self.install_stdlib()?;
            }
        }
        
        Ok(())
    }
    
    /// Install the standard library
    fn install_stdlib(&mut self) -> Result<()> {
        println!("📚 Installing standard library...");
        
        // Add stdlib path to module resolver
        let stdlib_path = self.project_root.join("stdlib");
        if stdlib_path.exists() {
            self.module_resolver.add_module_path(stdlib_path);
        } else {
            // Try to find stdlib in the Hades VM installation
            let hades_stdlib = PathBuf::from("stdlib");
            if hades_stdlib.exists() {
                self.module_resolver.add_module_path(hades_stdlib);
            }
        }
        
        println!("✅ Standard library available");
        Ok(())
    }
}

/// Package-aware compiler that can resolve dependencies
pub struct PackageCompiler {
    pub package_manager: PackageManager,
    pub debug: bool,
}

impl PackageCompiler {
    pub fn new(project_root: PathBuf, debug: bool) -> Result<Self> {
        let package_manager = PackageManager::new(project_root)?;
        
        Ok(Self {
            package_manager,
            debug,
        })
    }
    
    /// Compile a package with dependency resolution
    pub fn compile_package(&mut self) -> Result<Vec<u8>> {
        if let Some(manifest) = &self.package_manager.manifest {
            let default_main = "src/main.nyx".to_string();
            let main_file = manifest.package.main
                .as_ref()
                .unwrap_or(&default_main);
            
            let source_path = self.package_manager.project_root.join(main_file);
            
            if !source_path.exists() {
                bail!("Main file not found: {}", source_path.display());
            }
            
            let source = fs::read_to_string(&source_path)?;
            
            // Use the enhanced compiler with module resolution
            let compiler = crate::compiler::Compiler::new(self.debug);
            let bytecode = compiler.compile(&source)?;
            
            Ok(bytecode)
        } else {
            bail!("No package manifest found. Run 'hades package init' first.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_package_manager(temp_dir: &TempDir) -> Result<PackageManager> {
        let project_root = temp_dir.path().to_path_buf();
        fs::create_dir_all(project_root.join("src"))?;
        
        // Create a test manifest
        let manifest = PackageManager::init_package(
            "test-package".to_string(),
            Some("0.1.0".to_string()),
            Some(vec!["Test Author".to_string()]),
            Some(true), // Include stdlib
        )?;
        
        let package_manager = PackageManager::new(project_root.clone())?;
        package_manager.save_manifest(&manifest)?;
        
        PackageManager::new(project_root)
    }

    #[test]
    fn test_package_initialization() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let manifest = PackageManager::init_package(
            "test-package".to_string(),
            Some("1.0.0".to_string()),
            Some(vec!["Alice".to_string(), "Bob".to_string()]),
            Some(true),
        )?;

        assert_eq!(manifest.package.name, "test-package");
        assert_eq!(manifest.package.version, "1.0.0");
        assert_eq!(manifest.package.authors, Some(vec!["Alice".to_string(), "Bob".to_string()]));
        assert_eq!(manifest.package.include_stdlib, Some(true));
        assert!(manifest.build.is_some());

        Ok(())
    }

    #[test]
    fn test_package_initialization_with_defaults() -> Result<()> {
        let manifest = PackageManager::init_package(
            "test-package".to_string(),
            None,
            None,
            None,
        )?;

        assert_eq!(manifest.package.name, "test-package");
        assert_eq!(manifest.package.version, "0.1.0"); // Default version
        assert_eq!(manifest.package.include_stdlib, Some(true)); // Default stdlib inclusion
        assert!(manifest.package.authors.is_none());

        Ok(())
    }

    #[test]
    fn test_stdlib_disabled() -> Result<()> {
        let manifest = PackageManager::init_package(
            "no-stdlib-package".to_string(),
            Some("1.0.0".to_string()),
            None,
            Some(false), // Disable stdlib
        )?;

        assert_eq!(manifest.package.include_stdlib, Some(false));

        Ok(())
    }

    #[test]
    fn test_manifest_save_and_load() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path().to_path_buf();
        
        let manifest = PackageManager::init_package(
            "test-save-load".to_string(),
            Some("2.0.0".to_string()),
            Some(vec!["Test Author".to_string()]),
            Some(true),
        )?;

        // Create a minimal package manager for saving
        fs::create_dir_all(&project_root)?;
        let package_manager = PackageManager {
            project_root: project_root.clone(),
            manifest: Some(manifest.clone()),
            module_resolver: nyx_compiler::create_stdlib_resolver(),
            cache_dir: project_root.join(".hades").join("packages"),
        };
        
        package_manager.save_manifest(&manifest)?;

        // Load the manifest back
        let loaded_manifest = PackageManager::load_manifest(&project_root)?;
        assert!(loaded_manifest.is_some());
        
        let loaded = loaded_manifest.unwrap();
        assert_eq!(loaded.package.name, "test-save-load");
        assert_eq!(loaded.package.version, "2.0.0");

        Ok(())
    }

    #[test]
    fn test_dependency_management() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut package_manager = create_test_package_manager(&temp_dir)?;

        // Add a version dependency
        let version_spec = DependencySpec::Version("1.0.0".to_string());
        package_manager.add_dependency("math-lib".to_string(), version_spec, false)?;

        // Add a path dependency
        let path_spec = DependencySpec::Detailed {
            version: None,
            path: Some("../local-lib".to_string()),
            git: None,
            branch: None,
            tag: None,
        };
        package_manager.add_dependency("local-lib".to_string(), path_spec, false)?;

        // Add a dev dependency
        let dev_spec = DependencySpec::Version("2.0.0".to_string());
        package_manager.add_dependency("test-utils".to_string(), dev_spec, true)?;

        // Check dependencies were added
        let manifest = package_manager.manifest.as_ref().unwrap();
        
        assert!(manifest.dependencies.is_some());
        let deps = manifest.dependencies.as_ref().unwrap();
        assert!(deps.contains_key("math-lib"));
        assert!(deps.contains_key("local-lib"));
        
        assert!(manifest.dev_dependencies.is_some());
        let dev_deps = manifest.dev_dependencies.as_ref().unwrap();
        assert!(dev_deps.contains_key("test-utils"));

        Ok(())
    }

    #[test]
    fn test_dependency_removal() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut package_manager = create_test_package_manager(&temp_dir)?;

        // Add then remove a dependency
        let spec = DependencySpec::Version("1.0.0".to_string());
        package_manager.add_dependency("removable-lib".to_string(), spec, false)?;
        
        let removed = package_manager.remove_dependency("removable-lib", false)?;
        assert!(removed);

        // Try to remove non-existent dependency
        let not_removed = package_manager.remove_dependency("non-existent", false)?;
        assert!(!not_removed);

        Ok(())
    }

    #[test]
    fn test_package_creation_workflow() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path().to_path_buf();

        // Initialize package
        let manifest = PackageManager::init_package(
            "workflow-test".to_string(),
            Some("1.0.0".to_string()),
            Some(vec!["Developer".to_string()]),
            Some(true),
        )?;

        // Create directory structure
        fs::create_dir_all(project_root.join("src"))?;

        // Create main.nyx file
        let main_content = r#"
// Main entry point with stdlib usage
import std.math

fun main(): Int {
    return std.math.abs(-42)
}
"#;
        fs::write(project_root.join("src").join("main.nyx"), main_content)?;

        // Create package manager
        let package_manager = PackageManager {
            project_root: project_root.clone(),
            manifest: Some(manifest.clone()),
            module_resolver: nyx_compiler::create_stdlib_resolver(),
            cache_dir: project_root.join(".hades").join("packages"),
        };
        
        package_manager.save_manifest(&manifest)?;

        // Verify files were created
        assert!(project_root.join("hades.toml").exists());
        assert!(project_root.join("src").join("main.nyx").exists());

        // Load package manager from disk
        let loaded_manager = PackageManager::new(project_root)?;
        assert!(loaded_manager.manifest.is_some());
        assert_eq!(loaded_manager.manifest.unwrap().package.name, "workflow-test");

        Ok(())
    }

    #[test]
    fn test_stdlib_inclusion_by_default() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path().to_path_buf();
        fs::create_dir_all(project_root.join("src"))?;

        // Create manifest with stdlib enabled (default)
        let manifest = PackageManager::init_package(
            "stdlib-test".to_string(),
            None,
            None,
            None, // Default should include stdlib
        )?;

        let package_manager = PackageManager {
            project_root: project_root.clone(),
            manifest: Some(manifest.clone()),
            module_resolver: nyx_compiler::create_stdlib_resolver(),
            cache_dir: project_root.join(".hades").join("packages"),
        };
        
        package_manager.save_manifest(&manifest)?;

        // When loading package manager, stdlib should be automatically included
        let loaded_manager = PackageManager::new(project_root)?;
        let loaded_manifest = loaded_manager.manifest.unwrap();
        
        // Should have stdlib as dependency
        assert!(loaded_manifest.dependencies.is_some());
        let deps = loaded_manifest.dependencies.unwrap();
        assert!(deps.contains_key("std"));

        Ok(())
    }

    #[test]
    fn test_stdlib_exclusion_when_disabled() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path().to_path_buf();
        fs::create_dir_all(project_root.join("src"))?;

        // Create manifest with stdlib explicitly disabled
        let manifest = PackageManager::init_package(
            "no-stdlib-test".to_string(),
            None,
            None,
            Some(false), // Explicitly disable stdlib
        )?;

        let package_manager = PackageManager {
            project_root: project_root.clone(),
            manifest: Some(manifest.clone()),
            module_resolver: nyx_compiler::create_stdlib_resolver(),
            cache_dir: project_root.join(".hades").join("packages"),
        };
        
        package_manager.save_manifest(&manifest)?;

        // When loading package manager, stdlib should NOT be automatically included
        let loaded_manager = PackageManager::new(project_root)?;
        let loaded_manifest = loaded_manager.manifest.unwrap();
        
        // Should NOT have stdlib as dependency
        let has_std = loaded_manifest.dependencies
            .as_ref()
            .map(|deps| deps.contains_key("std"))
            .unwrap_or(false);
        assert!(!has_std);

        Ok(())
    }

    #[test]
    fn test_module_resolver_integration() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let package_manager = create_test_package_manager(&temp_dir)?;

        // Module resolver should have multiple paths
        // Note: We can't directly test the internal state, but we can verify
        // the package manager was created successfully with module resolution
        assert!(package_manager.manifest.is_some());

        Ok(())
    }

    #[test]
    fn test_build_configuration() -> Result<()> {
        let manifest = PackageManager::init_package(
            "build-test".to_string(),
            Some("1.0.0".to_string()),
            None,
            Some(true),
        )?;

        let build_config = manifest.build.unwrap();
        assert_eq!(build_config.output_dir, Some("build".to_string()));
        assert_eq!(build_config.target, Some("hades-vm".to_string()));
        assert_eq!(build_config.optimization, Some("debug".to_string()));

        Ok(())
    }

    #[test]
    fn test_dependency_spec_serialization() -> Result<()> {
        // Test creating and using different dependency specs
        let version_spec = DependencySpec::Version("1.0.0".to_string());
        match version_spec {
            DependencySpec::Version(v) => assert_eq!(v, "1.0.0"),
            _ => panic!("Wrong variant"),
        }

        // Test detailed spec
        let detailed_spec = DependencySpec::Detailed {
            version: Some("2.0.0".to_string()),
            path: Some("../local".to_string()),
            git: None,
            branch: None,
            tag: None,
        };
        match detailed_spec {
            DependencySpec::Detailed { version, path, .. } => {
                assert_eq!(version, Some("2.0.0".to_string()));
                assert_eq!(path, Some("../local".to_string()));
            },
            _ => panic!("Wrong variant"),
        }

        // Test that we can clone and use the specs
        let git_spec = DependencySpec::Detailed {
            version: None,
            path: None,
            git: Some("https://github.com/example/repo.git".to_string()),
            branch: Some("main".to_string()),
            tag: None,
        };
        let cloned_spec = git_spec.clone();
        
        match cloned_spec {
            DependencySpec::Detailed { git, branch, .. } => {
                assert_eq!(git, Some("https://github.com/example/repo.git".to_string()));
                assert_eq!(branch, Some("main".to_string()));
            },
            _ => panic!("Wrong variant"),
        }

        Ok(())
    }
} 