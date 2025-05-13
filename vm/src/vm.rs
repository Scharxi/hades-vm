use std::{collections::VecDeque, io::{stdout, Stdout, Write}};

use crate::{instruction::{Instruction, RawInstruction, IntoRaw}, memory::{MemoryRegionType, SegmentedMemory, AccessPermission}, opcode::Opcode};

pub struct CPU {
    fetcher: InstructionFetcher, 
    decoder: InstructionDecoder, 
    executor: InstructionExecutor,
    stack: VecDeque<i32>,
    memory: SegmentedMemory, 
}

impl CPU {
    pub fn new(memory_size: usize) -> Self {
        #[cfg(test)]
        let memory = match SegmentedMemory::create_test_layout(memory_size) {
            Ok(mem) => mem,
            Err(e) => {
                // Fallback: Erstelle einen Speicher ohne Regionen für Tests
                let mut mem = SegmentedMemory::new(memory_size);
                
                // Erstelle eine einzige Region für den gesamten Speicher mit allen Berechtigungen
                let all_region = crate::memory::MemoryRegion::new(
                    MemoryRegionType::Data,
                    0,
                    memory_size,
                    vec![
                        crate::memory::AccessPermission::Read,
                        crate::memory::AccessPermission::Write,
                        crate::memory::AccessPermission::Execute,
                    ],
                    Some("All Memory".to_string()),
                );
                
                if let Err(e2) = mem.define_region(all_region) {
                    panic!("Failed to create memory: {} and fallback failed: {}", e, e2);
                }
                
                mem
            }
        };
        
        #[cfg(not(test))]
        let memory = match SegmentedMemory::create_standard_layout(memory_size) {
            Ok(mem) => mem,
            Err(e) => {
                // Fallback: Erstelle einen Speicher ohne Regionen für Tests
                let mut mem = SegmentedMemory::new(memory_size);
                
                // Erstelle eine einzige Region für den gesamten Speicher mit allen Berechtigungen
                let all_region = crate::memory::MemoryRegion::new(
                    MemoryRegionType::Data,
                    0,
                    memory_size,
                    vec![
                        crate::memory::AccessPermission::Read,
                        crate::memory::AccessPermission::Write,
                        crate::memory::AccessPermission::Execute,
                    ],
                    Some("All Memory".to_string()),
                );
                
                if let Err(e2) = mem.define_region(all_region) {
                    panic!("Failed to create memory: {} and fallback failed: {}", e, e2);
                }
                
                mem
            }
        };
        
        Self {
            fetcher: InstructionFetcher::new(),
            decoder: InstructionDecoder,
            executor: InstructionExecutor::new(),
            stack: VecDeque::new(),
            memory,
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        // Programm in den Speicher in die Code-Region laden
        self.load_program_to_memory(program);
        
        // Anschließend den InstructionFetcher mit dem Programm initialisieren
        self.fetcher.load_program(program);
    }
    
    /// Lädt das Programm in die Code-Region des Speichers
    fn load_program_to_memory(&mut self, program: &[u8]) {
        // Stellen Sie sicher, dass die Byte-Länge ein Vielfaches von 4 ist
        if program.len() % 4 != 0 {
            panic!("Program length must be a multiple of 4 bytes");
        }
        
        // Wandeln Sie das Byte-Array in 32-Bit-Instruktionen um und schreiben Sie sie in die Code-Region
        for (i, chunk) in program.chunks(4).enumerate() {
            if chunk.len() == 4 {
                let raw_instr = RawInstruction::from_bytes(chunk[0], chunk[1], chunk[2], chunk[3]);
                let value = raw_instr.as_i32();
                
                // Schreiben Sie in die Code-Region mit dem entsprechenden Offset
                if let Err(e) = self.memory.write_to_region(MemoryRegionType::Code, i, value) {
                    // Falls Code-Region nicht beschreibbar ist (z.B. im Standard-Layout),
                    // versuchen wir direkt in den Speicher zu schreiben
                    if let Err(e2) = self.memory.write(i, value) {
                        panic!("Failed to load program to memory: {} and fallback failed: {}", e, e2);
                    }
                }
            }
        }
    }

    pub fn step(&mut self) -> bool {
        if let Some(instruction_raw) = self.fetcher.fetch() {
            let instruction = self.decoder.decode(instruction_raw);
            let jump_address = self.executor.execute(&instruction, &mut self.stack, &mut self.memory);
            
            // Handle jump instructions
            if let Some(address) = jump_address {
                self.fetcher.set_pc(address);
            }
            
            true
        } else {
            false
        }
    }
}

pub struct ALU; 

impl ALU {
    pub fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
    
