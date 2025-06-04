mod compiler;
mod vm;
mod hex_tool;

use std::path::PathBuf;
use std::fs;
use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use compiler::Compiler;
use vm::VM;

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
