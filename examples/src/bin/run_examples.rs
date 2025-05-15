use std::fs;
use std::path::Path;
use vm::vm::VirtualMachine;
use nyx_compiler::{lexer::Lexer, parser::Parser};

fn run_example(name: &str) -> Result<i64, String> {
    println!("\nRunning example: {}", name);
    
    // Read the source file
    let path = Path::new("examples/nyx").join(format!("{}.nyx", name));
    let source = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    
    // Parse and compile
    let mut parser = Parser::new(&source);
    let ast = parser.parse_program()
        .map_err(|e| format!("Failed to parse {}: {:?}", name, e))?;
    
    // Create VM and run
    let mut vm = VirtualMachine::new(true); // Enable debug mode
    
    // TODO: Once compiler is implemented, replace this with actual compilation
    println!("AST: {:?}", ast);
    
    Ok(42) // Temporary return until compiler is implemented
}

fn main() {
    let examples = vec![
        "arithmetic",
        "stack_ops",
        "control_flow",
        "functions",
        "memory_ops",
    ];
    
    for example in examples {
        match run_example(example) {
            Ok(result) => println!("{} result: {}", example, result),
            Err(e) => eprintln!("{} error: {}", example, e),
        }
    }
} 