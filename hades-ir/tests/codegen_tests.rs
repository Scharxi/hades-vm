use std::sync::Arc;
use hades_ir::{
    Type,
    Value,
    Function,
    BasicBlock,
    Instruction,
    Operation,
    CodeGenerator,
    Linkage,
};

#[test]
fn test_simple_function_codegen() {
    // Create a simple function that adds two integers
    let i32_type = Type::i32();
    let fn_type = Type::function(
        i32_type.clone(),
        vec![i32_type.clone(), i32_type.clone()],
        false
    );
    
    let mut function = Function::new("add".to_string(), fn_type, Linkage::External).unwrap();
    
    // Add parameters
    let param1 = function.add_parameter(Some("a".to_string()), i32_type.clone()).unwrap();
    let param2 = function.add_parameter(Some("b".to_string()), i32_type.clone()).unwrap();
    
    // Create basic block
    let mut block = BasicBlock::named("entry");
    
    // Create add instruction
    let add_inst = Instruction::new(
        Operation::Add,
        vec![param1.to_value(), param2.to_value()],
        Some(i32_type.clone())
    );
    
    // Store the ID before moving the instruction
    let add_id = add_inst.id();
    block.add_instruction(add_inst);
    
    // Create return instruction
    let result = Value::Instruction {
        id: add_id,
        ty: i32_type,
    };
    let ret_inst = Instruction::new(Operation::Ret, vec![result], None);
    block.add_instruction(ret_inst);
    
    // Add block to function
    function.add_basic_block(block);
    
    // Generate code
    let generator = CodeGenerator::new();
    let (bytecode, constants) = generator.generate(&function).unwrap();
    
    // Verify bytecode structure
    assert!(!bytecode.is_empty());
    
    // Expected bytecode layout:
    // [0] ENTER opcode (0x00)
    // [1-4] Number of parameters (2) as u32 LE
    // [5] LOAD_ARG opcode (0x02)
    // [6-13] First argument ID (8 bytes)
    // [14] LOAD_ARG opcode (0x02)
    // [15-22] Second argument ID (8 bytes)
    // [23] ADD opcode (0x10)
    // [24] RET opcode (0xFF)
    
    assert_eq!(bytecode[0], 0x00); // ENTER opcode
    assert_eq!(&bytecode[1..5], &(2u32).to_le_bytes()); // 2 parameters
    
    assert_eq!(bytecode[5], 0x02); // LOAD_ARG opcode for first parameter
    // Skip 8 bytes for argument ID
    
    assert_eq!(bytecode[14], 0x02); // LOAD_ARG opcode for second parameter
    // Skip 8 bytes for argument ID
    
    assert_eq!(bytecode[23], 0x10); // ADD opcode
    assert_eq!(bytecode[24], 0xFF); // RET opcode
    
    // Verify constant pool
    assert!(constants.is_empty()); // This example doesn't use any constants
}

#[test]
fn test_constant_loading() {
    // Create a function that returns a constant
    let i32_type = Type::i32();
    let fn_type = Type::function(i32_type.clone(), vec![], false);
    let mut function = Function::new("const42".to_string(), fn_type, Linkage::External).unwrap();
    
    // Create basic block
    let mut block = BasicBlock::named("entry");
    
    // Create constant
    let const_val = Value::integer_constant(42, i32_type.clone());
    
    // Create return instruction
    let ret_inst = Instruction::new(Operation::Ret, vec![const_val], None);
    block.add_instruction(ret_inst);
    
    // Add block to function
    function.add_basic_block(block);
    
    // Generate code
    let generator = CodeGenerator::new();
    let (bytecode, constants) = generator.generate(&function).unwrap();
    
    // Verify bytecode structure
    assert!(!bytecode.is_empty());
    assert_eq!(bytecode[0], 0x00); // ENTER opcode
    assert_eq!(&bytecode[1..5], &(0u32).to_le_bytes()); // 0 parameters
    
    // Should have LOAD_CONST followed by RET
    assert_eq!(bytecode[5], 0x01); // LOAD_CONST opcode
    assert_eq!(&bytecode[6..10], &(0u32).to_le_bytes()); // Constant index 0
    assert_eq!(bytecode[10], 0xFF); // RET opcode
    
    // Verify constant pool
    assert_eq!(constants.len(), 1);
    match &constants[0] {
        Value::IntegerConstant { value, .. } => assert_eq!(*value, 42),
        _ => panic!("Expected integer constant"),
    }
}

