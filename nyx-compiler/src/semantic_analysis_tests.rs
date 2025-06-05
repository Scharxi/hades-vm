#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_analyzer::*;
    use crate::ast::*;

    fn create_test_program(statements: Vec<Statement>) -> Program {
        Program {
            functions: vec![Function {
                name: "test".to_string(),
                visibility: Visibility::Public,
                parameters: vec![],
                return_type: Type::Void,
                body: Block { statements },
            }],
        }
    }

    #[test]
    fn test_int_method_calls() {
        let mut analyzer = SemanticAnalyzer::new();
        
        // Test Int.abs() method call
        let statements = vec![
            Statement::VariableDeclaration {
                name: "x".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::Literal(Literal::Integer(-5))),
            },
            Statement::VariableDeclaration {
                name: "result".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("x".to_string())),
                    method: "abs".to_string(),
                    arguments: vec![],
                }),
            },
        ];
        
        let program = create_test_program(statements);
        assert!(analyzer.analyze_program(&program).is_ok());
    }

    #[test]
    fn test_int_pow_method() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "base".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::Literal(Literal::Integer(2))),
            },
            Statement::VariableDeclaration {
                name: "result".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("base".to_string())),
                    method: "pow".to_string(),
                    arguments: vec![Expression::Literal(Literal::Integer(3))],
                }),
            },
        ];
        
        let program = create_test_program(statements);
        assert!(analyzer.analyze_program(&program).is_ok());
    }

    #[test]
    fn test_int_boolean_methods() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "num".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::Literal(Literal::Integer(42))),
            },
            Statement::VariableDeclaration {
                name: "is_even".to_string(),
                mutable: false,
                type_: Some(Type::Bool),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("num".to_string())),
                    method: "isEven".to_string(),
                    arguments: vec![],
                }),
            },
            Statement::VariableDeclaration {
                name: "is_prime".to_string(),
                mutable: false,
                type_: Some(Type::Bool),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("num".to_string())),
                    method: "isPrime".to_string(),
                    arguments: vec![],
                }),
            },
        ];
        
        let program = create_test_program(statements);
        assert!(analyzer.analyze_program(&program).is_ok());
    }

    #[test]
    fn test_int_type_conversions() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "num".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::Literal(Literal::Integer(42))),
            },
            Statement::VariableDeclaration {
                name: "as_float".to_string(),
                mutable: false,
                type_: Some(Type::Float),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("num".to_string())),
                    method: "toFloat".to_string(),
                    arguments: vec![],
                }),
            },
            Statement::VariableDeclaration {
                name: "as_string".to_string(),
                mutable: false,
                type_: Some(Type::String),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("num".to_string())),
                    method: "toString".to_string(),
                    arguments: vec![],
                }),
            },
        ];
        
        let program = create_test_program(statements);
        assert!(analyzer.analyze_program(&program).is_ok());
    }

    #[test]
    fn test_float_math_methods() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "angle".to_string(),
                mutable: false,
                type_: Some(Type::Float),
                initializer: Some(Expression::Literal(Literal::Float(1.57))),
            },
            Statement::VariableDeclaration {
                name: "sine".to_string(),
                mutable: false,
                type_: Some(Type::Float),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("angle".to_string())),
                    method: "sin".to_string(),
                    arguments: vec![],
                }),
            },
        ];
        
        let program = create_test_program(statements);
        assert!(analyzer.analyze_program(&program).is_ok());
    }

    #[test]
    fn test_float_rounding_methods() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "value".to_string(),
                mutable: false,
                type_: Some(Type::Float),
                initializer: Some(Expression::Literal(Literal::Float(3.14159))),
            },
            Statement::VariableDeclaration {
                name: "rounded".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("value".to_string())),
                    method: "round".to_string(),
                    arguments: vec![],
                }),
            },
            Statement::VariableDeclaration {
                name: "floored".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("value".to_string())),
                    method: "floor".to_string(),
                    arguments: vec![],
                }),
            },
        ];
        
        let program = create_test_program(statements);
        assert!(analyzer.analyze_program(&program).is_ok());
    }

    #[test]
    fn test_bool_logical_methods() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "a".to_string(),
                mutable: false,
                type_: Some(Type::Bool),
                initializer: Some(Expression::Literal(Literal::Boolean(true))),
            },
            Statement::VariableDeclaration {
                name: "b".to_string(),
                mutable: false,
                type_: Some(Type::Bool),
                initializer: Some(Expression::Literal(Literal::Boolean(false))),
            },
            Statement::VariableDeclaration {
                name: "and_result".to_string(),
                mutable: false,
                type_: Some(Type::Bool),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("a".to_string())),
                    method: "and".to_string(),
                    arguments: vec![Expression::Identifier("b".to_string())],
                }),
            },
            Statement::VariableDeclaration {
                name: "xor_result".to_string(),
                mutable: false,
                type_: Some(Type::Bool),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("a".to_string())),
                    method: "xor".to_string(),
                    arguments: vec![Expression::Identifier("b".to_string())],
                }),
            },
        ];
        
        let program = create_test_program(statements);
        assert!(analyzer.analyze_program(&program).is_ok());
    }

    #[test]
    fn test_string_methods() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "text".to_string(),
                mutable: false,
                type_: Some(Type::String),
                initializer: Some(Expression::Literal(Literal::String("Hello World".to_string()))),
            },
            Statement::VariableDeclaration {
                name: "length".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("text".to_string())),
                    method: "length".to_string(),
                    arguments: vec![],
                }),
            },
            Statement::VariableDeclaration {
                name: "upper".to_string(),
                mutable: false,
                type_: Some(Type::String),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("text".to_string())),
                    method: "toUpperCase".to_string(),
                    arguments: vec![],
                }),
            },
            Statement::VariableDeclaration {
                name: "contains_hello".to_string(),
                mutable: false,
                type_: Some(Type::Bool),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("text".to_string())),
                    method: "contains".to_string(),
                    arguments: vec![Expression::Literal(Literal::String("Hello".to_string()))],
                }),
            },
        ];
        
        let program = create_test_program(statements);
        assert!(analyzer.analyze_program(&program).is_ok());
    }

    #[test]
    fn test_char_methods() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "ch".to_string(),
                mutable: false,
                type_: Some(Type::Class("Char".to_string())),
                initializer: Some(Expression::ObjectCreation {
                    class_name: "Char".to_string(),
                    arguments: vec![Expression::Literal(Literal::String("A".to_string()))],
                }),
            },
            Statement::VariableDeclaration {
                name: "is_letter".to_string(),
                mutable: false,
                type_: Some(Type::Bool),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("ch".to_string())),
                    method: "isLetter".to_string(),
                    arguments: vec![],
                }),
            },
            Statement::VariableDeclaration {
                name: "lower".to_string(),
                mutable: false,
                type_: Some(Type::Class("Char".to_string())),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("ch".to_string())),
                    method: "toLowerCase".to_string(),
                    arguments: vec![],
                }),
            },
        ];
        
        let program = create_test_program(statements);
        assert!(analyzer.analyze_program(&program).is_ok());
    }

    #[test]
    fn test_invalid_method_call() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "num".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::Literal(Literal::Integer(42))),
            },
            Statement::VariableDeclaration {
                name: "result".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("num".to_string())),
                    method: "nonExistentMethod".to_string(),
                    arguments: vec![],
                }),
            },
        ];
        
        let program = create_test_program(statements);
        let result = analyzer.analyze_program(&program);
        assert!(result.is_err());
        
        if let Err(SemanticError::MethodNotFound { class, method }) = result {
            assert_eq!(class, "Int");
            assert_eq!(method, "nonExistentMethod");
        } else {
            panic!("Expected MethodNotFound error");
        }
    }

    #[test]
    fn test_wrong_argument_count() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "num".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::Literal(Literal::Integer(42))),
            },
            Statement::VariableDeclaration {
                name: "result".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("num".to_string())),
                    method: "pow".to_string(),
                    arguments: vec![], // pow requires 1 argument
                }),
            },
        ];
        
        let program = create_test_program(statements);
        let result = analyzer.analyze_program(&program);
        assert!(result.is_err());
        
        if let Err(SemanticError::InvalidArgumentCount { expected, found }) = result {
            assert_eq!(expected, 1);
            assert_eq!(found, 0);
        } else {
            panic!("Expected InvalidArgumentCount error");
        }
    }

    #[test]
    fn test_wrong_argument_type() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "num".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::Literal(Literal::Integer(42))),
            },
            Statement::VariableDeclaration {
                name: "result".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("num".to_string())),
                    method: "pow".to_string(),
                    arguments: vec![Expression::Literal(Literal::String("not a number".to_string()))],
                }),
            },
        ];
        
        let program = create_test_program(statements);
        let result = analyzer.analyze_program(&program);
        assert!(result.is_err());
        
        if let Err(SemanticError::TypeMismatch { expected, found }) = result {
            assert_eq!(expected, "Int");
            assert_eq!(found, "String");
        } else {
            panic!("Expected TypeMismatch error");
        }
    }

    #[test]
    fn test_primitive_type_constructors() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "int_from_string".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::ObjectCreation {
                    class_name: "Int".to_string(),
                    arguments: vec![Expression::Literal(Literal::String("42".to_string()))],
                }),
            },
            Statement::VariableDeclaration {
                name: "float_from_int".to_string(),
                mutable: false,
                type_: Some(Type::Float),
                initializer: Some(Expression::ObjectCreation {
                    class_name: "Float".to_string(),
                    arguments: vec![Expression::Literal(Literal::Integer(42))],
                }),
            },
            Statement::VariableDeclaration {
                name: "bool_from_string".to_string(),
                mutable: false,
                type_: Some(Type::Bool),
                initializer: Some(Expression::ObjectCreation {
                    class_name: "Bool".to_string(),
                    arguments: vec![Expression::Literal(Literal::String("true".to_string()))],
                }),
            },
        ];
        
        let program = create_test_program(statements);
        assert!(analyzer.analyze_program(&program).is_ok());
    }

    #[test]
    fn test_binary_operations_with_primitives() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "a".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::Literal(Literal::Integer(10))),
            },
            Statement::VariableDeclaration {
                name: "b".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::Literal(Literal::Integer(5))),
            },
            Statement::VariableDeclaration {
                name: "sum".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::Binary {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    operator: BinaryOperator::Add,
                    right: Box::new(Expression::Identifier("b".to_string())),
                }),
            },
            Statement::VariableDeclaration {
                name: "is_equal".to_string(),
                mutable: false,
                type_: Some(Type::Bool),
                initializer: Some(Expression::Binary {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    operator: BinaryOperator::Equal,
                    right: Box::new(Expression::Identifier("b".to_string())),
                }),
            },
        ];
        
        let program = create_test_program(statements);
        assert!(analyzer.analyze_program(&program).is_ok());
    }

    #[test]
    fn test_mixed_arithmetic_types() {
        let mut analyzer = SemanticAnalyzer::new();
        
        let statements = vec![
            Statement::VariableDeclaration {
                name: "int_val".to_string(),
                mutable: false,
                type_: Some(Type::Int),
                initializer: Some(Expression::Literal(Literal::Integer(10))),
            },
            Statement::VariableDeclaration {
                name: "float_val".to_string(),
                mutable: false,
                type_: Some(Type::Float),
                initializer: Some(Expression::Literal(Literal::Float(3.14))),
            },
            Statement::VariableDeclaration {
                name: "result".to_string(),
                mutable: false,
                type_: Some(Type::Float),
                initializer: Some(Expression::Binary {
                    left: Box::new(Expression::Identifier("int_val".to_string())),
                    operator: BinaryOperator::Add,
                    right: Box::new(Expression::Identifier("float_val".to_string())),
                }),
            },
        ];
        
        let program = create_test_program(statements);
        assert!(analyzer.analyze_program(&program).is_ok());
    }
} 