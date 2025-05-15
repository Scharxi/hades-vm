use std::sync::Arc;
use hades_ir::Type;

#[test]
fn test_integer_types() {
    assert_eq!(&*Type::i32(), &Type::Integer(32));
    assert_eq!(&*Type::i64(), &Type::Integer(64));
}

#[test]
fn test_float_types() {
    assert_eq!(&*Type::f32(), &Type::Float(32));
    assert_eq!(&*Type::f64(), &Type::Float(64));
}

#[test]
fn test_boolean_type() {
    assert_eq!(&*Type::boolean(), &Type::Boolean);
}

#[test]
fn test_pointer_type() {
    let pointee = Type::i32();
    let ptr = Type::pointer(pointee.clone());
    assert!(matches!(&*ptr, Type::Pointer(p) if Arc::ptr_eq(p, &pointee)));
}

#[test]
fn test_array_type() {
    let element = Type::i32();
    let size = 10;
    let array = Type::array(element.clone(), size);
    assert!(matches!(&*array, Type::Array { element_type, size: s } 
        if Arc::ptr_eq(element_type, &element) && *s == size));
}

#[test]
fn test_struct_type() {
    let field_types = vec![Type::i32(), Type::boolean()];
    let field_names = vec!["x".to_string(), "y".to_string()];
    let struct_type = Type::structure("Point", field_types.clone(), field_names.clone());
    
    assert!(matches!(&*struct_type, Type::Struct { 
        name,
        field_types: ft,
        field_names: fn_
    } if name == "Point" 
        && ft.len() == field_types.len()
        && fn_.len() == field_names.len()
        && ft.iter().zip(&field_types).all(|(a, b)| Arc::ptr_eq(a, b))
        && fn_ == &field_names));
}

#[test]
fn test_function_type() {
    let return_type = Type::i32();
    let param_types = vec![Type::boolean(), Type::f64()];
    let func_type = Type::function(return_type.clone(), param_types.clone(), false);
    
    assert!(matches!(&*func_type, Type::Function {
        return_type: rt,
        param_types: pt,
        is_variadic
    } if Arc::ptr_eq(rt, &return_type)
        && pt.len() == param_types.len()
        && pt.iter().zip(&param_types).all(|(a, b)| Arc::ptr_eq(a, b))
        && !is_variadic));
} 