#[test]
fn test_memory_operations() {
    // Create a function that stores a value to memory and loads it back
    let i32_type = Type::i32();
    let ptr_type = Type::pointer(i32_type.clone());
    let fn_type = Type::function(i32_type.clone(), vec![ptr_type.clone()], false);
    
    let mut function = Function::new("load_store".to_string(), fn_type, Linkage::External).unwrap();
    
    // Add parameter (pointer)
    let ptr_param = function.add_parameter(Some("ptr".to_string()), ptr_type).unwrap();
    
    // Create basic block
    let mut block = BasicBlock::named("entry");
    
    // Create constant to store
    let const_val = Value::integer_constant(42, i32_type.clone());
    
    // Create store instruction
    let store_inst = Instruction::new(
        Operation::Store,
        vec![const_val, ptr_param.to_value()],
        None
    );
    block.add_instruction(store_inst);
    
    // Create load instruction
    let load_inst = Instruction::new(
        Operation::Load,
        vec![ptr_param.to_value()],
        Some(i32_type.clone())
    );
    
    // Store the ID before moving the instruction
    let load_id = load_inst.id();
    block.add_instruction(load_inst);
    
    // Create return instruction
    let result = Value::Instruction {
        id: load_id,
        ty: i32_type,
    };
    let ret_inst = Instruction::new(Operation::Ret, vec![result], None);
    block.add_instruction(ret_inst);
    
    // Add block to function
    function.add_basic_block(block);
    
    // Generate code
    let generator = CodeGenerator::new();
    let (bytecode, constants) = generator.generate(&function).unwrap();
    
    // Verify bytecode structure
    assert!(!bytecode.is_empty());
    
    // Expected bytecode layout:
    // [0] ENTER opcode (0x00)
    // [1-4] Number of parameters (1) as u32 LE
    // [5] LOAD_CONST opcode (0x01)
    // [6-9] Constant index (0) as u32 LE
    // [10] LOAD_ARG opcode (0x02)
    // [11-18] Argument ID (8 bytes)
    // [19] STORE opcode (0x21)
    // [20] LOAD_ARG opcode (0x02)
    // [21-28] Argument ID (8 bytes)
    // [29] LOAD opcode (0x20)
    // [30] RET opcode (0xFF)
    
    assert_eq!(bytecode[0], 0x00); // ENTER opcode
    assert_eq!(&bytecode[1..5], &(1u32).to_le_bytes()); // 1 parameter
    
    assert_eq!(bytecode[5], 0x01); // LOAD_CONST opcode
    assert_eq!(&bytecode[6..10], &(0u32).to_le_bytes()); // Constant index 0
    
    assert_eq!(bytecode[10], 0x02); // LOAD_ARG opcode
    // Skip 8 bytes for argument ID
    
    assert_eq!(bytecode[19], 0x21); // STORE opcode
    
    assert_eq!(bytecode[20], 0x02); // LOAD_ARG opcode
    // Skip 8 bytes for argument ID
    
    assert_eq!(bytecode[29], 0x20); // LOAD opcode
    assert_eq!(bytecode[30], 0xFF); // RET opcode
    
    // Verify constant pool
    assert_eq!(constants.len(), 1);
    match &constants[0] {
        Value::IntegerConstant { value, .. } => assert_eq!(*value, 42),
        _ => panic!("Expected integer constant"),
    }
} 