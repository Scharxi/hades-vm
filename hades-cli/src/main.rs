mod compiler;
mod vm;
mod hex_tool;
mod package;

use std::path::PathBuf;
use std::fs;
use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use compiler::Compiler;
use vm::VM;
use package::{PackageManager, PackageManifest, DependencySpec};

/// Hades VM CLI - Compile and run Hades source files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compile and run using the legacy bytecode format
    Run {
        /// Source file to compile and run
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output file for compiled bytecode (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Only compile, don't run
        #[arg(short, long)]
        compile_only: bool,
        
        /// Print debug information
        #[arg(short, long)]
        debug: bool,
        
        /// Function arguments (for main function)
        #[arg(short, long, num_args = 0.., value_delimiter = ' ')]
        args: Vec<i32>,
    },
    
    /// Compile source code to HEX executable format
    CompileHex {
        /// Source file to compile
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output HEX file
        #[arg(short, long)]
        output: PathBuf,
        
        /// Print debug information
        #[arg(short, long)]
        debug: bool,
    },
    
    /// Run a HEX executable file
    RunHex {
        /// HEX executable file to run
        #[arg(short, long)]
        input: PathBuf,
        
        /// Print debug information
        #[arg(short, long)]
        debug: bool,
    },
    
    /// Compile and run source code using HEX format (one-step)
    HexRun {
        /// Source file to compile and run
        #[arg(short, long)]
        input: PathBuf,
        
        /// Print debug information
        #[arg(short, long)]
        debug: bool,
    },
    
    /// Display information about a HEX executable file
    HexInfo {
        /// HEX executable file to inspect
        #[arg(short, long)]
        input: PathBuf,
    },
    
    /// Create a test HEX executable file
    CreateTestHex {
        /// Output HEX file
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Package management commands
    Package {
        #[command(subcommand)]
        command: PackageCommands,
    },
}

#[derive(Subcommand, Debug)]
enum PackageCommands {
    /// Initialize a new package in the current directory
    Init {
        /// Package name
        name: Option<String>,
        
        /// Package version
        #[arg(short, long)]
        version: Option<String>,
        
        /// Package authors
        #[arg(short, long, num_args = 0.., value_delimiter = ',')]
        authors: Option<Vec<String>>,
        
        /// Disable automatic stdlib inclusion
        #[arg(long)]
        no_stdlib: bool,
    },
    
    /// Add a dependency to the package
    Add {
        /// Dependency name
        name: String,
        
        /// Dependency version
        #[arg(short, long)]
        version: Option<String>,
        
        /// Local path to the dependency
        #[arg(short, long)]
        path: Option<String>,
        
        /// Git repository URL
        #[arg(short, long)]
        git: Option<String>,
        
        /// Add as development dependency
        #[arg(short, long)]
        dev: bool,
    },
    
    /// Remove a dependency from the package
    Remove {
        /// Dependency name
        name: String,
        
        /// Remove from development dependencies
        #[arg(short, long)]
        dev: bool,
    },
    
    /// Install package dependencies
    Install {
        /// Install development dependencies too
        #[arg(short, long)]
        dev: bool,
    },
    
    /// Build the current package
    Build {
        /// Build with debug information
        #[arg(short, long)]
        debug: bool,
    },
    
    /// Run the built package
    Run {
        /// Build and run with debug information
        #[arg(short, long)]
        debug: bool,
        
        /// Function arguments (for main function)
        #[arg(short, long, num_args = 0.., value_delimiter = ' ')]
        args: Vec<i32>,
        
        /// Only build, don't run
        #[arg(short, long)]
        build_only: bool,
    },
    
    /// List installed dependencies
    List,
    
    /// Clean build artifacts and package cache
    Clean,
    
    /// Show package information
    Info,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Run { input, output, compile_only, debug, args: func_args } => {
            run_legacy_mode(input, output, compile_only, debug, func_args)?;
        }
        
        Commands::CompileHex { input, output, debug } => {
            hex_tool::compile_to_hex(&input, &output, debug)
                .with_context(|| "Failed to compile to HEX format")?;
            
            println!("✅ Successfully compiled {} to {}", 
                     input.display(), output.display());
        }
        
        Commands::RunHex { input, debug } => {
            hex_tool::run_hex_file(&input, debug)
                .with_context(|| "Failed to run HEX executable")?;
        }
        
        Commands::HexRun { input, debug } => {
            hex_tool::compile_and_run(&input, debug)
                .with_context(|| "Failed to compile and run with HEX format")?;
        }
        
        Commands::HexInfo { input } => {
            hex_tool::hex_info(&input)
                .with_context(|| "Failed to display HEX file information")?;
        }
        
        Commands::CreateTestHex { output } => {
            hex_tool::create_test_hex(&output)
                .with_context(|| "Failed to create test HEX file")?;
        }

        Commands::Package { command } => {
            let current_dir = std::env::current_dir()
                .with_context(|| "Failed to get current directory")?;
            
            match command {
                PackageCommands::Init { name, version, authors, no_stdlib } => {
                    handle_package_init(current_dir, name, version, authors, no_stdlib)?;
                }
                PackageCommands::Add { name, version, path, git, dev } => {
                    handle_package_add(current_dir, name, version, path, git, dev)?;
                }
                PackageCommands::Remove { name, dev } => {
                    handle_package_remove(current_dir, name, dev)?;
                }
                PackageCommands::Install { dev } => {
                    handle_package_install(current_dir, dev)?;
                }
                PackageCommands::Build { debug } => {
                    handle_package_build(current_dir, debug)?;
                }
                PackageCommands::List => {
                    handle_package_list(current_dir)?;
                }
                PackageCommands::Clean => {
                    handle_package_clean(current_dir)?;
                }
                PackageCommands::Info => {
                    handle_package_info(current_dir)?;
                }
                PackageCommands::Run { debug, args, build_only } => {
                    handle_package_run(current_dir, debug, args, build_only)?;
                }
            }
        }
    }

    Ok(())
}

