use std::sync::Arc;
use hades_ir::{
    instruction::Instruction,
    value::Value,
    types::Type,
};
use hades_ir_interpreter::IRInterpreter;

fn main() {
    let mut interpreter = IRInterpreter::new();

    // Create string constants
    let hello = Value::string_constant("Hello ");
    let world = Value::string_constant("World");

    // Concatenate strings
    let concat = Instruction::string_concat(hello.clone(), world.clone());
    interpreter.execute_instruction(&concat).unwrap();

    // Print the result
    let print = Instruction::print(concat.to_value());
    interpreter.execute_instruction(&print).unwrap();

    // Get string length
    let length = Instruction::string_length(hello);
    interpreter.execute_instruction(&length).unwrap();

    // Print the length
    let print = Instruction::print(length.to_value());
    interpreter.execute_instruction(&print).unwrap();

    // Get substring
    let str = Value::string_constant("Hello World");
    let start = Value::integer_constant(0, Arc::new(Type::Integer(32)));
    let len = Value::integer_constant(5, Arc::new(Type::Integer(32)));
    let substring = Instruction::string_substring(str, start, len);
    interpreter.execute_instruction(&substring).unwrap();

    // Print the substring
    let print = Instruction::print(substring.to_value());
    interpreter.execute_instruction(&print).unwrap();
} 