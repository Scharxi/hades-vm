use std::sync::Arc;
use hades_ir::{BasicBlock, Instruction, Operation, Type, Value};

#[test]
fn test_basic_block_creation() {
    // Test named block
    let named_block = BasicBlock::named("entry");
    assert_eq!(named_block.name(), Some("entry"));
    assert!(named_block.instructions().is_empty());
    
    // Test unnamed block
    let unnamed_block = BasicBlock::unnamed();
    assert_eq!(unnamed_block.name(), None);
    assert!(unnamed_block.instructions().is_empty());
}

#[test]
fn test_basic_block_instructions() {
    let mut block = BasicBlock::named("test");
    let i32_type = Type::i32();
    
    // Create some test instructions
    let val1 = Value::integer_constant(1, i32_type.clone());
    let val2 = Value::integer_constant(2, i32_type.clone());
    
    let add_inst = Instruction::binary_op(
        Operation::Add,
        val1.clone(),
        val2.clone(),
        i32_type.clone()
    );
    
    let ret_inst = Instruction::ret(Some(val1));
    
    // Add instructions
    block.add_instruction(add_inst.clone());
    block.add_instruction(ret_inst.clone());
    
    // Test instruction access
    assert_eq!(block.len(), 2);
    assert_eq!(block.get_instruction(0).unwrap().operation(), &Operation::Add);
    assert_eq!(block.get_instruction(1).unwrap().operation(), &Operation::Ret);
}

#[test]
fn test_basic_block_terminator() {
    let mut block = BasicBlock::named("test");
    let i32_type = Type::i32();
    let val = Value::integer_constant(42, i32_type.clone());
    
    // Initially no terminator
    assert!(!block.has_terminator());
    
    // Add non-terminator instruction
    let add_inst = Instruction::binary_op(
        Operation::Add,
        val.clone(),
        val.clone(),
        i32_type.clone()
    );
    block.add_instruction(add_inst);
    assert!(!block.has_terminator());
    
    // Add terminator instruction
    let ret_inst = Instruction::ret(Some(val));
    block.add_instruction(ret_inst);
    assert!(block.has_terminator());
    
    // Check terminator
    let terminator = block.get_terminator().unwrap();
    assert!(matches!(terminator.operation(), &Operation::Ret));
}

#[test]
fn test_basic_block_metadata() {
    let mut block = BasicBlock::named("test");
    
    // Test metadata operations
    block.add_metadata("author", "test");
    block.add_metadata("version", "1.0");
    
    assert_eq!(block.get_metadata("author").unwrap(), "test");
    assert_eq!(block.get_metadata("version").unwrap(), "1.0");
    assert_eq!(block.get_metadata("nonexistent"), None);
}

#[test]
fn test_basic_block_instruction_manipulation() {
    let mut block = BasicBlock::named("test");
    let i32_type = Type::i32();
    let val = Value::integer_constant(42, i32_type.clone());
    
    // Add some instructions
    let inst1 = Instruction::binary_op(Operation::Add, val.clone(), val.clone(), i32_type.clone());
    let inst2 = Instruction::binary_op(Operation::Mul, val.clone(), val.clone(), i32_type.clone());
    let inst3 = Instruction::ret(Some(val));
    
    block.add_instruction(inst1);
    block.add_instruction(inst2);
    block.add_instruction(inst3);
    
    // Test instruction removal
    assert_eq!(block.len(), 3);
    block.remove_instruction(1);
    assert_eq!(block.len(), 2);
    assert!(matches!(block.get_instruction(0).unwrap().operation(), &Operation::Add));
    assert!(matches!(block.get_instruction(1).unwrap().operation(), &Operation::Ret));
    
    // Test instruction iteration
    let ops: Vec<_> = block.iter().map(|i| i.operation()).collect();
    assert_eq!(ops, &[&Operation::Add, &Operation::Ret]);
}

#[test]
fn test_basic_block_value_conversion() {
    let block = BasicBlock::named("test");
    let value = block.to_value();
    
    if let Value::BasicBlock { name, .. } = value {
        assert_eq!(name.unwrap(), "test");
    } else {
        panic!("Expected BasicBlock value");
    }
} 