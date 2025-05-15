mod compiler;
mod vm;

use std::path::PathBuf;
use std::fs;
use clap::Parser;
use anyhow::{Result, Context};
use compiler::Compiler;
use vm::VM;

/// Hades VM CLI - Compile and run Hades source files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
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
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read source file
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read source file: {}", args.input.display()))?;

    // Create compiler and compile source
    let compiler = Compiler::new(args.debug);
    let bytecode = compiler.compile(&source)
        .with_context(|| "Failed to compile source")?;

    // If output file is specified, write the bytecode
    if let Some(output_path) = args.output {
        fs::write(&output_path, &bytecode)
            .with_context(|| format!("Failed to write bytecode to: {}", output_path.display()))?;
    }

    // Run the bytecode unless compile-only flag is set
    if !args.compile_only {
        let mut vm = VM::new(args.debug);
        
        // Push function arguments onto the stack in forward order
        // since we're using base_pointer + index to access them
        for arg in args.args.iter() {
            vm.push_arg(*arg)?;
        }
        
        let result = vm.execute(&bytecode)
            .with_context(|| "Failed to execute bytecode")?;
        
        if args.debug {
            println!("Program completed with result: {}", result);
        } else {
            println!("{}", result);
        }
    }

    Ok(())
}
