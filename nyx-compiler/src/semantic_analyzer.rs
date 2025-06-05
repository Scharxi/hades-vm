use crate::ast::*;
use std::collections::HashMap;
use std::sync::Arc;

/// Error types for semantic analysis
#[derive(Debug, Clone)]
pub enum SemanticError {
    UndefinedVariable(String),
    UndefinedFunction(String),
    UndefinedMethod { class: String, method: String },
    TypeMismatch { expected: String, found: String },
    InvalidOperation { operation: String, types: Vec<String> },
    ClassNotFound(String),
    MethodNotFound { class: String, method: String },
    InvalidArgumentCount { expected: usize, found: usize },
    InvalidPropertyAccess { class: String, property: String },
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemanticError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            SemanticError::UndefinedFunction(name) => write!(f, "Undefined function: {}", name),
            SemanticError::UndefinedMethod { class, method } => {
                write!(f, "Undefined method '{}' on class '{}'", method, class)
            }
            SemanticError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected '{}', found '{}'", expected, found)
            }
            SemanticError::InvalidOperation { operation, types } => {
                write!(f, "Invalid operation '{}' for types: {:?}", operation, types)
            }
            SemanticError::ClassNotFound(name) => write!(f, "Class not found: {}", name),
            SemanticError::MethodNotFound { class, method } => {
                write!(f, "Method '{}' not found on class '{}'", method, class)
            }
            SemanticError::InvalidArgumentCount { expected, found } => {
                write!(f, "Invalid argument count: expected {}, found {}", expected, found)
            }
            SemanticError::InvalidPropertyAccess { class, property } => {
                write!(f, "Property '{}' not found on class '{}'", property, class)
            }
        }
    }
}

impl std::error::Error for SemanticError {}

pub type SemanticResult<T> = Result<T, SemanticError>;

/// Information about a method signature
#[derive(Debug, Clone)]
pub struct MethodSignature {
    pub name: String,
    pub params: Vec<Type>,
    pub return_type: Type,
    pub is_static: bool,
}

/// Information about a primitive type class
#[derive(Debug, Clone)]
pub struct PrimitiveTypeInfo {
    pub name: String,
    pub methods: HashMap<String, MethodSignature>,
    pub properties: HashMap<String, Type>,
    pub static_methods: HashMap<String, MethodSignature>,
    pub static_properties: HashMap<String, Type>,
}

impl PrimitiveTypeInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            methods: HashMap::new(),
            properties: HashMap::new(),
            static_methods: HashMap::new(),
            static_properties: HashMap::new(),
        }
    }

    pub fn add_method(&mut self, method: MethodSignature) {
        if method.is_static {
            self.static_methods.insert(method.name.clone(), method);
        } else {
            self.methods.insert(method.name.clone(), method);
        }
    }

    pub fn add_property(&mut self, name: String, type_: Type, is_static: bool) {
        if is_static {
            self.static_properties.insert(name, type_);
        } else {
            self.properties.insert(name, type_);
        }
    }
}

