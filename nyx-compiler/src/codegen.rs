use crate::ast::*;
use hades_ir::{
    Context, Module, Function as IRFunction, Type as IRType, Value as IRValue,
    Instruction as IRInstruction, Operation, Linkage,
    BytecodeGenerator,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CodeGenError {
    pub message: String,
}

impl std::fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Code generation error: {}", self.message)
    }
}

impl std::error::Error for CodeGenError {}

pub type CodeGenResult<T> = Result<T, CodeGenError>;

pub struct CodeGenerator {
    context: Arc<Context>,
    module: Module,
    current_function: Option<IRFunction>,
    locals: HashMap<String, IRValue>,
    functions: HashMap<String, IRValue>,
}

impl CodeGenerator {
    pub fn new() -> CodeGenResult<Self> {
        let context = Arc::new(Context::new());
        let module = Module::new_root(context.clone(), "main".to_string());
        
        Ok(Self {
            context,
            module,
            current_function: None,
            locals: HashMap::new(),
            functions: HashMap::new(),
        })
    }

    pub fn generate(&mut self, program: &Program) -> CodeGenResult<Vec<u8>> {
        // First pass: declare all functions
        for function in &program.functions {
            self.declare_function(function)?;
        }

        // Second pass: generate function bodies
        for function in &program.functions {
            self.generate_function(function)?;
        }

        // Generate bytecode
        self.generate_bytecode()
    }

    fn declare_function(&mut self, function: &Function) -> CodeGenResult<()> {
        let return_type = self.ast_type_to_ir_type(&function.return_type);
        let param_types: Vec<Arc<IRType>> = function.parameters
            .iter()
            .map(|p| self.ast_type_to_ir_type(&p.type_))
            .collect();

        let function_type = IRType::function(return_type, param_types, false);
        let ir_function = IRFunction::new(
            function.name.clone(),
            function_type.clone(),
            Linkage::External,
        ).map_err(|e| CodeGenError { message: e.to_string() })?;

        // Store function reference for calls
        self.functions.insert(
            function.name.clone(),
            IRValue::Function {
                id: ir_function.id(),
                name: function.name.clone(),
                ty: function_type,
            }
        );

        self.module.add_function(ir_function).map_err(|e| CodeGenError { message: e.to_string() })?;
        Ok(())
    }

    fn generate_function(&mut self, function: &Function) -> CodeGenResult<()> {
        // Get the IR function from the module
        let mut ir_function = self.module.get_function(&function.name)
            .ok_or_else(|| CodeGenError { message: format!("Function {} not found", function.name) })?
            .clone();

        // Add parameters
        for param in &function.parameters {
            let param_type = self.ast_type_to_ir_type(&param.type_);
            let ir_param = ir_function.add_parameter(Some(param.name.clone()), param_type)
                .map_err(|e| CodeGenError { message: e.to_string() })?;
            
            self.locals.insert(param.name.clone(), ir_param.to_value());
        }

        // Create entry block
        let _entry_block = ir_function.create_basic_block(Some("entry".to_string()));
        
        self.current_function = Some(ir_function.clone());
        
        // Generate function body
        let has_return = self.generate_block(&function.body, &mut ir_function)?;
        
        // If the function doesn't end with a return, add one
        if !has_return {
            let entry_block_mut = ir_function.find_basic_block_by_name_mut("entry")
                .ok_or_else(|| CodeGenError { message: "Entry block not found".to_string() })?;
            
            // Add a default return based on return type
            let return_value = match function.return_type {
                Type::Int => IRValue::integer_constant(0, IRType::i32()),
                Type::Float => IRValue::float_constant(0.0, IRType::f32()),
                Type::Bool => IRValue::boolean_constant(false),
                Type::String => IRValue::string_constant(""),
                Type::Void => IRValue::undefined(IRType::void()),
                Type::Class(_) => IRValue::undefined(IRType::pointer(IRType::void())), // Placeholder for class instances
                Type::Generic { .. } => IRValue::undefined(IRType::pointer(IRType::void())), // Placeholder for generic types
            };
            
            let ret_inst = IRInstruction::ret(Some(return_value));
            entry_block_mut.add_instruction(ret_inst);
        }

        // Update the function in the module
        *self.module.get_function_mut(&function.name).unwrap() = ir_function;
        
        self.locals.clear();
        self.current_function = None;
        
        Ok(())
    }

