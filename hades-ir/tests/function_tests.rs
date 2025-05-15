use std::sync::Arc;
use hades_ir::{Function, Type, Value, BasicBlock, Linkage, Instruction};

#[test]
fn test_function_creation() {
    let i32_type = Type::i32();
    let fn_type = Type::function(i32_type.clone(), vec![i32_type.clone()], false);
    let function = Function::new("test_fn".to_string(), fn_type.clone(), Linkage::External).unwrap();
    
    assert_eq!(function.name(), "test_fn");
    assert!(Arc::ptr_eq(&function.ty(), &fn_type));
    assert_eq!(function.linkage(), Linkage::External);
    assert!(function.parameters().is_empty());
    assert!(function.basic_blocks().is_empty());
}

#[test]
fn test_function_parameters() {
    let i32_type = Type::i32();
    let fn_type = Type::function(i32_type.clone(), vec![i32_type.clone(), i32_type.clone()], false);
    let mut function = Function::new("test_fn".to_string(), fn_type, Linkage::External).unwrap();
    
    // Add parameters
    let param1 = function.add_parameter(Some("a".to_string()), i32_type.clone()).unwrap();
    let param2 = function.add_parameter(Some("b".to_string()), i32_type.clone()).unwrap();
    
    // Check parameters
    let params = function.parameters();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name(), Some("a"));
    assert_eq!(params[1].name(), Some("b"));
    assert!(Arc::ptr_eq(&params[0].ty(), &i32_type));
    assert!(Arc::ptr_eq(&params[1].ty(), &i32_type));
    assert_eq!(params[0].index(), 0);
    assert_eq!(params[1].index(), 1);
}

#[test]
fn test_function_basic_blocks() {
    let i32_type = Type::i32();
    let fn_type = Type::function(i32_type.clone(), vec![i32_type.clone()], false);
    let mut function = Function::new("test_fn".to_string(), fn_type, Linkage::External).unwrap();
    
    // Add basic blocks
    let entry = BasicBlock::named("entry");
    let body = BasicBlock::named("body");
    let exit = BasicBlock::named("exit");
    
    function.add_basic_block(entry);
    function.add_basic_block(body);
    function.add_basic_block(exit);
    
    // Check basic blocks
    assert_eq!(function.basic_blocks().len(), 3);
    assert_eq!(function.entry_block().unwrap().name(), Some("entry"));
    
    // Test block lookup
    assert_eq!(function.find_basic_block_by_name("body").unwrap().name(), Some("body"));
}

#[test]
fn test_function_metadata() {
    let i32_type = Type::i32();
    let fn_type = Type::function(i32_type.clone(), vec![i32_type.clone()], false);
    let mut function = Function::new("test_fn".to_string(), fn_type, Linkage::External).unwrap();
    
    function.add_metadata("author", "test");
    function.add_metadata("version", "1.0");
    
    assert_eq!(function.get_metadata("author").unwrap(), "test");
    assert_eq!(function.get_metadata("version").unwrap(), "1.0");
    assert_eq!(function.get_metadata("nonexistent"), None);
}

#[test]
fn test_function_verification() {
    let i32_type = Type::i32();
    let fn_type = Type::function(i32_type.clone(), vec![i32_type.clone(), i32_type.clone()], false);
    let mut function = Function::new("test_fn".to_string(), fn_type, Linkage::External).unwrap();
    
    // Add parameters
    function.add_parameter(Some("a".to_string()), i32_type.clone()).unwrap();
    function.add_parameter(Some("b".to_string()), i32_type.clone()).unwrap();
    
    // Function without blocks should fail verification
    assert!(function.verify().is_err());
    
    // Add a basic block with a terminator
    let mut block = BasicBlock::named("entry");
    let val = Value::integer_constant(0, i32_type.clone());
    let ret_inst = Instruction::ret(Some(val));
    block.add_instruction(ret_inst);
    
    function.add_basic_block(block);
    
    // Now verification should pass
    assert!(function.verify().is_ok());
}

#[test]
fn test_function_value_conversion() {
    let i32_type = Type::i32();
    let fn_type = Type::function(i32_type.clone(), vec![i32_type.clone()], false);
    let function = Function::new("test_fn".to_string(), fn_type.clone(), Linkage::External).unwrap();
    
    let value = function.to_value();
    if let Value::Function { name, ty, .. } = value {
        assert_eq!(name, "test_fn");
        assert!(Arc::ptr_eq(&ty, &fn_type));
    } else {
        panic!("Expected Function value");
    }
}

#[test]
fn test_function_linkage() {
    let i32_type = Type::i32();
    let fn_type = Type::function(i32_type.clone(), vec![i32_type.clone()], false);
    let mut function = Function::new("test_fn".to_string(), fn_type, Linkage::External).unwrap();
    
    // Test default linkage
    assert_eq!(function.linkage(), Linkage::External);
    
    // Test linkage modification
    function.set_linkage(Linkage::Internal);
    assert_eq!(function.linkage(), Linkage::Internal);
    
    function.set_linkage(Linkage::InlineOnly);
    assert_eq!(function.linkage(), Linkage::InlineOnly);
    
    function.set_linkage(Linkage::LinkOnceODR);
    assert_eq!(function.linkage(), Linkage::LinkOnceODR);
} 