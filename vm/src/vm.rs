use std::{collections::VecDeque, io::{stdout, Stdout}};

use crate::{instruction::{Instruction, RawInstruction}, opcode::Opcode};

pub struct CPU {
    fetcher: InstructionFetcher, 
    decoder: InstructionDecoder, 
    executor: InstructionExecutor,
    stack: VecDeque<i32>,
    // memory: Memory, 
}

impl CPU {
    pub fn new() -> Self {
        Self {
            fetcher: InstructionFetcher::new(),
            decoder: InstructionDecoder,
            executor: InstructionExecutor::new(),
            stack: VecDeque::new(),
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.fetcher.load_program(program);
    }

    pub fn step(&mut self) {
        if let Some(instruction_raw) = self.fetcher.fetch() {
            let instruction = self.decoder.decode(instruction_raw);
            self.executor.execute(&instruction, &mut self.stack);
        }
    }
}

pub struct ALU; 

impl ALU {
    pub fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

pub struct InstructionFetcher {
    pub pc: usize, 
    pub program: Vec<u8>, 
}

impl InstructionFetcher {
    pub fn new() -> Self {
        Self { program: vec![], pc: 0 }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.program = program.to_vec();
        self.pc = 0;
    }

    pub fn peek_next(&self) -> Option<RawInstruction> {
        if self.pc + 3 < self.program.len() {
            let b0 = self.program[self.pc];
            let b1 = self.program[self.pc + 1];
            let b2 = self.program[self.pc + 2];
            let b3 = self.program[self.pc + 3];
            
            Some(RawInstruction::from_bytes(b0, b1, b2, b3))
        } else {
            None
        }
    }

    pub fn fetch(&mut self) -> Option<RawInstruction> {
        // Ensure we have at least 4 bytes to read
        if self.pc + 3 < self.program.len() {
            // Read 4 bytes and combine them into a 32-bit instruction
            let b0 = self.program[self.pc];
            let b1 = self.program[self.pc + 1];
            let b2 = self.program[self.pc + 2];
            let b3 = self.program[self.pc + 3];
            
            // Combine bytes into a 32-bit instruction (big endian)
            let instruction = RawInstruction::from_bytes(b0, b1, b2, b3);
            
            // Increment program counter by 4 bytes
            self.pc += 4;
            
            Some(instruction)
        } else {
            None
        }
    }
}


pub struct InstructionDecoder; 

impl InstructionDecoder {
    pub fn decode(&self, instruction: RawInstruction) -> Instruction {
        // the last 8 bits are the opcode
        let opcode = instruction.opcode(); 
        let operands = instruction.get_operands(); 

        // Extract operands based on the opcode's operand count
        Instruction { opcode, operands }
    }
}


pub struct InstructionExecutor { 
    alu: ALU, 
}

impl InstructionExecutor {

    pub fn new() -> Self {
        Self { alu: ALU }
    }

    pub fn execute(&mut self, instruction: &Instruction,  stack: &mut VecDeque<i32>,) {
        match instruction.opcode {
            Opcode::Add => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Add instruction requires 0 operands");
                }

                let a = stack.pop_back();
                let b = stack.pop_back();
                
                if let (Some(a), Some(b)) = (a, b) {
                    let result = self.alu.add(a, b);
                    stack.push_back(result);
                } else {
                    panic!("Stack underflow");
                }
            }, 
            Opcode::Store => {
                if instruction.opcode.operand_count() != 1 {
                    panic!("Store instruction requires 1 operand");
                }

                let value = instruction.operands[0];

                // push the value to the stack
                stack.push_back(value);
            }
        }
    }
}



pub struct VirtualMachine {
    pub stack: VecDeque<i32>, 
    pub cpu: CPU, 
    pub output: Stdout, 
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self { stack: VecDeque::new(), cpu: CPU::new(), output: stdout() }
    }

    pub fn run(&mut self) {
        self.cpu.step();
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.cpu.load_program(program);
    }

    pub fn run_until_completion(&mut self) {
        while self.cpu.fetcher.peek_next().is_some() {
            self.run();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; 

    #[test]
    fn test_execute() {
        // Create a program with:
        // 1. Store 5 onto the stack
        // 2. Store 7 onto the stack
        // 3. Add instruction to add the two values
        let program = [
            // First store instruction (opcode 0x02) with operand 5
            0x00, 0x00, 0x05, 0x02,
            // Second store instruction with operand 7
            0x00, 0x00, 0x07, 0x02,
            // Add instruction (opcode 0x01) with no operands
            0x00, 0x00, 0x00, 0x01
        ];
        
        let mut vm = VirtualMachine::new();


        vm.load_program(&program);

        vm.run_until_completion();
        
        // Check if result is 12 (5 + 7)
        assert_eq!(vm.cpu.stack.pop_back(), Some(12));
    }
}