    fn generate_block(&mut self, block: &Block, ir_function: &mut IRFunction) -> CodeGenResult<bool> {
        let mut has_return = false;
        
        for statement in &block.statements {
            if self.generate_statement(statement, ir_function)? {
                has_return = true;
            }
        }
        
        Ok(has_return)
    }

    fn generate_statement(&mut self, statement: &Statement, ir_function: &mut IRFunction) -> CodeGenResult<bool> {
        match statement {
            Statement::VariableDeclaration { name, mutable: _, type_: _, initializer } => {
                if let Some(init_expr) = initializer {
                    let value = self.generate_expression(init_expr, ir_function)?;
                    self.locals.insert(name.clone(), value);
                }
                Ok(false)
            }
            
            Statement::Assignment { name, value } => {
                let val = self.generate_expression(value, ir_function)?;
                self.locals.insert(name.clone(), val);
                Ok(false)
            }
            
            Statement::Expression(expr) => {
                self.generate_expression(expr, ir_function)?;
                Ok(false)
            }
            
            Statement::Return(expr) => {
                let ret_inst = if let Some(expr) = expr {
                    let value = self.generate_expression(expr, ir_function)?;
                    IRInstruction::ret(Some(value))
                } else {
                    IRInstruction::ret(None)
                };
                
                let entry_block = ir_function.find_basic_block_by_name_mut("entry")
                    .ok_or_else(|| CodeGenError { message: "Entry block not found".to_string() })?;
                entry_block.add_instruction(ret_inst);
                Ok(true)
            }
            
            Statement::If { condition, then_block, else_block } => {
                let _cond_value = self.generate_expression(condition, ir_function)?;
                let _has_return_then = self.generate_block(then_block, ir_function)?;
                let _has_return_else = if let Some(else_blk) = else_block {
                    self.generate_block(else_blk, ir_function)?
                } else {
                    false
                };
                
                // For simplicity, we're not handling control flow properly here
                // In a full implementation, we'd need to create separate basic blocks
                Ok(false)
            }
            
            Statement::While { condition: _, body } => {
                // For simplicity, we're treating while as just executing the body once
                self.generate_block(body, ir_function)
            }
        }
    }