fn run_legacy_mode(
    input: PathBuf,
    output: Option<PathBuf>,
    compile_only: bool,
    debug: bool,
    func_args: Vec<i32>,
) -> Result<()> {
    // Read source file
    let source = fs::read_to_string(&input)
        .with_context(|| format!("Failed to read source file: {}", input.display()))?;

    // Create compiler and compile source
    let compiler = Compiler::new(debug);
    let bytecode = compiler.compile(&source)
        .with_context(|| "Failed to compile source")?;

    // If output file is specified, write the bytecode
    if let Some(output_path) = output {
        fs::write(&output_path, &bytecode)
            .with_context(|| format!("Failed to write bytecode to: {}", output_path.display()))?;
    }

    // Run the bytecode unless compile-only flag is set
    if !compile_only {
        let mut vm = VM::new(debug);
        
        // Push function arguments onto the stack in forward order
        // since we're using base_pointer + index to access them
        for arg in func_args.iter() {
            vm.push_arg(*arg)?;
        }
        
        let result = vm.execute(&bytecode)
            .with_context(|| "Failed to execute bytecode")?;
        
        if debug {
            println!("Program completed with result: {}", result);
        } else {
            println!("{}", result);
        }
    }

    Ok(())
}

// Package command handlers

fn handle_package_init(
    project_root: PathBuf,
    name: Option<String>,
    version: Option<String>,
    authors: Option<Vec<String>>,
    no_stdlib: bool,
) -> Result<()> {
    let package_name = match name {
        Some(n) => n.clone(),
        None => {
            // Use directory name as package name
            project_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("my-package")
                .to_string()
        }
    };

    println!("📦 Initializing package: {}", package_name);

    // Check if already initialized
    let manifest_path = project_root.join("hades.toml");
    if manifest_path.exists() {
        println!("⚠️  Package already initialized (hades.toml exists)");
        return Ok(());
    }

    // Create package manifest
    let manifest = PackageManager::init_package(
        package_name.clone(),
        version,
        authors,
        Some(!no_stdlib),
    )?;

    // Create directory structure
    fs::create_dir_all(project_root.join("src"))?;
    
    // Create main.nyx file if it doesn't exist
    let main_file = project_root.join("src").join("main.nyx");
    if !main_file.exists() {
        let main_content = r#"// Main entry point for the package

fun main(): Int {
    return 42
}
"#;
        fs::write(main_file, main_content)?;
    }

    // Save manifest
    let package_manager = PackageManager::new(project_root)?;
    package_manager.save_manifest(&manifest)?;

    println!("✅ Package '{}' initialized successfully", package_name);
    println!("   📁 Created src/main.nyx");
    println!("   📄 Created hades.toml");
    
    Ok(())
}

fn handle_package_add(
    project_root: PathBuf,
    name: String,
    version: Option<String>,
    path: Option<String>,
    git: Option<String>,
    dev: bool,
) -> Result<()> {
    let mut package_manager = PackageManager::new(project_root)?;
    
    if package_manager.manifest.is_none() {
        anyhow::bail!("No package manifest found. Run 'hades package init' first.");
    }

    println!("📦 Adding dependency: {}", name);

    let spec = if let Some(local_path) = path {
        DependencySpec::Detailed {
            version: None,
            path: Some(local_path),
            git: None,
            branch: None,
            tag: None,
        }
    } else if let Some(git_url) = git {
        DependencySpec::Detailed {
            version: None,
            path: None,
            git: Some(git_url),
            branch: None,
            tag: None,
        }
    } else if let Some(ver) = version {
        DependencySpec::Version(ver)
    } else {
        DependencySpec::Version("*".to_string())
    };

    package_manager.add_dependency(name.clone(), spec, dev)?;
    
    if let Some(manifest) = &package_manager.manifest {
        package_manager.save_manifest(manifest)?;
    }

    println!("✅ Added {} as a {} dependency", name, if dev { "development" } else { "runtime" });
    
    Ok(())
}

