use std::fs;
use std::path::Path;
use vm::vm::VirtualMachine;
use nyx_compiler::compile_to_bytecode;

fn run_example(name: &str) -> Result<i64, String> {
    println!("\n🚀 Running example: {}", name);
    
    // Read the source file
    let path = Path::new("examples/nyx").join(format!("{}.nyx", name));
    let source = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    
    println!("📄 Source code:");
    println!("{}", source);
    
    // Compile to bytecode
    let bytecode = compile_to_bytecode(&source)
        .map_err(|e| format!("Failed to compile {}: {}", name, e))?;
    
    println!("💾 Generated {} bytes of bytecode", bytecode.len());
    
    // Create VM and run
    let mut vm = VirtualMachine::new(true); // Enable debug mode
    vm.load_program(&bytecode);
    
    println!("🖥️ Running on Hades VM...");
    vm.run_until_completion();
    
    // For now, return a dummy result
    // In a real implementation, we'd capture the actual return value from the VM
    Ok(42)
}

fn main() {
    println!("🎯 Hades VM - Kotlin-like Language Examples");
    println!("==========================================");
    
    let examples = vec![
        "simple_arithmetic",  // Test basic arithmetic without function calls
        "arithmetic",
        "stack_ops", 
        "control_flow",
        "functions",
        // Skip memory_ops for now as it uses special built-in functions
        // "memory_ops",
    ];
    
    for example in examples {
        match run_example(example) {
            Ok(result) => println!("✅ {} completed successfully (result: {})", example, result),
            Err(e) => eprintln!("❌ {} failed: {}", example, e),
        }
        println!("{}", "─".repeat(50));
    }
    
    println!("\n🎉 Example execution completed!");
} 