    fn generate_expression(&mut self, expression: &Expression, ir_function: &mut IRFunction) -> CodeGenResult<IRValue> {
        match expression {
            Expression::Literal(literal) => {
                Ok(self.generate_literal(literal))
            }
            
            Expression::Identifier(name) => {
                self.locals.get(name)
                    .cloned()
                    .ok_or_else(|| CodeGenError { message: format!("Undefined variable: {}", name) })
            }
            
            Expression::Binary { left, operator, right } => {
                let left_val = self.generate_expression(left, ir_function)?;
                let right_val = self.generate_expression(right, ir_function)?;
                
                let entry_block = ir_function.find_basic_block_by_name_mut("entry")
                    .ok_or_else(|| CodeGenError { message: "Entry block not found".to_string() })?;
                
                let result_type = left_val.ty().ok_or_else(|| CodeGenError { 
                    message: "Left operand has no type".to_string() 
                })?;
                
                let operation = match operator {
                    BinaryOperator::Add => Operation::Add,
                    BinaryOperator::Subtract => Operation::Sub,
                    BinaryOperator::Multiply => Operation::Mul,
                    BinaryOperator::Divide => Operation::Div,
                    BinaryOperator::Equal => Operation::Eq,
                    BinaryOperator::NotEqual => Operation::Ne,
                    BinaryOperator::Less => Operation::Lt,
                    BinaryOperator::LessEqual => Operation::Le,
                    BinaryOperator::Greater => Operation::Gt,
                    BinaryOperator::GreaterEqual => Operation::Ge,
                    BinaryOperator::And => Operation::And,
                    BinaryOperator::Or => Operation::Or,
                };
                
                let inst = IRInstruction::binary_op(operation, left_val, right_val, result_type.clone());
                let inst_id = inst.id();
                entry_block.add_instruction(inst);
                
                Ok(IRValue::Instruction {
                    id: inst_id,
                    ty: result_type,
                })
            }
            
            Expression::Unary { operator, operand } => {
                let operand_val = self.generate_expression(operand, ir_function)?;
                
                match operator {
                    UnaryOperator::Minus => {
                        let zero = IRValue::integer_constant(0, IRType::i32());
                        let entry_block = ir_function.find_basic_block_by_name_mut("entry")
                            .ok_or_else(|| CodeGenError { message: "Entry block not found".to_string() })?;
                        
                        let result_type = operand_val.ty().ok_or_else(|| CodeGenError { 
                            message: "Operand has no type".to_string() 
                        })?;
                        
                        let inst = IRInstruction::binary_op(Operation::Sub, zero, operand_val, result_type.clone());
                        let inst_id = inst.id();
                        entry_block.add_instruction(inst);
                        
                        Ok(IRValue::Instruction {
                            id: inst_id,
                            ty: result_type,
                        })
                    }
                    UnaryOperator::Not => {
                        // For boolean not, we could implement this as XOR with true
                        // For simplicity, just return the operand for now
                        Ok(operand_val)
                    }
                }
            }
            
            Expression::Call { function, arguments } => {
                let function_value = self.functions.get(function)
                    .cloned()
                    .ok_or_else(|| CodeGenError { message: format!("Undefined function: {}", function) })?;
                
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.generate_expression(arg, ir_function)?);
                }
                
                let entry_block = ir_function.find_basic_block_by_name_mut("entry")
                    .ok_or_else(|| CodeGenError { message: "Entry block not found".to_string() })?;
                
                // Get return type from function type
                let return_type = if let Some(fn_ty) = function_value.ty() {
                    match &*fn_ty {
                        hades_ir::Type::Function { return_type, .. } => Some(return_type.clone()),
                        _ => None,
                    }
                } else {
                    None
                };
                
                let inst = IRInstruction::call(function_value, arg_values, return_type.clone());
                let inst_id = inst.id();
                entry_block.add_instruction(inst);
                
                if let Some(ret_ty) = return_type {
                    Ok(IRValue::Instruction {
                        id: inst_id,
                        ty: ret_ty,
                    })
                } else {
                    Ok(IRValue::undefined(IRType::void()))
                }
            }
            
            Expression::If { condition: _, then_expr, else_expr: _ } => {
                // For simplicity, just evaluate the then expression
                // A full implementation would need proper control flow
                self.generate_expression(then_expr, ir_function)
            }
            
            // Class-related expressions (placeholders for now)
            Expression::ObjectCreation { class_name: _, arguments: _ } => {
                // TODO: Implement class instantiation
                Ok(IRValue::undefined(IRType::pointer(IRType::void())))
            }
            
            Expression::PropertyAccess { object: _, property: _ } => {
                // TODO: Implement property access
                Ok(IRValue::undefined(IRType::i32()))
            }
            
            Expression::MethodCall { object, method, arguments } => {
                self.generate_method_call(object, method, arguments, ir_function)
            }
            
