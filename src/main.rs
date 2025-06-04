use vm::{
    instruction::{LoadConstant, Add, Multiply, Print, StoreMemory, LoadMemory, IntoRaw},
    vm::VirtualMachine
};

fn main() {
    println!("Hades VM Example Program");
    println!("========================\n");
    
    // Create a new virtual machine
    let mut vm = VirtualMachine::new(false);
    
    // Program to calculate Fibonacci sequence directly
    let program = [
        // Print F(0) = 0
        LoadConstant(0).into_raw().as_i32(),
        Print.into_raw().as_i32(),
        
        // Print F(1) = 1
        LoadConstant(1).into_raw().as_i32(),
        Print.into_raw().as_i32(),
        
        // Print F(2) = 1
        LoadConstant(1).into_raw().as_i32(),
        Print.into_raw().as_i32(),
        
        // Store F(1) = 1
        LoadConstant(1).into_raw().as_i32(),
        StoreMemory(1).into_raw().as_i32(),
        
        // Store F(2) = 1
        LoadConstant(1).into_raw().as_i32(),
        StoreMemory(2).into_raw().as_i32(),
        
        // Calculate and print F(3) = F(2) + F(1) = 1 + 1 = 2
        LoadMemory(1).into_raw().as_i32(), // F(1)
        LoadMemory(2).into_raw().as_i32(), // F(2)
        Add.into_raw().as_i32(),
        Print.into_raw().as_i32(),
        
        // Store F(3) and update F(1) and F(2)
        // Store F(3) temporarily
        StoreMemory(3).into_raw().as_i32(),
        
        // Update F(1) = F(2)
        LoadMemory(2).into_raw().as_i32(),
        StoreMemory(1).into_raw().as_i32(),
        
        // Update F(2) = F(3)
        LoadMemory(3).into_raw().as_i32(),
        StoreMemory(2).into_raw().as_i32(),
        
        // Calculate and print F(4) = F(3) + F(2) = 2 + 1 = 3
        LoadMemory(1).into_raw().as_i32(), // F(2)
        LoadMemory(2).into_raw().as_i32(), // F(3)
        Add.into_raw().as_i32(),
        Print.into_raw().as_i32(),
        
        // Store F(4) and update F(2) and F(3)
        // Store F(4) temporarily
        StoreMemory(3).into_raw().as_i32(),
        
        // Update F(1) = F(2)
        LoadMemory(2).into_raw().as_i32(),
        StoreMemory(1).into_raw().as_i32(),
        
        // Update F(2) = F(4)
        LoadMemory(3).into_raw().as_i32(),
        StoreMemory(2).into_raw().as_i32(),
        
        // Calculate and print F(5) = F(4) + F(3) = 3 + 2 = 5
        LoadMemory(1).into_raw().as_i32(), // F(3)
        LoadMemory(2).into_raw().as_i32(), // F(4)
        Add.into_raw().as_i32(),
        Print.into_raw().as_i32(),
        
        // Store F(5) and update F(3) and F(4)
        // Store F(5) temporarily
        StoreMemory(3).into_raw().as_i32(),
        
        // Update F(1) = F(2)
        LoadMemory(2).into_raw().as_i32(),
        StoreMemory(1).into_raw().as_i32(),
        
        // Update F(2) = F(5)
        LoadMemory(3).into_raw().as_i32(),
        StoreMemory(2).into_raw().as_i32(),
        
        // Calculate and print F(6) = F(5) + F(4) = 5 + 3 = 8
        LoadMemory(1).into_raw().as_i32(), // F(4)
        LoadMemory(2).into_raw().as_i32(), // F(5)
        Add.into_raw().as_i32(),
        Print.into_raw().as_i32(),
        
        // Store F(6) and update F(4) and F(5)
        // Store F(6) temporarily
        StoreMemory(3).into_raw().as_i32(),
        
        // Update F(1) = F(2)
        LoadMemory(2).into_raw().as_i32(),
        StoreMemory(1).into_raw().as_i32(),
        
        // Update F(2) = F(6)
        LoadMemory(3).into_raw().as_i32(),
        StoreMemory(2).into_raw().as_i32(),
        
        // Calculate and print F(7) = F(6) + F(5) = 8 + 5 = 13
        LoadMemory(1).into_raw().as_i32(), // F(5)
        LoadMemory(2).into_raw().as_i32(), // F(6)
        Add.into_raw().as_i32(),
        Print.into_raw().as_i32(),
        
        // Store F(7) and update F(5) and F(6)
        // Store F(7) temporarily
        StoreMemory(3).into_raw().as_i32(),
        
        // Update F(1) = F(2)
        LoadMemory(2).into_raw().as_i32(),
        StoreMemory(1).into_raw().as_i32(),
        
        // Update F(2) = F(7)
        LoadMemory(3).into_raw().as_i32(),
        StoreMemory(2).into_raw().as_i32(),
        
        // Calculate and print F(8) = F(7) + F(6) = 13 + 8 = 21
        LoadMemory(1).into_raw().as_i32(), // F(6)
        LoadMemory(2).into_raw().as_i32(), // F(7)
        Add.into_raw().as_i32(),
        Print.into_raw().as_i32(),
        
        // Finally, demonstrate multiplication (3 * 4 = 12)
        LoadConstant(3).into_raw().as_i32(),
        LoadConstant(4).into_raw().as_i32(),
        Multiply.into_raw().as_i32(),
        Print.into_raw().as_i32(),
    ];
    
    // Convert our instructions to bytes
    let mut bytes = Vec::new();
    for instruction in program.iter() {
        bytes.extend_from_slice(&instruction.to_be_bytes());
    }
    
    // Load and run the program
    vm.load_program(&bytes);
    println!("Running Fibonacci sequence calculation (0 to 8):");
    vm.run_until_completion();
    
    println!("\nProgram execution completed!");
}