fn handle_package_remove(
    project_root: PathBuf,
    name: String,
    dev: bool,
) -> Result<()> {
    let mut package_manager = PackageManager::new(project_root)?;
    
    if package_manager.manifest.is_none() {
        anyhow::bail!("No package manifest found. Run 'hades package init' first.");
    }

    println!("📦 Removing dependency: {}", name);

    let removed = package_manager.remove_dependency(&name, dev)?;
    
    if removed {
        if let Some(manifest) = &package_manager.manifest {
            package_manager.save_manifest(manifest)?;
        }
        println!("✅ Removed {} from {} dependencies", name, if dev { "development" } else { "runtime" });
    } else {
        println!("⚠️  Dependency '{}' not found in {} dependencies", name, if dev { "development" } else { "runtime" });
    }
    
    Ok(())
}

fn handle_package_install(
    project_root: PathBuf,
    dev: bool,
) -> Result<()> {
    let mut package_manager = PackageManager::new(project_root)?;
    
    if package_manager.manifest.is_none() {
        anyhow::bail!("No package manifest found. Run 'hades package init' first.");
    }

    package_manager.install_dependencies(dev)?;
    
    Ok(())
}

fn handle_package_build(
    project_root: PathBuf,
    debug: bool,
) -> Result<()> {
    let package_manager = PackageManager::new(project_root)?;
    
    if package_manager.manifest.is_none() {
        anyhow::bail!("No package manifest found. Run 'hades package init' first.");
    }

    package_manager.build_package(debug)?;
    
    Ok(())
}

fn handle_package_list(project_root: PathBuf) -> Result<()> {
    let package_manager = PackageManager::new(project_root)?;
    package_manager.list_packages()?;
    Ok(())
}

fn handle_package_clean(project_root: PathBuf) -> Result<()> {
    let package_manager = PackageManager::new(project_root)?;
    package_manager.clean()?;
    Ok(())
}

fn handle_package_info(project_root: PathBuf) -> Result<()> {
    let package_manager = PackageManager::new(project_root)?;
    
    if let Some(manifest) = &package_manager.manifest {
        println!("📦 Package Information");
        println!("   Name: {}", manifest.package.name);
        println!("   Version: {}", manifest.package.version);
        
        if let Some(desc) = &manifest.package.description {
            println!("   Description: {}", desc);
        }
        
        if let Some(authors) = &manifest.package.authors {
            println!("   Authors: {}", authors.join(", "));
        }
        
        if let Some(deps) = &manifest.dependencies {
            println!("   Dependencies: {}", deps.len());
            for (name, _) in deps {
                println!("     - {}", name);
            }
        }
        
        if let Some(dev_deps) = &manifest.dev_dependencies {
            println!("   Dev Dependencies: {}", dev_deps.len());
            for (name, _) in dev_deps {
                println!("     - {}", name);
            }
        }
    } else {
        println!("No package manifest found. Run 'hades package init' first.");
    }
    
    Ok(())
}

fn handle_package_run(
    project_root: PathBuf,
    debug: bool,
    args: Vec<i32>,
    build_only: bool,
) -> Result<()> {
    let package_manager = PackageManager::new(project_root)?;
    
    if package_manager.manifest.is_none() {
        anyhow::bail!("No package manifest found. Run 'hades package init' first.");
    }

    // Build the package first
    package_manager.build_package(debug)?;

    if !build_only {
        // Get the built bytecode file path
        let manifest = package_manager.manifest.as_ref().unwrap();
        let build_dir = package_manager.project_root.join(
            manifest.build.as_ref()
                .and_then(|b| b.output_dir.as_ref())
                .unwrap_or(&"build".to_string())
        );
        let bytecode_path = build_dir.join(format!("{}.hvm", manifest.package.name));
        
        // Read the bytecode from the file
        let bytecode = fs::read(&bytecode_path)
            .with_context(|| format!("Failed to read built package: {}", bytecode_path.display()))?;
        
        // Create VM and run the bytecode
        let mut vm = VM::new(debug);
        
        // Push function arguments onto the stack in forward order
        // since we're using base_pointer + index to access them
        for arg in args.iter() {
            vm.push_arg(*arg)?;
        }
        
        let result = vm.execute(&bytecode)?;
        
        if debug {
            println!("Program completed with result: {}", result);
        } else {
            println!("{}", result);
        }
    }

    Ok(())
}

