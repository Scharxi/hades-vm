use crate::ast::*;
use hades_ir::{
    Context, Module, Function as IRFunction, Type as IRType, Value as IRValue,
    Instruction as IRInstruction, Operation, Linkage, ExecutableGenerator,
    HadesExecutable, SectionType,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct HexCompilerError {
    pub message: String,
}

impl std::fmt::Display for HexCompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HEX Compiler error: {}", self.message)
    }
}

impl std::error::Error for HexCompilerError {}

pub type HexResult<T> = Result<T, HexCompilerError>;

/// Compiler that generates Hades Executable (HEX) files
pub struct HexCompiler {
    context: Arc<Context>,
    debug: bool,
}

impl HexCompiler {
    pub fn new(debug: bool) -> HexResult<Self> {
        let context = Arc::new(Context::new());
        
        Ok(Self {
            context,
            debug,
        })
    }

    /// Compile a program to a HEX executable
    pub fn compile_to_hex(&mut self, program: &Program) -> HexResult<HadesExecutable> {
        if self.debug {
            println!("Starting HEX compilation...");
        }

        // Create IR module
        let mut module = Module::new(self.context.clone(), "main".to_string());

        // First pass: declare all functions
        for function in &program.functions {
            self.declare_function(&mut module, function)?;
        }

        // Second pass: generate function bodies
        for function in &program.functions {
            self.generate_function_body(&mut module, function)?;
        }

        if self.debug {
            println!("Generated IR module with {} functions", module.functions().len());
        }

        // Generate executable
        let mut generator = ExecutableGenerator::new();
        generator.set_entry_function("main".to_string());
        generator.generate_from_module(&module)
            .map_err(|e| HexCompilerError { message: e.to_string() })?;

        let executable = generator.into_executable();

        if self.debug {
            println!("Generated HEX executable with {} sections", executable.sections.len());
        }

        Ok(executable)
    }

    /// Compile and write executable to file
    pub fn compile_to_file<P: AsRef<Path>>(&mut self, program: &Program, output_path: P) -> HexResult<()> {
        let mut executable = self.compile_to_hex(program)?;
        
        let file = File::create(output_path)
            .map_err(|e| HexCompilerError { message: format!("Failed to create output file: {}", e) })?;
        
        let mut writer = BufWriter::new(file);
        executable.write_to(&mut writer)
            .map_err(|e| HexCompilerError { message: format!("Failed to write executable: {}", e) })?;
        
        writer.flush()
            .map_err(|e| HexCompilerError { message: format!("Failed to flush output: {}", e) })?;
        
        if self.debug {
            println!("Successfully wrote HEX executable to file");
        }
        
        Ok(())
    }

    fn declare_function(&self, module: &mut Module, function: &Function) -> HexResult<()> {
        let return_type = self.ast_type_to_ir_type(&function.return_type);
        let param_types: Vec<Arc<IRType>> = function.parameters
            .iter()
            .map(|p| self.ast_type_to_ir_type(&p.type_))
            .collect();

        let function_type = IRType::function(return_type, param_types, false);
        let ir_function = IRFunction::new(
            function.name.clone(),
            function_type,
            Linkage::External,
        ).map_err(|e| HexCompilerError { message: e.to_string() })?;

        module.add_function(ir_function)
            .map_err(|e| HexCompilerError { message: e.to_string() })?;
        
        Ok(())
    }

    fn generate_function_body(&self, module: &mut Module, function: &Function) -> HexResult<()> {
        // Get the IR function from the module
        let ir_function = module.get_function_mut(&function.name)
            .ok_or_else(|| HexCompilerError { 
                message: format!("Function {} not found in module", function.name) 
            })?;

        // Add parameters
        for param in &function.parameters {
            let param_type = self.ast_type_to_ir_type(&param.type_);
            ir_function.add_parameter(Some(param.name.clone()), param_type)
                .map_err(|e| HexCompilerError { message: e.to_string() })?;
        }

        // Create entry block
        let entry_block = ir_function.create_basic_block(Some("entry".to_string()));
        
        // Generate function body
        self.generate_block(&function.body, &mut *ir_function)?;
        
        Ok(())
    }