    pub fn sub(&self, a: i32, b: i32) -> i32 {
        a - b
    }
    
    pub fn multiply(&self, a: i32, b: i32) -> i32 {
        a * b
    }

    pub fn divide(&self, a: i32, b: i32) -> i32 {
        a / b
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
    
    pub fn set_pc(&mut self, address: usize) {
        if address % 4 != 0 {
            panic!("Program counter must be aligned to 4 bytes");
        }
        if address >= self.program.len() {
            panic!("Program counter out of bounds");
        }
        self.pc = address;
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
    output: Stdout,
}

impl InstructionExecutor {

    pub fn new() -> Self {
        Self { 
            alu: ALU,
            output: stdout()
        }
    }

    // Return Some(address) if a jump should occur, otherwise None
    pub fn execute(&mut self, instruction: &Instruction, stack: &mut VecDeque<i32>, memory: &mut SegmentedMemory) -> Option<usize> {
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
                        None
                    },
            Opcode::Store => {
                        if instruction.opcode.operand_count() != 1 {
                            panic!("Store instruction requires 1 operand");
                        }

                        let value = instruction.operands[0];

                        // push the value to the stack
                        stack.push_back(value);
                        None
                    },
            Opcode::Sub => {
                        if instruction.opcode.operand_count() > 0 {
                            panic!("Sub instruction requires 0 operands");
                        }

                        let a = stack.pop_back();
                        let b = stack.pop_back();
                
                        if let (Some(a), Some(b)) = (a, b) {
                            // b - a (pop order)
                            let result = self.alu.sub(b, a);
                            stack.push_back(result);
                        } else {
                            panic!("Stack underflow");
                        }
                        None
                    },
            Opcode::LoadConstant => {
                        if instruction.opcode.operand_count() != 1 {
                            panic!("LoadConstant instruction requires 1 operand");
                        }

                        let value = instruction.operands[0];
                        // push the value to the stack, like Store
                        stack.push_back(value);
                        None
                    },
            Opcode::Multiply => {
                        if instruction.opcode.operand_count() > 0 {
                            panic!("Multiply instruction requires 0 operands");
                        }

                        let a = stack.pop_back();
                        let b = stack.pop_back();
                
                        if let (Some(a), Some(b)) = (a, b) {
                            let result = self.alu.multiply(a, b);
                            stack.push_back(result);
                        } else {
                            panic!("Stack underflow");
                        }
                        None
                    },
            Opcode::Print => {
                        if instruction.opcode.operand_count() > 0 {
                            panic!("Print instruction requires 0 operands");
                        }

                        if let Some(value) = stack.back() {
                            writeln!(self.output, "{}", value).expect("Failed to write to stdout");
                        } else {
                            panic!("Stack underflow");
                        }
                        None
                    },
            Opcode::LoadMemory => {
                        if instruction.opcode.operand_count() != 1 {
                            panic!("LoadMemory instruction requires 1 operand");
                        }

                        let address = instruction.operands[0] as usize;
                        match memory.read(address) {
                            Ok(value) => stack.push_back(value),
                            Err(e) => panic!("Memory error: {}", e),
                        }
                        None
                    },
            Opcode::StoreMemory => {
                        if instruction.opcode.operand_count() != 1 {
                            panic!("StoreMemory instruction requires 1 operand");
                        }

                        let address = instruction.operands[0] as usize;
                        let value = stack.pop_back().expect("Stack underflow");
                
                        match memory.write(address, value) {
                            Ok(_) => {},
                            Err(e) => panic!("Memory error: {}", e),
                        }
                        None
                    },
            Opcode::JumpIfZero => {
                        if instruction.opcode.operand_count() != 1 {
                            panic!("JumpIfZero instruction requires 1 operand");
                        }

                        let jump_address = instruction.operands[0] as usize;
                
                        // The address should be a multiple of 4 (instruction size)
                        if jump_address % 4 != 0 {
                            panic!("Jump address must be aligned to 4 bytes");
                        }
                
                        // Jump if top of stack is zero
                        if let Some(value) = stack.pop_back() {
                            if value == 0 {
                                // Jump to the specified address
                                return Some(jump_address);
                            }
                        } else {
                            panic!("Stack underflow in JumpIfZero");
                        }
                        None
                    },
            Opcode::Divide =>   {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Divide instruction requires 0 operands");
                }

                let a = stack.pop_back();
                let b = stack.pop_back();

                if let (Some(a), Some(b)) = (a, b) {
                    let result = self.alu.divide(b, a);
                    stack.push_back(result);
                } else {
                    panic!("Stack underflow");
                }

                None
            },
            // Neue Opcodes, die mit Speicherregionen arbeiten
            Opcode::LoadFromRegion => {
                let region_id = instruction.operands[0] as usize;
                let offset = instruction.operands[1] as usize;
                
                // Region-ID in RegionType umwandeln
                let region_type = match region_id {
                    0 => MemoryRegionType::Code,
                    1 => MemoryRegionType::Data,
                    2 => MemoryRegionType::Stack,
                    3 => MemoryRegionType::Heap,
                    4 => MemoryRegionType::Constants,
                    5 => MemoryRegionType::IO,
                    _ => return None,
                };
                
                let value = memory.read_from_region(region_type, offset);
                match value {
                    Ok(value) => stack.push_back(value),
                    Err(e) => panic!("Memory error: {}", e),
                }
                None
            },
            
            Opcode::StoreToRegion => {
                let region_id = instruction.operands[0] as usize;
                let offset = instruction.operands[1] as usize;
                let value = stack.pop_back().unwrap_or(0);
                
                // Region-ID in RegionType umwandeln
                let region_type = match region_id {
                    0 => MemoryRegionType::Code,
                    1 => MemoryRegionType::Data,
                    2 => MemoryRegionType::Stack,
                    3 => MemoryRegionType::Heap,
                    4 => MemoryRegionType::Constants,
                    5 => MemoryRegionType::IO,
                    _ => return None,
                };
                
                match memory.write_to_region(region_type, offset, value) {
                    Ok(_) => None,
                    Err(e) => panic!("Memory error: {}", e),
                }
            },
        }
    }
}

