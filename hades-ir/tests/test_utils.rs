use std::sync::Arc;
use hades_ir::{
    Context,
    Type,
    Value,
    BasicBlock,
    Builder,
    Function,
    Module,
    Linkage,
    Operation,
};

/// Creates a new context for testing
pub fn create_test_context() -> Arc<Context> {
    Arc::new(Context::new())
}

/// Creates a simple function type (i32, i32) -> i32
pub fn create_test_function_type(context: Arc<Context>) -> Arc<Type> {
    let i32_type = Type::i32();
    let param_types = vec![i32_type.clone(), i32_type.clone()];
    Type::function(i32_type, param_types, false)
}

/// Creates a test module with a given name
pub fn create_test_module(context: Arc<Context>, name: &str) -> Module {
    let mut module = Module::new_root(context, name.to_string());
    module.set_source_file("test.rs".to_string());
    module
}

/// Creates a simple test function that adds two integers
pub fn create_test_function(name: &str, ty: Arc<Type>) -> Result<Function, hades_ir::Error> {
    Function::new(name.to_string(), ty, Linkage::External)
}

/// Creates a builder with a new context
pub fn create_test_builder() -> Builder {
    Builder::new(create_test_context())
}

/// Creates a basic block with a given name
pub fn create_test_block(name: &str) -> BasicBlock {
    BasicBlock::named(name)
} 