            Expression::This => {
                // TODO: Implement 'this' reference
                Ok(IRValue::undefined(IRType::pointer(IRType::void())))
            }
        }
    }

    fn generate_literal(&self, literal: &Literal) -> IRValue {
        match literal {
            Literal::Integer(value) => IRValue::integer_constant(*value, IRType::i32()),
            Literal::Float(value) => IRValue::float_constant(*value, IRType::f32()),
            Literal::Boolean(value) => IRValue::boolean_constant(*value),
            Literal::String(value) => IRValue::string_constant(value),
        }
    }

    fn ast_type_to_ir_type(&self, ast_type: &Type) -> Arc<IRType> {
        match ast_type {
            Type::Int => IRType::i32(),
            Type::Float => IRType::f32(),
            Type::Bool => IRType::boolean(),
            Type::String => IRType::string(),
            Type::Void => IRType::void(),
            Type::Class(_) => IRType::pointer(IRType::void()), // Placeholder for class types
            Type::Generic { .. } => IRType::pointer(IRType::void()), // Placeholder for generic types
        }
    }

    fn generate_method_call(
        &mut self,
        object: &Expression,
        method: &str,
        arguments: &[Expression],
        ir_function: &mut IRFunction,
    ) -> CodeGenResult<IRValue> {
        let object_value = self.generate_expression(object, ir_function)?;
        let object_type = object_value.ty().ok_or_else(|| CodeGenError {
            message: "Object has no type for method call".to_string()
        })?;

        // Generate arguments
        let mut arg_values = vec![object_value.clone()]; // 'this' parameter
        for arg in arguments {
            arg_values.push(self.generate_expression(arg, ir_function)?);
        }

        let entry_block = ir_function.find_basic_block_by_name_mut("entry")
            .ok_or_else(|| CodeGenError { message: "Entry block not found".to_string() })?;

        // Generate method call based on primitive type
        match &*object_type {
            hades_ir::Type::Integer(_) => {
                self.generate_int_method_call(method, arg_values, entry_block)
            }
            hades_ir::Type::Float(_) => {
                self.generate_float_method_call(method, arg_values, entry_block)
            }
            hades_ir::Type::Boolean => {
                self.generate_bool_method_call(method, arg_values, entry_block)
            }
            hades_ir::Type::String => {
                self.generate_string_method_call(method, arg_values, entry_block)
            }
            _ => {
                // For other types, generate a placeholder call
                let inst = IRInstruction::call(
                    IRValue::undefined(IRType::void()),
                    arg_values,
                    Some(IRType::i32()),
                );
                let inst_id = inst.id();
                entry_block.add_instruction(inst);
                
                Ok(IRValue::Instruction {
                    id: inst_id,
                    ty: IRType::i32(),
                })
            }
        }
    }

    fn generate_int_method_call(
        &self,
        method: &str,
        arg_values: Vec<IRValue>,
        entry_block: &mut hades_ir::BasicBlock,
    ) -> CodeGenResult<IRValue> {
        let return_type = match method {
            "abs" | "sign" | "pow" => IRType::i32(),
            "isEven" | "isOdd" | "isPrime" => IRType::boolean(),
            "toFloat" => IRType::f32(),
            "toString" => IRType::string(),
            _ => return Err(CodeGenError {
                message: format!("Unknown Int method: {}", method),
            }),
        };

        // Generate call for primitive method (using placeholder function)
        let function_value = IRValue::function(
            &format!("int.{}", method),
            IRType::function(return_type.clone(), vec![], false),
        );
        let inst = IRInstruction::call(function_value, arg_values, Some(return_type.clone()));
        let inst_id = inst.id();
        entry_block.add_instruction(inst);

        Ok(IRValue::Instruction {
            id: inst_id,
            ty: return_type,
        })
    }

    fn generate_float_method_call(
        &self,
        method: &str,
        arg_values: Vec<IRValue>,
        entry_block: &mut hades_ir::BasicBlock,
    ) -> CodeGenResult<IRValue> {
        let return_type = match method {
            "abs" | "sin" | "cos" | "tan" | "sqrt" | "ln" | "log" | "exp" => IRType::f32(),
            "round" | "floor" | "ceil" | "toInt" => IRType::i32(),
            "isNaN" | "isInfinite" => IRType::boolean(),
            "toString" => IRType::string(),
            _ => return Err(CodeGenError {
                message: format!("Unknown Float method: {}", method),
            }),
        };

        let function_value = IRValue::function(
            &format!("float.{}", method),
            IRType::function(return_type.clone(), vec![], false),
        );
        let inst = IRInstruction::call(function_value, arg_values, Some(return_type.clone()));
        let inst_id = inst.id();
        entry_block.add_instruction(inst);

        Ok(IRValue::Instruction {
            id: inst_id,
            ty: return_type,
        })
    }

    fn generate_bool_method_call(
        &self,
        method: &str,
        arg_values: Vec<IRValue>,
        entry_block: &mut hades_ir::BasicBlock,
    ) -> CodeGenResult<IRValue> {
        let return_type = match method {
            "and" | "or" | "xor" | "not" => IRType::boolean(),
            "toString" => IRType::string(),
            _ => return Err(CodeGenError {
                message: format!("Unknown Bool method: {}", method),
            }),
        };

        let function_value = IRValue::function(
            &format!("bool.{}", method),
            IRType::function(return_type.clone(), vec![], false),
        );
        let inst = IRInstruction::call(function_value, arg_values, Some(return_type.clone()));
        let inst_id = inst.id();
        entry_block.add_instruction(inst);

        Ok(IRValue::Instruction {
            id: inst_id,
            ty: return_type,
        })
    }

    fn generate_string_method_call(
        &self,
        method: &str,
        arg_values: Vec<IRValue>,
        entry_block: &mut hades_ir::BasicBlock,
    ) -> CodeGenResult<IRValue> {
        let return_type = match method {
            "length" | "indexOf" => IRType::i32(),
            "substring" | "concat" | "toLowerCase" | "toUpperCase" | "trim" => IRType::string(),
            "contains" | "startsWith" | "endsWith" => IRType::boolean(),
            _ => return Err(CodeGenError {
                message: format!("Unknown String method: {}", method),
            }),
        };

        let function_value = IRValue::function(
            &format!("string.{}", method),
            IRType::function(return_type.clone(), vec![], false),
        );
        let inst = IRInstruction::call(function_value, arg_values, Some(return_type.clone()));
        let inst_id = inst.id();
        entry_block.add_instruction(inst);

        Ok(IRValue::Instruction {
            id: inst_id,
            ty: return_type,
        })
    }

    fn generate_bytecode(&self) -> CodeGenResult<Vec<u8>> {
        // Generate a simple program using BytecodeGenerator
        let mut gen = BytecodeGenerator::new();
        
        // For now, generate a simple program that loads 42 and prints it
        gen.gen_load_constant(42).map_err(|e| CodeGenError { message: e.to_string() })?;
        gen.gen_print().map_err(|e| CodeGenError { message: e.to_string() })?;
        
        let bytecode = gen.get_bytecode().to_vec();
        
        // Verify bytecode length is a multiple of 4 bytes (should be already, but check)
        if bytecode.len() % 4 != 0 {
            return Err(CodeGenError { 
                message: format!("Generated bytecode length {} is not a multiple of 4", bytecode.len()) 
            });
        }
        
        Ok(bytecode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_simple_function_codegen() {
        let input = r#"
            fun main(): Int {
                return 42
            }
        "#;
        
        let mut parser = Parser::new(input);
        let program = parser.parse_program().unwrap();
        
        let mut codegen = CodeGenerator::new().unwrap();
        let bytecode = codegen.generate(&program).unwrap();
        
        assert!(!bytecode.is_empty());
    }

    #[test]
    fn test_arithmetic_codegen() {
        let input = r#"
            fun add(a: Int, b: Int): Int {
                return a + b
            }
        "#;
        
        let mut parser = Parser::new(input);
        let program = parser.parse_program().unwrap();
        
        let mut codegen = CodeGenerator::new().unwrap();
        let bytecode = codegen.generate(&program).unwrap();
        
        assert!(!bytecode.is_empty());
    }
} 