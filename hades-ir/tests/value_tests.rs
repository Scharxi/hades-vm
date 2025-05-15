use std::sync::Arc;
use hades_ir::{Value, Type, BasicBlock, Function, Linkage};

#[test]
fn test_constant_values() {
    let i32_type = Type::i32();
    let f64_type = Type::f64();
    
    // Test integer constant
    let int_const = Value::integer_constant(42, i32_type.clone());
    if let Value::IntegerConstant { value, ty } = &int_const {
        assert_eq!(*value, 42);
        assert!(Arc::ptr_eq(&ty, &i32_type));
    } else {
        panic!("Expected integer constant");
    }
    
    // Test float constant
    let float_const = Value::float_constant(3.14, f64_type.clone());
    if let Value::FloatConstant { value, ty } = &float_const {
        assert_eq!(*value, 3.14);
        assert!(Arc::ptr_eq(&ty, &f64_type));
    } else {
        panic!("Expected float constant");
    }
    
    // Test bool constant
    let bool_const = Value::boolean_constant(true);
    if let Value::BooleanConstant { value } = &bool_const {
        assert!(*value);
    } else {
        panic!("Expected boolean constant");
    }
}

#[test]
fn test_function_value() {
    let i32_type = Type::i32();
    let fn_type = Type::function(i32_type.clone(), vec![i32_type.clone()], false);
    let function = Function::new("test_fn".to_string(), fn_type.clone(), Linkage::External).unwrap();
    let fn_value = function.to_value();
    
    if let Value::Function { name, ty, .. } = &fn_value {
        assert_eq!(name, "test_fn");
        assert!(Arc::ptr_eq(&ty, &fn_type));
    } else {
        panic!("Expected function value");
    }
}

#[test]
fn test_basic_block_value() {
    let block = BasicBlock::named("test_block");
    let block_value = block.to_value();
    
    if let Value::BasicBlock { name, .. } = &block_value {
        assert_eq!(name.as_ref().unwrap(), "test_block");
    } else {
        panic!("Expected basic block value");
    }
}

#[test]
fn test_argument_value() {
    let i32_type = Type::i32();
    let fn_type = Type::function(i32_type.clone(), vec![i32_type.clone()], false);
    let mut function = Function::new("test_fn".to_string(), fn_type, Linkage::External).unwrap();
    
    let param = function.add_parameter(Some("arg".to_string()), i32_type.clone()).unwrap();
    let arg_value = param.to_value();
    
    if let Value::Argument { name, ty, index, .. } = &arg_value {
        assert_eq!(name.as_ref().unwrap(), "arg");
        assert!(Arc::ptr_eq(&ty, &i32_type));
        assert_eq!(index, &0_usize);
    } else {
        panic!("Expected argument value");
    }
}

#[test]
fn test_value_uniqueness() {
    let i32_type = Type::i32();
    let const1 = Value::integer_constant(42, i32_type.clone());
    let const2 = Value::integer_constant(42, i32_type.clone());
    
    // Even with same value and type, values should be distinct
    assert!(!std::ptr::eq(&const1, &const2));
}

#[test]
fn test_value_type_consistency() {
    let i32_type = Type::i32();
    let f64_type = Type::f64();
    
    // Test that values maintain their type information correctly
    let int_const = Value::integer_constant(42, i32_type.clone());
    let float_const = Value::float_constant(3.14, f64_type.clone());
    
    match (&int_const, &float_const) {
        (Value::IntegerConstant { ty: ty1, .. }, Value::FloatConstant { ty: ty2, .. }) => {
            assert!(Arc::ptr_eq(&ty1, &i32_type));
            assert!(Arc::ptr_eq(&ty2, &f64_type));
        }
        _ => panic!("Expected integer and float constants"),
    }
} 