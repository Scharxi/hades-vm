use std::sync::Arc;
use hades_ir::{Type, Value, TypeChecker, Function, Instruction, Operation};
use hades_ir::error::Error;
use hades_ir::basic_block::BasicBlock;

#[test]
fn test_assignment_compatibility() {
    let checker = TypeChecker::new();

    // Test integer compatibility
    assert!(checker.check_assignment_compatible(&Type::Integer(64), &Type::Integer(32)).is_ok());
    assert!(checker.check_assignment_compatible(&Type::Integer(32), &Type::Integer(64)).is_err());
    
    // Test float compatibility
    assert!(checker.check_assignment_compatible(&Type::Float(64), &Type::Float(32)).is_ok());
    assert!(checker.check_assignment_compatible(&Type::Float(32), &Type::Float(64)).is_err());
    
    // Test pointer compatibility
    let int_ptr = Type::pointer(Arc::new(Type::Integer(32)));
    let void_ptr = Type::pointer(Arc::new(Type::Void));
    assert!(checker.check_assignment_compatible(&int_ptr, &void_ptr).is_ok());
    assert!(checker.check_assignment_compatible(&void_ptr, &int_ptr).is_err());
}

#[test]
fn test_array_compatibility() {
    let checker = TypeChecker::new();
    
    let arr1 = Type::array(Arc::new(Type::Integer(32)), 10);
    let arr2 = Type::array(Arc::new(Type::Integer(32)), 10);
    let arr3 = Type::array(Arc::new(Type::Integer(32)), 5);
    
    assert!(checker.check_assignment_compatible(&arr1, &arr2).is_ok());
    assert!(checker.check_assignment_compatible(&arr1, &arr3).is_err());
}

#[test]
fn test_function_compatibility() {
    let checker = TypeChecker::new();
    
    let fn1 = Type::function(
        Arc::new(Type::Integer(32)),
        vec![Arc::new(Type::Float(64))],
        false
    );
    let fn2 = Type::function(
        Arc::new(Type::Integer(32)),
        vec![Arc::new(Type::Float(64))],
        false
    );
    let fn3 = Type::function(
        Arc::new(Type::Integer(32)),
        vec![Arc::new(Type::Float(32))],
        false
    );
    
    assert!(checker.check_assignment_compatible(&fn1, &fn2).is_ok());
    assert!(checker.check_assignment_compatible(&fn1, &fn3).is_err());
}

#[test]
fn test_struct_compatibility() {
    let checker = TypeChecker::new();
    
    let struct1 = Type::structure(
        "Point",
        vec![Arc::new(Type::Integer(32)), Arc::new(Type::Integer(32))],
        vec!["x".to_string(), "y".to_string()]
    );
    let struct2 = Type::structure(
        "Point",
        vec![Arc::new(Type::Integer(32)), Arc::new(Type::Integer(32))],
        vec!["x".to_string(), "y".to_string()]
    );
    let struct3 = Type::structure(
        "Point3D",
        vec![Arc::new(Type::Integer(32)), Arc::new(Type::Integer(32)), Arc::new(Type::Integer(32))],
        vec!["x".to_string(), "y".to_string(), "z".to_string()]
    );
    
    assert!(checker.check_assignment_compatible(&struct1, &struct2).is_ok());
    assert!(checker.check_assignment_compatible(&struct1, &struct3).is_err());
}

#[test]
fn test_numeric_operands() {
    let checker = TypeChecker::new();
    
    let int32 = Value::integer_constant(42, Arc::new(Type::Integer(32)));
    let float64 = Value::float_constant(3.14, Arc::new(Type::Float(64)));
    let bool_val = Value::boolean_constant(true);
    
    // Test valid numeric operations
    assert!(checker.check_numeric_operands(&int32, &int32).is_ok());
    assert!(checker.check_numeric_operands(&float64, &float64).is_ok());
    
    // Test invalid numeric operations
    assert!(checker.check_numeric_operands(&int32, &float64).is_err());
    assert!(checker.check_numeric_operands(&int32, &bool_val).is_err());
}

#[test]
fn test_instruction_type_checking() {
    let checker = TypeChecker::new();
    
    let int32_ty = Arc::new(Type::Integer(32));
    let int32_val = Value::integer_constant(42, int32_ty.clone());
    let int32_val2 = Value::integer_constant(24, int32_ty.clone());
    
    // Test binary operation (add)
    let add_inst = Instruction::new(
        Operation::Add,
        vec![int32_val.clone(), int32_val2.clone()],
        Some(int32_ty.clone())
    );
    assert!(checker.check_instruction(&add_inst).is_ok());
    
    // Test load operation
    let ptr_ty = Arc::new(Type::pointer(int32_ty.clone()));
    let ptr_val = Value::null_pointer(int32_ty.clone());
    
    let load_inst = Instruction::new(
        Operation::Load,
        vec![ptr_val.clone()],
        Some(int32_ty.clone())
    );
    assert!(checker.check_instruction(&load_inst).is_ok());
    
    // Test store operation
    let store_inst = Instruction::new(
        Operation::Store,
        vec![int32_val, ptr_val],
        None
    );
    assert!(checker.check_instruction(&store_inst).is_ok());
}

#[test]
fn test_function_type_checking() {
    let checker = TypeChecker::new();
    
    // Create a simple function that takes an i32 and returns an i32
    let int32_ty = Arc::new(Type::Integer(32));
    let fn_type = Type::function(
        int32_ty.clone(),
        vec![int32_ty.clone()],
        false
    );
    
    let mut function = Function::new("test".to_string(), fn_type, Default::default()).unwrap();
    
    // Add a parameter
    function.add_parameter(Some("x".to_string()), int32_ty.clone()).unwrap();
    
    // Add a basic block
    let mut block = BasicBlock::named("entry");
    
    // Add a return instruction with correct type
    let return_value = Value::integer_constant(42, int32_ty);
    let ret_inst = Instruction::new(Operation::Ret, vec![return_value], None);
    block.add_instruction(ret_inst);
    
    function.add_basic_block(block);
    
    assert!(checker.check_function(&function).is_ok());
}