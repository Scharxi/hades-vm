/// CPU implementation for the Hades VM.
///
/// This module contains the central processing unit (CPU) implementation,
/// which is responsible for the fetch-decode-execute cycle.

use crate::{
    memory::{MemoryRegionType, SegmentedMemory},
    instruction_processor::{InstructionFetcher, InstructionDecoder, InstructionExecutor},
    stack::Stack,
    instruction::RawInstruction,
    opcode::Opcode,
};

/// The central processing unit of the virtual machine.
///
/// The CPU is responsible for executing the instruction cycle:
/// 1. Fetch instructions from memory (via InstructionFetcher)
/// 2. Decode instructions (via InstructionDecoder)
/// 3. Execute instructions (via InstructionExecutor)
///
/// It also manages the stack and memory access.
pub struct CPU {
    /// Fetches raw instructions from the program
    pub fetcher: InstructionFetcher, 
    /// Decodes raw instructions into executable instructions
    pub decoder: InstructionDecoder, 
    /// Executes decoded instructions
    pub executor: InstructionExecutor,
    /// The stack for storing values and execution context
    pub stack: Stack,
    /// The memory system with segmented regions
    pub memory: SegmentedMemory, 
}

impl CPU {
    /// Creates a new CPU with the specified memory size.
    ///
    /// This initializes the CPU components (fetcher, decoder, executor),
    /// creates a stack, and sets up the memory layout depending on the
    /// context (test or production).
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
            stack: Stack::new(256),
            memory,
        }
    }

    /// Loads a program into memory and initializes the instruction fetcher.
    pub fn load_program(&mut self, program: &[u8]) {
        // Programm in den Speicher in die Code-Region laden
        self.load_program_to_memory(program);
        
        // Anschließend den InstructionFetcher mit dem Programm initialisieren
        self.fetcher.load_program(program);
    }
    
    /// Loads the program into the Code region of memory.
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

    /// Executes a single instruction cycle (fetch, decode, execute).
    ///
    /// Returns true if an instruction was executed, false if there are no more instructions.
    pub fn step(&mut self) -> bool {
        if let Some(instruction_raw) = self.fetcher.fetch() {
            let instruction = self.decoder.decode(instruction_raw);
            
            // Für Call-Opcode: Der aktuelle PC nach Fetch ist bereits die Rückkehradresse
            let return_pc = self.fetcher.pc;
            
            // Die tatsächliche Ausführung            
            let jump_address = self.executor.execute(&instruction, &mut self.stack, &mut self.memory);
            
            // Bei einem Call-Opcode, setzen wir die korrekte Rückkehradresse im Stack-Frame
            if instruction.opcode == Opcode::Call && jump_address.is_some() {
                // Finde den zuletzt erstellten Frame
                if let Some(frame_idx) = self.stack.current_frame {
                    if frame_idx < self.stack.frames.len() {
                        // Aktualisiere die Rückkehradresse
                        self.stack.frames[frame_idx].return_address = return_pc;
                    }
                }
            }
            
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack::StackValue;

    #[test]
    fn test_cpu_execution() {
        // Create a simple program that adds two numbers (3 + 4 = 7)
        // Format: opcode is last byte, operand is 24 bits [0-2]
        let program = vec![
            0x00, 0x00, 0x03, 0x04, // LoadConstant (opcode 4) with operand 3
            0x00, 0x00, 0x04, 0x04, // LoadConstant (opcode 4) with operand 4
            0x00, 0x00, 0x00, 0x01, // Add (opcode 1)
        ];
        
        let mut cpu = CPU::new(1000);
        cpu.load_program(&program);
        
        // Execute the program
        assert!(cpu.step()); // LoadConstant 3
        assert!(cpu.step()); // LoadConstant 4
        assert!(cpu.step()); // Add
        
        // Check the result
        assert_eq!(cpu.stack.peek(), Some(&StackValue::Integer(7)));
        
        // No more instructions
        assert!(!cpu.step());
    }

    #[test]
    fn test_cpu_execution_with_pickn() {
        let program = vec![
            0x00, 0x00, 0x03, 0x04, // LoadConstant (opcode 4) with operand 3
            0x00, 0x00, 0x04, 0x04, // LoadConstant (opcode 4) with operand 4
            0x00, 0x00, 0x00, 0x13, // PickN (opcode 19) with operand 1
        ];

        let mut cpu = CPU::new(1000);
        cpu.load_program(&program);
        
        assert!(cpu.step()); // LoadConstant 3
        assert!(cpu.step()); // LoadConstant 4
        assert!(cpu.step()); // PickN
        
        assert_eq!(cpu.stack.peek(), Some(&StackValue::Integer(4)));
    }

    #[test]
    fn test_cpu_execution_with_dup() {
        let program = vec![
            0x00, 0x00, 0x03, 0x04, // LoadConstant (opcode 4) with operand 3
            0x00, 0x00, 0x00, 0x14, // Dup (opcode 20)
        ];

        let mut cpu = CPU::new(1000);
        cpu.load_program(&program);
        
        assert!(cpu.step()); // LoadConstant 3
        assert!(cpu.step()); // Dup
        
        assert_eq!(cpu.stack.peek(), Some(&StackValue::Integer(3)));
        assert_eq!(cpu.stack.peek(), Some(&StackValue::Integer(3)));
        assert_eq!(cpu.stack.len(), 2);
        }

    #[test]
    fn test_cpu_execution_with_swap() {
        let program = vec![
            0x00, 0x00, 0x03, 0x04, // LoadConstant (opcode 4) with operand 3
            0x00, 0x00, 0x04, 0x04, // LoadConstant (opcode 4) with operand 4
            0x00, 0x00, 0x00, 0x15, // Swap (opcode 21)
        ];

        let mut cpu = CPU::new(1000);
        cpu.load_program(&program);
        
        assert!(cpu.step()); // LoadConstant 3
        assert!(cpu.step()); // LoadConstant 4
        assert!(cpu.step()); // Swap - Need to execute this step to perform the swap
        
        assert_eq!(cpu.stack.peek(), Some(&StackValue::Integer(3)));
        assert_eq!(cpu.stack.len(), 2);
    }

    #[test]
    fn test_cpu_execution_with_drop() {
        let program = vec![
            0x00, 0x00, 0x03, 0x04, // LoadConstant (opcode 4) with operand 3
            0x00, 0x00, 0x00, 0x16, // Drop (opcode 22)
        ];

        let mut cpu = CPU::new(1000);
        cpu.load_program(&program);
        
        assert!(cpu.step()); // LoadConstant 3
        assert!(cpu.step()); // Drop
        
        assert_eq!(cpu.stack.len(), 0);
    }
    
} 