pub struct VirtualMachine {
    pub cpu: CPU,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self { cpu: CPU::new(10000) }
    }

    // Konstruktor mit angegebener Speichergröße
    pub fn with_memory_size(size: usize) -> Self {
        Self { cpu: CPU::new(size) }
    }

    pub fn run(&mut self) -> bool {
        self.cpu.step()
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.cpu.load_program(program);
    }

    pub fn run_until_completion(&mut self) {
        while self.run() {}
    }
    
    pub fn stack_top(&self) -> Option<i32> {
        self.cpu.stack.back().copied()
    }
    
    /// Gibt eine Debugansicht des Speichers aus
    pub fn print_memory_map(&self) {
        self.cpu.memory.print_memory_map();
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
        assert_eq!(vm.stack_top(), Some(12));
    }
    
    #[test]
    fn test_new_opcodes() {
        // Test Sub instruction: 10 - 4 = 6
        let program = [
            // LoadConstant 10
            0x00, 0x00, 0x0A, 0x04,
            // LoadConstant 4
            0x00, 0x00, 0x04, 0x04,
            // Sub instruction (10 - 4)
            0x00, 0x00, 0x00, 0x03
        ];
        
        let mut vm = VirtualMachine::new();
        vm.load_program(&program);
        vm.run_until_completion();
        
        // Check if result is 6 (10 - 4)
        assert_eq!(vm.stack_top(), Some(6));
    }
    
    #[test]
    fn test_multiply() {
        let program = [
            // LoadConstant 6
            0x00, 0x00, 0x06, 0x04,
            // LoadConstant 7
            0x00, 0x00, 0x07, 0x04,
            // Multiply
            0x00, 0x00, 0x00, 0x05
        ];
        
        let mut vm = VirtualMachine::new();
        vm.load_program(&program);
        vm.run_until_completion();
        
        // Check if result is 42 (6 * 7)
        assert_eq!(vm.stack_top(), Some(42));
    }
    
    #[test]
    fn test_memory_operations() {
        let program = [
            // LoadConstant 42 (value to store)
            0x00, 0x00, 0x2A, 0x04,
            // StoreMemory at address 100
            0x00, 0x00, 0x64, 0x08,
            // LoadMemory from address 100
            0x00, 0x00, 0x64, 0x07
        ];
        
        let mut vm = VirtualMachine::new();
        vm.load_program(&program);
        vm.run_until_completion();
        
        // Check if we loaded the same value we stored (42)
        assert_eq!(vm.stack_top(), Some(42));
    }
    
    #[test]
    fn test_jump_if_zero() {
        // Create a simple program to count from 3 down to 0
        // and sum the numbers (3+2+1=6) with proper padding
        let program = [
            // Initialize memory[0] with counter = 3
            0x00, 0x00, 0x03, 0x04, // (0) LoadConstant 3
            0x00, 0x00, 0x00, 0x08, // (4) StoreMemory 0
            
            // Initialize memory[1] with sum = 0
            0x00, 0x00, 0x00, 0x04, // (8) LoadConstant 0
            0x00, 0x00, 0x01, 0x08, // (12) StoreMemory 1
            
            // Initialize memory[2] with constant 0 to use for jump condition
            0x00, 0x00, 0x00, 0x04, // (16) LoadConstant 0
            0x00, 0x00, 0x02, 0x08, // (20) StoreMemory 2
            
            // START LOOP (byte 24)
            // Check if counter is 0, if yes exit loop
            0x00, 0x00, 0x00, 0x07, // (24) LoadMemory 0 (counter)
            0x00, 0x00, 0x58, 0x09, // (28) JumpIfZero 88 (exit loop)
            
            // Add counter to sum
            0x00, 0x00, 0x00, 0x07, // (32) LoadMemory 0 (counter)
            0x00, 0x00, 0x01, 0x07, // (36) LoadMemory 1 (sum)
            0x00, 0x00, 0x00, 0x01, // (40) Add
            0x00, 0x00, 0x01, 0x08, // (44) StoreMemory 1 (update sum)
            
            // Decrement counter
            0x00, 0x00, 0x00, 0x07, // (48) LoadMemory 0 (counter)
            0x00, 0x00, 0x01, 0x04, // (52) LoadConstant 1
            0x00, 0x00, 0x00, 0x03, // (56) Sub
            0x00, 0x00, 0x00, 0x08, // (60) StoreMemory 0 (update counter)
            
            // Load 0 from memory to use as jump condition (always true)
            0x00, 0x00, 0x02, 0x07, // (64) LoadMemory 2 (constant 0)
            0x00, 0x00, 0x18, 0x09, // (68) JumpIfZero 24 (go back to start of loop)
            
            // Padding instructions (not executed, just to make sure we have enough space)
            0x00, 0x00, 0x00, 0x00, // (72) padding
            0x00, 0x00, 0x00, 0x00, // (76) padding
            0x00, 0x00, 0x00, 0x00, // (80) padding
            0x00, 0x00, 0x00, 0x00, // (84) padding
            
            // LOOP EXIT (byte 88)
            // Load the sum to check the result
            0x00, 0x00, 0x01, 0x07, // (88) LoadMemory 1 (sum)
        ];
        
        let mut vm = VirtualMachine::new();
        vm.load_program(&program);
        vm.run_until_completion();
        
        // The sum should be 6 (3+2+1)
        assert_eq!(vm.stack_top(), Some(6));
    }

    #[test]
    fn test_divide() {
        let program = [
            // LoadConstant 10
            0x00, 0x00, 0x0A, 0x04,
            // LoadConstant 2
            0x00, 0x00, 0x02, 0x04,
            // Divide
            0x00, 0x00, 0x00, 0x0A
        ];
        
        let mut vm = VirtualMachine::new();
        vm.load_program(&program);
        vm.run_until_completion();
        
        // The result should be 5 (10 / 2)
        assert_eq!(vm.stack_top(), Some(5));
    }
}