    fn generate_block(&self, block: &Block, ir_function: &mut IRFunction) -> HexResult<()> {
        for statement in &block.statements {
            self.generate_statement(statement, ir_function)?;
        }
        
        Ok(())
    }

    fn generate_statement(&self, statement: &Statement, ir_function: &mut IRFunction) -> HexResult<()> {
        match statement {
            Statement::VariableDeclaration { name, mutable: _, type_: _, initializer } => {
                if let Some(init_expr) = initializer {
                    let _value = self.generate_expression(init_expr, ir_function)?;
                    // Store the variable - in a complete implementation we'd track locals
                }
                Ok(())
            }
            
            Statement::Assignment { name, value } => {
                let _val = self.generate_expression(value, ir_function)?;
                // Store to variable - in a complete implementation we'd track locals
                Ok(())
            }
            
            Statement::Expression(expr) => {
                self.generate_expression(expr, ir_function)?;
                Ok(())
            }
            
            Statement::Return(expr) => {
                // Generate the return value first (if any) to avoid borrow checker issues
                let ret_inst = if let Some(expr) = expr {
                    let value = self.generate_expression(expr, ir_function)?;
                    IRInstruction::ret(Some(value))
                } else {
                    IRInstruction::ret(None)
                };
                
                // Now get the entry block and add the instruction
                let entry_block = ir_function.find_basic_block_by_name_mut("entry")
                    .ok_or_else(|| HexCompilerError { message: "Entry block not found".to_string() })?;
                
                entry_block.add_instruction(ret_inst);
                Ok(())
            }
            
            Statement::If { condition, then_block, else_block } => {
                // Generate condition
                let _cond_value = self.generate_expression(condition, ir_function)?;
                
                // Generate blocks (simplified - in practice we'd create separate basic blocks)
                self.generate_block(then_block, ir_function)?;
                if let Some(else_blk) = else_block {
                    self.generate_block(else_blk, ir_function)?;
                }
                
                Ok(())
            }
            
            Statement::While { condition, body } => {
                // Generate condition and body (simplified)
                let _cond_value = self.generate_expression(condition, ir_function)?;
                self.generate_block(body, ir_function)?;
                Ok(())
            }
        }
    }