/// Semantic analyzer for the Nyx language
pub struct SemanticAnalyzer {
    /// Map of primitive type information
    primitive_types: HashMap<String, PrimitiveTypeInfo>,
    /// Current scope variables
    variables: HashMap<String, Type>,
    /// Current scope functions
    functions: HashMap<String, (Vec<Type>, Type)>, // (params, return_type)
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            primitive_types: HashMap::new(),
            variables: HashMap::new(),
            functions: HashMap::new(),
        };
        
        analyzer.initialize_primitive_types();
        analyzer
    }

    /// Initialize built-in primitive type information
    fn initialize_primitive_types(&mut self) {
        self.initialize_int_type();
        self.initialize_float_type();
        self.initialize_bool_type();
        self.initialize_string_type();
        self.initialize_char_type();
    }

    fn initialize_int_type(&mut self) {
        let mut int_type = PrimitiveTypeInfo::new("Int".to_string());
        
        // Instance methods
        int_type.add_method(MethodSignature {
            name: "abs".to_string(),
            params: vec![],
            return_type: Type::Int,
            is_static: false,
        });
        
        int_type.add_method(MethodSignature {
            name: "sign".to_string(),
            params: vec![],
            return_type: Type::Int,
            is_static: false,
        });
        
        int_type.add_method(MethodSignature {
            name: "pow".to_string(),
            params: vec![Type::Int],
            return_type: Type::Int,
            is_static: false,
        });
        
        int_type.add_method(MethodSignature {
            name: "isEven".to_string(),
            params: vec![],
            return_type: Type::Bool,
            is_static: false,
        });
        
        int_type.add_method(MethodSignature {
            name: "isOdd".to_string(),
            params: vec![],
            return_type: Type::Bool,
            is_static: false,
        });
        
        int_type.add_method(MethodSignature {
            name: "isPrime".to_string(),
            params: vec![],
            return_type: Type::Bool,
            is_static: false,
        });
        
        int_type.add_method(MethodSignature {
            name: "toFloat".to_string(),
            params: vec![],
            return_type: Type::Float,
            is_static: false,
        });
        
        int_type.add_method(MethodSignature {
            name: "toString".to_string(),
            params: vec![],
            return_type: Type::String,
            is_static: false,
        });
        
        // Static methods
        int_type.add_method(MethodSignature {
            name: "parse".to_string(),
            params: vec![Type::String],
            return_type: Type::Int,
            is_static: true,
        });
        
        int_type.add_method(MethodSignature {
            name: "min".to_string(),
            params: vec![Type::Int, Type::Int],
            return_type: Type::Int,
            is_static: true,
        });
        
        int_type.add_method(MethodSignature {
            name: "max".to_string(),
            params: vec![Type::Int, Type::Int],
            return_type: Type::Int,
            is_static: true,
        });
        
        // Static properties
        int_type.add_property("MIN_VALUE".to_string(), Type::Int, true);
        int_type.add_property("MAX_VALUE".to_string(), Type::Int, true);
        int_type.add_property("ZERO".to_string(), Type::Int, true);
        int_type.add_property("ONE".to_string(), Type::Int, true);
        
        self.primitive_types.insert("Int".to_string(), int_type);
    }

    fn initialize_float_type(&mut self) {
        let mut float_type = PrimitiveTypeInfo::new("Float".to_string());
        
        // Instance methods
        float_type.add_method(MethodSignature {
            name: "abs".to_string(),
            params: vec![],
            return_type: Type::Float,
            is_static: false,
        });
        
        float_type.add_method(MethodSignature {
            name: "sin".to_string(),
            params: vec![],
            return_type: Type::Float,
            is_static: false,
        });
        
        float_type.add_method(MethodSignature {
            name: "cos".to_string(),
            params: vec![],
            return_type: Type::Float,
            is_static: false,
        });
        
        float_type.add_method(MethodSignature {
            name: "sqrt".to_string(),
            params: vec![],
            return_type: Type::Float,
            is_static: false,
        });
        
        float_type.add_method(MethodSignature {
            name: "round".to_string(),
            params: vec![],
            return_type: Type::Int,
            is_static: false,
        });
        
        float_type.add_method(MethodSignature {
            name: "floor".to_string(),
            params: vec![],
            return_type: Type::Int,
            is_static: false,
        });
        
        float_type.add_method(MethodSignature {
            name: "ceil".to_string(),
            params: vec![],
            return_type: Type::Int,
            is_static: false,
        });
        
        float_type.add_method(MethodSignature {
            name: "isNaN".to_string(),
            params: vec![],
            return_type: Type::Bool,
            is_static: false,
        });
        
        float_type.add_method(MethodSignature {
            name: "isInfinite".to_string(),
            params: vec![],
            return_type: Type::Bool,
            is_static: false,
        });
        
        float_type.add_method(MethodSignature {
            name: "toInt".to_string(),
            params: vec![],
            return_type: Type::Int,
            is_static: false,
        });
        
        float_type.add_method(MethodSignature {
            name: "toString".to_string(),
            params: vec![],
            return_type: Type::String,
            is_static: false,
        });
        
        // Static methods
        float_type.add_method(MethodSignature {
            name: "parse".to_string(),
            params: vec![Type::String],
            return_type: Type::Float,
            is_static: true,
        });
        
        // Static properties
        float_type.add_property("NaN".to_string(), Type::Float, true);
        float_type.add_property("POSITIVE_INFINITY".to_string(), Type::Float, true);
        float_type.add_property("NEGATIVE_INFINITY".to_string(), Type::Float, true);
        float_type.add_property("PI".to_string(), Type::Float, true);
        float_type.add_property("E".to_string(), Type::Float, true);
        
        self.primitive_types.insert("Float".to_string(), float_type);
    }

    fn initialize_bool_type(&mut self) {
        let mut bool_type = PrimitiveTypeInfo::new("Bool".to_string());
        
        // Instance methods
        bool_type.add_method(MethodSignature {
            name: "and".to_string(),
            params: vec![Type::Bool],
            return_type: Type::Bool,
            is_static: false,
        });
        
        bool_type.add_method(MethodSignature {
            name: "or".to_string(),
            params: vec![Type::Bool],
            return_type: Type::Bool,
            is_static: false,
        });
        
        bool_type.add_method(MethodSignature {
            name: "xor".to_string(),
            params: vec![Type::Bool],
            return_type: Type::Bool,
            is_static: false,
        });
        
        bool_type.add_method(MethodSignature {
            name: "not".to_string(),
            params: vec![],
            return_type: Type::Bool,
            is_static: false,
        });
        
        bool_type.add_method(MethodSignature {
            name: "toString".to_string(),
            params: vec![],
            return_type: Type::String,
            is_static: false,
        });
        
        // Static methods
        bool_type.add_method(MethodSignature {
            name: "parse".to_string(),
            params: vec![Type::String],
            return_type: Type::Bool,
            is_static: true,
        });
        
        self.primitive_types.insert("Bool".to_string(), bool_type);
    }

    fn initialize_string_type(&mut self) {
        let mut string_type = PrimitiveTypeInfo::new("String".to_string());
        
        // Instance methods
        string_type.add_method(MethodSignature {
            name: "length".to_string(),
            params: vec![],
            return_type: Type::Int,
            is_static: false,
        });
        
        string_type.add_method(MethodSignature {
            name: "substring".to_string(),
            params: vec![Type::Int, Type::Int],
            return_type: Type::String,
            is_static: false,
        });
        
        string_type.add_method(MethodSignature {
            name: "concat".to_string(),
            params: vec![Type::String],
            return_type: Type::String,
            is_static: false,
        });
        
        string_type.add_method(MethodSignature {
            name: "toLowerCase".to_string(),
            params: vec![],
            return_type: Type::String,
            is_static: false,
        });
        
        string_type.add_method(MethodSignature {
            name: "toUpperCase".to_string(),
            params: vec![],
            return_type: Type::String,
            is_static: false,
        });
        
        string_type.add_method(MethodSignature {
            name: "trim".to_string(),
            params: vec![],
            return_type: Type::String,
            is_static: false,
        });
        
        string_type.add_method(MethodSignature {
            name: "indexOf".to_string(),
            params: vec![Type::String],
            return_type: Type::Int,
            is_static: false,
        });
        
        string_type.add_method(MethodSignature {
            name: "contains".to_string(),
            params: vec![Type::String],
            return_type: Type::Bool,
            is_static: false,
        });
        
        string_type.add_method(MethodSignature {
            name: "startsWith".to_string(),
            params: vec![Type::String],
            return_type: Type::Bool,
            is_static: false,
        });
        
        string_type.add_method(MethodSignature {
            name: "endsWith".to_string(),
            params: vec![Type::String],
            return_type: Type::Bool,
            is_static: false,
        });
        
        self.primitive_types.insert("String".to_string(), string_type);
    }

    fn initialize_char_type(&mut self) {
        let mut char_type = PrimitiveTypeInfo::new("Char".to_string());
        
        // Instance methods
        char_type.add_method(MethodSignature {
            name: "isLetter".to_string(),
            params: vec![],
            return_type: Type::Bool,
            is_static: false,
        });
        
        char_type.add_method(MethodSignature {
            name: "isDigit".to_string(),
            params: vec![],
            return_type: Type::Bool,
            is_static: false,
        });
        
        char_type.add_method(MethodSignature {
            name: "isWhitespace".to_string(),
            params: vec![],
            return_type: Type::Bool,
            is_static: false,
        });
        
        char_type.add_method(MethodSignature {
            name: "toLowerCase".to_string(),
            params: vec![],
            return_type: Type::Class("Char".to_string()),
            is_static: false,
        });
        
        char_type.add_method(MethodSignature {
            name: "toUpperCase".to_string(),
            params: vec![],
            return_type: Type::Class("Char".to_string()),
            is_static: false,
        });
        
        char_type.add_method(MethodSignature {
            name: "toString".to_string(),
            params: vec![],
            return_type: Type::String,
            is_static: false,
        });
        
        self.primitive_types.insert("Char".to_string(), char_type);
    }

    /// Analyze a program for semantic correctness
    pub fn analyze_program(&mut self, program: &Program) -> SemanticResult<()> {
        for function in &program.functions {
            self.analyze_function(function)?;
        }
        Ok(())
    }

    /// Analyze a function for semantic correctness
    pub fn analyze_function(&mut self, function: &Function) -> SemanticResult<()> {
        // Add function parameters to scope
        for param in &function.parameters {
            self.variables.insert(param.name.clone(), param.type_.clone());
        }
        
        // Analyze function body
        self.analyze_block(&function.body)?;
        
        // Clear function scope
        self.variables.clear();
        
        Ok(())
    }

    /// Analyze a block of statements
    pub fn analyze_block(&mut self, block: &Block) -> SemanticResult<()> {
        for statement in &block.statements {
            self.analyze_statement(statement)?;
        }
        Ok(())
    }

    /// Analyze a statement
    pub fn analyze_statement(&mut self, statement: &Statement) -> SemanticResult<()> {
        match statement {
            Statement::VariableDeclaration { name, type_, initializer, .. } => {
                if let Some(init_expr) = initializer {
                    let expr_type = self.analyze_expression(init_expr)?;
                    if let Some(expected_type) = type_ {
                        if !self.types_compatible(expected_type, &expr_type) {
                            return Err(SemanticError::TypeMismatch {
                                expected: format!("{}", expected_type),
                                found: format!("{}", expr_type),
                            });
                        }
                    }
                    self.variables.insert(name.clone(), expr_type);
                } else if let Some(var_type) = type_ {
                    self.variables.insert(name.clone(), var_type.clone());
                }
            }
            
            Statement::Assignment { name, value } => {
                let expr_type = self.analyze_expression(value)?;
                if let Some(var_type) = self.variables.get(name) {
                    if !self.types_compatible(var_type, &expr_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: format!("{}", var_type),
                            found: format!("{}", expr_type),
                        });
                    }
                } else {
                    return Err(SemanticError::UndefinedVariable(name.clone()));
                }
            }
            
            Statement::Expression(expr) => {
                self.analyze_expression(expr)?;
            }
            
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    self.analyze_expression(expr)?;
                }
            }
            
            Statement::If { condition, then_block, else_block } => {
                let cond_type = self.analyze_expression(condition)?;
                if !self.types_compatible(&Type::Bool, &cond_type) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Bool".to_string(),
                        found: format!("{}", cond_type),
                    });
                }
                self.analyze_block(then_block)?;
                if let Some(else_blk) = else_block {
                    self.analyze_block(else_blk)?;
                }
            }
            
            Statement::While { condition, body } => {
                let cond_type = self.analyze_expression(condition)?;
                if !self.types_compatible(&Type::Bool, &cond_type) {
                    return Err(SemanticError::TypeMismatch {
                        expected: "Bool".to_string(),
                        found: format!("{}", cond_type),
                    });
                }
                self.analyze_block(body)?;
            }
        }
        Ok(())
    }

    /// Analyze an expression and return its type
    pub fn analyze_expression(&mut self, expression: &Expression) -> SemanticResult<Type> {
        match expression {
            Expression::Literal(literal) => Ok(self.analyze_literal(literal)),
            
            Expression::Identifier(name) => {
                self.variables.get(name)
                    .cloned()
                    .ok_or_else(|| SemanticError::UndefinedVariable(name.clone()))
            }
            
            Expression::Binary { left, operator, right } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;
                self.analyze_binary_operation(&left_type, operator, &right_type)
            }
            
            Expression::Unary { operator, operand } => {
                let operand_type = self.analyze_expression(operand)?;
                self.analyze_unary_operation(operator, &operand_type)
            }
            
            Expression::Call { function, arguments } => {
                // Check if function exists and validate arguments
                if let Some((param_types, return_type)) = self.functions.get(function).cloned() {
                    if arguments.len() != param_types.len() {
                        return Err(SemanticError::InvalidArgumentCount {
                            expected: param_types.len(),
                            found: arguments.len(),
                        });
                    }
                    
                    for (i, arg) in arguments.iter().enumerate() {
                        let arg_type = self.analyze_expression(arg)?;
                        if !self.types_compatible(&param_types[i], &arg_type) {
                            return Err(SemanticError::TypeMismatch {
                                expected: format!("{}", param_types[i]),
                                found: format!("{}", arg_type),
                            });
                        }
                    }
                    
                    Ok(return_type)
                } else {
                    Err(SemanticError::UndefinedFunction(function.clone()))
                }
            }
            
            Expression::MethodCall { object, method, arguments } => {
                let object_type = self.analyze_expression(object)?;
                self.analyze_method_call(&object_type, method, arguments)
            }
            
            Expression::PropertyAccess { object, property } => {
                let object_type = self.analyze_expression(object)?;
                self.analyze_property_access(&object_type, property)
            }
            
            Expression::ObjectCreation { class_name, arguments } => {
                // For primitive types, this validates constructor arguments
                self.analyze_object_creation(class_name, arguments)
            }
            
            Expression::This => {
                // TODO: Implement proper "this" type resolution in class context
                Ok(Type::Class("Object".to_string()))
            }
            
            _ => {
                // Handle other expression types as needed
                Ok(Type::Void)
            }
        }
    }

    fn analyze_literal(&self, literal: &Literal) -> Type {
        match literal {
            Literal::Integer(_) => Type::Int,
            Literal::Float(_) => Type::Float,
            Literal::Boolean(_) => Type::Bool,
            Literal::String(_) => Type::String,
        }
    }

    fn analyze_binary_operation(
        &self,
        left_type: &Type,
        operator: &BinaryOperator,
        right_type: &Type,
    ) -> SemanticResult<Type> {
        match operator {
            BinaryOperator::Add | BinaryOperator::Subtract | 
            BinaryOperator::Multiply | BinaryOperator::Divide => {
                if matches!((left_type, right_type), (Type::Int, Type::Int)) {
                    Ok(Type::Int)
                } else if matches!((left_type, right_type), 
                    (Type::Float, Type::Float) | (Type::Int, Type::Float) | (Type::Float, Type::Int)) {
                    Ok(Type::Float)
                } else if matches!((left_type, right_type), (Type::String, Type::String)) && 
                         matches!(operator, BinaryOperator::Add) {
                    Ok(Type::String)
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", operator),
                        types: vec![format!("{}", left_type), format!("{}", right_type)],
                    })
                }
            }
            
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                if self.types_compatible(left_type, right_type) {
                    Ok(Type::Bool)
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", operator),
                        types: vec![format!("{}", left_type), format!("{}", right_type)],
                    })
                }
            }
            
            BinaryOperator::Less | BinaryOperator::LessEqual | 
            BinaryOperator::Greater | BinaryOperator::GreaterEqual => {
                if matches!((left_type, right_type), 
                    (Type::Int, Type::Int) | (Type::Float, Type::Float) | 
                    (Type::Int, Type::Float) | (Type::Float, Type::Int)) {
                    Ok(Type::Bool)
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", operator),
                        types: vec![format!("{}", left_type), format!("{}", right_type)],
                    })
                }
            }
            
            BinaryOperator::And | BinaryOperator::Or => {
                if matches!((left_type, right_type), (Type::Bool, Type::Bool)) {
                    Ok(Type::Bool)
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: format!("{}", operator),
                        types: vec![format!("{}", left_type), format!("{}", right_type)],
                    })
                }
            }
        }
    }

    fn analyze_unary_operation(
        &self,
        operator: &UnaryOperator,
        operand_type: &Type,
    ) -> SemanticResult<Type> {
        match operator {
            UnaryOperator::Minus => {
                if matches!(operand_type, Type::Int | Type::Float) {
                    Ok(operand_type.clone())
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: "unary minus".to_string(),
                        types: vec![format!("{}", operand_type)],
                    })
                }
            }
            
            UnaryOperator::Not => {
                if matches!(operand_type, Type::Bool) {
                    Ok(Type::Bool)
                } else {
                    Err(SemanticError::InvalidOperation {
                        operation: "logical not".to_string(),
                        types: vec![format!("{}", operand_type)],
                    })
                }
            }
        }
    }

    fn analyze_method_call(
        &mut self,
        object_type: &Type,
        method: &str,
        arguments: &[Expression],
    ) -> SemanticResult<Type> {
        let type_name = match object_type {
            Type::Int => "Int",
            Type::Float => "Float", 
            Type::Bool => "Bool",
            Type::String => "String",
            Type::Class(name) if name == "Char" => "Char",
            Type::Class(name) => name,
            _ => return Err(SemanticError::MethodNotFound {
                class: format!("{}", object_type),
                method: method.to_string(),
            }),
        };

        if let Some(type_info) = self.primitive_types.get(type_name) {
            if let Some(method_sig) = type_info.methods.get(method).cloned() {
                // Validate argument count
                if arguments.len() != method_sig.params.len() {
                    return Err(SemanticError::InvalidArgumentCount {
                        expected: method_sig.params.len(),
                        found: arguments.len(),
                    });
                }
                
                // Validate argument types
                for (i, arg) in arguments.iter().enumerate() {
                    let arg_type = self.analyze_expression(arg)?;
                    if !self.types_compatible(&method_sig.params[i], &arg_type) {
                        return Err(SemanticError::TypeMismatch {
                            expected: format!("{}", method_sig.params[i]),
                            found: format!("{}", arg_type),
                        });
                    }
                }
                
                Ok(method_sig.return_type)
            } else {
                Err(SemanticError::MethodNotFound {
                    class: type_name.to_string(),
                    method: method.to_string(),
                })
            }
        } else {
            Err(SemanticError::ClassNotFound(type_name.to_string()))
        }
    }

    fn analyze_property_access(
        &self,
        object_type: &Type,
        property: &str,
    ) -> SemanticResult<Type> {
        let type_name = match object_type {
            Type::Int => "Int",
            Type::Float => "Float",
            Type::Bool => "Bool", 
            Type::String => "String",
            Type::Class(name) if name == "Char" => "Char",
            Type::Class(name) => name,
            _ => return Err(SemanticError::InvalidPropertyAccess {
                class: format!("{}", object_type),
                property: property.to_string(),
            }),
        };

        if let Some(type_info) = self.primitive_types.get(type_name) {
            if let Some(prop_type) = type_info.properties.get(property) {
                Ok(prop_type.clone())
            } else if let Some(prop_type) = type_info.static_properties.get(property) {
                Ok(prop_type.clone())
            } else {
                Err(SemanticError::InvalidPropertyAccess {
                    class: type_name.to_string(),
                    property: property.to_string(),
                })
            }
        } else {
            Err(SemanticError::ClassNotFound(type_name.to_string()))
        }
    }

    fn analyze_object_creation(
        &mut self,
        class_name: &str,
        arguments: &[Expression],
    ) -> SemanticResult<Type> {
        // For primitive type wrappers, validate constructor arguments
        match class_name {
            "Int" => {
                if arguments.len() == 1 {
                    let arg_type = self.analyze_expression(&arguments[0])?;
                    if matches!(arg_type, Type::Int | Type::Float | Type::String) {
                        Ok(Type::Int)
                    } else {
                        Err(SemanticError::TypeMismatch {
                            expected: "Int, Float, or String".to_string(),
                            found: format!("{}", arg_type),
                        })
                    }
                } else {
                    Err(SemanticError::InvalidArgumentCount {
                        expected: 1,
                        found: arguments.len(),
                    })
                }
            }
            
            "Float" => {
                if arguments.len() == 1 {
                    let arg_type = self.analyze_expression(&arguments[0])?;
                    if matches!(arg_type, Type::Int | Type::Float | Type::String) {
                        Ok(Type::Float)
                    } else {
                        Err(SemanticError::TypeMismatch {
                            expected: "Int, Float, or String".to_string(),
                            found: format!("{}", arg_type),
                        })
                    }
                } else {
                    Err(SemanticError::InvalidArgumentCount {
                        expected: 1,
                        found: arguments.len(),
                    })
                }
            }
            
            "Bool" => {
                if arguments.len() == 1 {
                    let arg_type = self.analyze_expression(&arguments[0])?;
                    if matches!(arg_type, Type::Bool | Type::String) {
                        Ok(Type::Bool)
                    } else {
                        Err(SemanticError::TypeMismatch {
                            expected: "Bool or String".to_string(),
                            found: format!("{}", arg_type),
                        })
                    }
                } else {
                    Err(SemanticError::InvalidArgumentCount {
                        expected: 1,
                        found: arguments.len(),
                    })
                }
            }
            
            "String" => {
                if arguments.len() == 1 {
                    // String can wrap any type
                    self.analyze_expression(&arguments[0])?;
                    Ok(Type::String)
                } else {
                    Err(SemanticError::InvalidArgumentCount {
                        expected: 1,
                        found: arguments.len(),
                    })
                }
            }
            
            "Char" => {
                if arguments.len() == 1 {
                    let arg_type = self.analyze_expression(&arguments[0])?;
                    if matches!(arg_type, Type::String | Type::Int) {
                        Ok(Type::Class("Char".to_string()))
                    } else {
                        Err(SemanticError::TypeMismatch {
                            expected: "String or Int".to_string(),
                            found: format!("{}", arg_type),
                        })
                    }
                } else {
                    Err(SemanticError::InvalidArgumentCount {
                        expected: 1,
                        found: arguments.len(),
                    })
                }
            }
            
            _ => {
                // For other classes, just return the class type
                Ok(Type::Class(class_name.to_string()))
            }
        }
    }

    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        match (expected, actual) {
            // Exact matches
            (a, b) if a == b => true,
            
            // Numeric type conversions
            (Type::Float, Type::Int) => true,
            
            // Class hierarchy (simplified for now)
            (Type::Class(expected_name), Type::Class(actual_name)) => {
                expected_name == actual_name
            }
            
            _ => false,
        }
    }
} 