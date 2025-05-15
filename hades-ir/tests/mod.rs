mod test_utils;
mod type_tests;
mod value_tests;
mod basic_block_tests;
mod function_tests;

// More test modules will be added here as we create them 

// Integration test that exercises multiple components together
#[test]
fn test_complete_ir_workflow() {
    use test_utils::*;
    use hades_ir::{Type, Value, BasicBlock, Operation, Instruction};
    
    // Create context and module
    let context = create_test_context();
    let mut module = create_test_module(context.clone(), "test_module");
    
    // Create function type and function
    let fn_type = create_test_function_type(context.clone());
    let mut function = create_test_function("add_or_mul", fn_type.clone()).unwrap();
    
    // Add parameters
    let i32_type = Type::i32();
    let param_a = function.add_parameter(Some("a".to_string()), i32_type.clone()).unwrap();
    let param_b = function.add_parameter(Some("b".to_string()), i32_type.clone()).unwrap();
    
    // Create basic blocks
    let mut entry = BasicBlock::named("entry");
    let mut then_block = BasicBlock::named("then");
    let mut else_block = BasicBlock::named("else");
    let mut exit = BasicBlock::named("exit");
    
    // Build entry block
    let a_val = param_a.to_value();
    let b_val = param_b.to_value();
    let ten = Value::integer_constant(10, i32_type.clone());
    let cond = Instruction::compare(Operation::Gt, a_val.clone(), ten);
    entry.add_instruction(cond.clone());
    entry.add_instruction(Instruction::cond_br(
        cond.to_value(),
        then_block.to_value(),
        else_block.to_value()
    ));
    
    // Build then block (add)
    let add = Instruction::binary_op(Operation::Add, a_val.clone(), b_val.clone(), i32_type.clone());
    then_block.add_instruction(add.clone());
    then_block.add_instruction(Instruction::br(exit.to_value()));
    
    // Build else block (multiply)
    let mul = Instruction::binary_op(Operation::Mul, a_val, b_val, i32_type.clone());
    else_block.add_instruction(mul.clone());
    else_block.add_instruction(Instruction::br(exit.to_value()));
    
    // Build exit block
    let phi = Instruction::phi(
        i32_type.clone(),
        vec![
            (add.to_value(), then_block.to_value()),
            (mul.to_value(), else_block.to_value())
        ]
    );
    exit.add_instruction(phi.clone());
    exit.add_instruction(Instruction::ret(Some(phi.to_value())));
    
    // Add blocks to function
    function.add_basic_block(entry);
    function.add_basic_block(then_block);
    function.add_basic_block(else_block);
    function.add_basic_block(exit);
    
    // Add function to module
    module.add_function(function).unwrap();
    
    // Verify the module
    assert!(module.verify().is_ok());
} 