    fn generate_expression(&self, expression: &Expression, ir_function: &mut IRFunction) -> HexResult<IRValue> {
        match expression {
            Expression::Literal(literal) => {
                Ok(self.generate_literal(literal))
            }
            
            Expression::Identifier(name) => {
                // For simplicity, create a dummy value
                // In practice, we'd look up the variable
                Ok(IRValue::integer_constant(0, IRType::i32()))
            }
            
            Expression::Binary { left, operator, right } => {
                let left_val = self.generate_expression(left, ir_function)?;
                let right_val = self.generate_expression(right, ir_function)?;
                
                let operation = match operator {
                    BinaryOperator::Add => Operation::Add,
                    BinaryOperator::Subtract => Operation::Sub,
                    BinaryOperator::Multiply => Operation::Mul,
                    BinaryOperator::Divide => Operation::Div,
                    _ => Operation::Add, // Simplified
                };
                
                // Create instruction
                let inst = IRInstruction::new(operation, vec![left_val, right_val], Some(IRType::i32()));
                
                // Add to entry block
                let entry_block = ir_function.find_basic_block_by_name_mut("entry")
                    .ok_or_else(|| HexCompilerError { message: "Entry block not found".to_string() })?;
                entry_block.add_instruction(inst.clone());
                
                // Return the result value (simplified)
                Ok(IRValue::integer_constant(0, IRType::i32()))
            }
            
            Expression::Call { function, arguments } => {
                // Generate arguments
                let mut arg_values = Vec::new();
                for arg in arguments {
                    let arg_val = self.generate_expression(arg, ir_function)?;
                    arg_values.push(arg_val);
                }
                
                // Create call instruction
                let call_inst = IRInstruction::new(Operation::Call, arg_values, Some(IRType::i32()));
                
                // Add to entry block
                let entry_block = ir_function.find_basic_block_by_name_mut("entry")
                    .ok_or_else(|| HexCompilerError { message: "Entry block not found".to_string() })?;
                entry_block.add_instruction(call_inst);
                
                // Return dummy result
                Ok(IRValue::integer_constant(0, IRType::i32()))
            }
            
            Expression::Unary { operator, operand } => {
                let operand_val = self.generate_expression(operand, ir_function)?;
                
                let operation = match operator {
                    UnaryOperator::Minus => Operation::Neg,
                    UnaryOperator::Not => Operation::Not,
                };
                
                // Create instruction
                let inst = IRInstruction::new(operation, vec![operand_val], Some(IRType::i32()));
                
                // Add to entry block
                let entry_block = ir_function.find_basic_block_by_name_mut("entry")
                    .ok_or_else(|| HexCompilerError { message: "Entry block not found".to_string() })?;
                entry_block.add_instruction(inst);
                
                // Return dummy result
                Ok(IRValue::integer_constant(0, IRType::i32()))
            }
            
            Expression::If { condition, then_expr, else_expr } => {
                // Generate condition
                let _cond_value = self.generate_expression(condition, ir_function)?;
                
                // Generate then expression
                let then_val = self.generate_expression(then_expr, ir_function)?;
                
                // Generate else expression
                let _else_val = self.generate_expression(else_expr, ir_function)?;
                
                // For simplicity, just return the then value
                Ok(then_val)
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
        }
    }
}

/// Convenience function to compile source code directly to HEX file
pub fn compile_source_to_hex_file<P: AsRef<Path>>(
    source: &str, 
    output_path: P,
    debug: bool
) -> HexResult<()> {
    use crate::parser::Parser;
    
    // Parse source code
    let mut parser = Parser::new(source);
    let program = parser.parse_program()
        .map_err(|e| HexCompilerError { message: format!("Parse error: {:?}", e) })?;

    // Compile to HEX
    let mut compiler = HexCompiler::new(debug)?;
    compiler.compile_to_file(&program, output_path)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_hex_compilation() {
        let source = r#"
            fun main(): Int {
                return 42
            }
        "#;
        
        let mut parser = crate::parser::Parser::new(source);
        let program = parser.parse_program().unwrap();
        
        let mut compiler = HexCompiler::new(true).unwrap();
        let executable = compiler.compile_to_hex(&program).unwrap();
        
        // Verify basic properties
        assert_eq!(executable.header.magic, hades_ir::HEX_MAGIC);
        assert_eq!(executable.header.version, hades_ir::HEX_VERSION);
        assert_eq!(executable.header.arch, hades_ir::HEX_ARCH);
        assert!(executable.sections.len() > 0);
    }

    #[test]
    fn test_hex_serialization() {
        let source = r#"
            fun main(): Int {
                return 42
            }
        "#;
        
        let mut parser = crate::parser::Parser::new(source);
        let program = parser.parse_program().unwrap();
        
        let mut compiler = HexCompiler::new(false).unwrap();
        let mut executable = compiler.compile_to_hex(&program).unwrap();
        
        // Write to buffer
        let mut buffer = Vec::new();
        executable.write_to(&mut buffer).unwrap();
        
        // Should be able to read it back
        let mut cursor = Cursor::new(&buffer);
        let decoded = hades_ir::HadesExecutable::read_from(&mut cursor).unwrap();
        
        assert_eq!(executable.header.magic, decoded.header.magic);
        assert_eq!(executable.header.entry_point, decoded.header.entry_point);
        assert_eq!(executable.sections.len(), decoded.sections.len());
    }
} 