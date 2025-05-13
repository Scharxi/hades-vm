/// Instruction Processing for the Hades VM.
///
/// This module contains components for fetching, decoding and executing
/// VM instructions, forming the core of the instruction cycle.
use std::io::{Stdout, Write, stdout};

use crate::{
    alu::ALU,
    instruction::{Instruction, RawInstruction},
    memory::{MemoryRegionType, SegmentedMemory},
    opcode::Opcode,
    stack::{Stack, StackFrame, StackValue},
};

/// Fetches instructions from the program.
///
/// The InstructionFetcher maintains a program counter (PC) and
/// reads raw instructions from the program data.
pub struct InstructionFetcher {
    /// Current program counter (position in program)
    pub pc: usize,
    /// Program data as a byte array
    pub program: Vec<u8>,
}

impl InstructionFetcher {
    /// Creates a new InstructionFetcher with an empty program.
    pub fn new() -> Self {
        Self {
            program: vec![],
            pc: 0,
        }
    }

    /// Loads a new program and resets the program counter.
    pub fn load_program(&mut self, program: &[u8]) {
        self.program = program.to_vec();
        self.pc = 0;
    }

    /// Peeks at the next instruction without advancing the program counter.
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

    /// Fetches the next instruction and advances the program counter.
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

    /// Sets the program counter to a new address.
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

/// Decodes raw instructions into executable instructions.
///
/// The InstructionDecoder extracts the opcode and operands from
/// raw instruction data.
pub struct InstructionDecoder;

impl InstructionDecoder {
    /// Decodes a raw instruction into an executable instruction.
    pub fn decode(&self, instruction: RawInstruction) -> Instruction {
        // the last 8 bits are the opcode
        let opcode = instruction.opcode();
        let operands = instruction.get_operands();

        // Extract operands based on the opcode's operand count
        Instruction { opcode, operands }
    }
}

/// Executes decoded instructions.
///
/// The InstructionExecutor contains the implementation of all
/// instruction behaviors, operating on the stack and memory.
pub struct InstructionExecutor {
    /// Arithmetic Logic Unit for mathematical operations
    alu: ALU,
    /// Output for print instructions
    output: Stdout,
}

impl InstructionExecutor {
    /// Creates a new InstructionExecutor.
    pub fn new() -> Self {
        Self {
            alu: ALU,
            output: stdout(),
        }
    }

    /// Executes a single instruction.
    ///
    /// This method implements the behavior of all supported opcodes.
    /// It operates on the stack and memory, and returns a jump address
    /// if the instruction requires a jump (e.g., Call, Return, JumpIfZero).
    ///
    /// # Returns
    /// - `Some(address)` if a jump should occur
    /// - `None` if no jump is needed
    pub fn execute(
        &mut self,
        instruction: &Instruction,
        stack: &mut Stack,
        memory: &mut SegmentedMemory,
    ) -> Option<usize> {
        match instruction.opcode {
            Opcode::Add => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Add instruction requires 0 operands");
                }

                let a = stack.pop();
                let b = stack.pop();

                if let (Some(a), Some(b)) = (a, b) {
                    match (a, b) {
                        (StackValue::Integer(a_val), StackValue::Integer(b_val)) => {
                            let result = self.alu.add_int(a_val, b_val);
                            stack.push(StackValue::Integer(result));
                        }
                        (StackValue::Float(a_val), StackValue::Float(b_val)) => {
                            let result = self.alu.add_float(a_val, b_val);
                            stack.push(StackValue::Float(result));
                        }
                        _ => panic!("Type mismatch in Add operation"),
                    }
                } else {
                    panic!("Stack underflow");
                }
                None
            }
            Opcode::Store => {
                if instruction.opcode.operand_count() != 1 {
                    panic!("Store instruction requires 1 operand");
                }

                let value = instruction.operands[0];

                // push the value to the stack as Integer
                stack.push(StackValue::Integer(value));
                None
            }
            Opcode::Sub => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Sub instruction requires 0 operands");
                }

                let a = stack.pop();
                let b = stack.pop();

                if let (Some(a), Some(b)) = (a, b) {
                    match (a, b) {
                        (StackValue::Integer(a_val), StackValue::Integer(b_val)) => {
                            // b - a (pop order)
                            let result = self.alu.sub_int(b_val, a_val);
                            stack.push(StackValue::Integer(result));
                        }
                        (StackValue::Float(a_val), StackValue::Float(b_val)) => {
                            let result = self.alu.sub_float(b_val, a_val);
                            stack.push(StackValue::Float(result));
                        }
                        _ => panic!("Type mismatch in Sub operation"),
                    }
                } else {
                    panic!("Stack underflow");
                }
                None
            }
            Opcode::LoadConstant => {
                if instruction.opcode.operand_count() != 1 {
                    panic!("LoadConstant instruction requires 1 operand");
                }

                let value = instruction.operands[0];
                // push the value to the stack as Integer
                stack.push(StackValue::Integer(value));
                None
            }
            Opcode::Multiply => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Multiply instruction requires 0 operands");
                }

                let a = stack.pop();
                let b = stack.pop();

                if let (Some(a), Some(b)) = (a, b) {
                    match (a, b) {
                        (StackValue::Integer(a_val), StackValue::Integer(b_val)) => {
                            let result = self.alu.multiply_int(a_val, b_val);
                            stack.push(StackValue::Integer(result));
                        }
                        (StackValue::Float(a_val), StackValue::Float(b_val)) => {
                            let result = self.alu.multiply_float(a_val, b_val);
                            stack.push(StackValue::Float(result));
                        }
                        _ => panic!("Type mismatch in Multiply operation"),
                    }
                } else {
                    panic!("Stack underflow");
                }
                None
            }
            Opcode::Print => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Print instruction requires 0 operands");
                }

                if let Some(value) = stack.peek() {
                    match value {
                        StackValue::Integer(i) => writeln!(self.output, "Integer: {}", i),
                        StackValue::Float(f) => writeln!(self.output, "Float: {}", f),
                        StackValue::Boolean(b) => writeln!(self.output, "Boolean: {}", b),
                        StackValue::Reference(r) => writeln!(self.output, "Reference: 0x{:x}", r),
                    }
                    .expect("Failed to write to stdout");
                } else {
                    panic!("Stack underflow");
                }
                None
            }
            Opcode::LoadMemory => {
                if instruction.opcode.operand_count() != 1 {
                    panic!("LoadMemory instruction requires 1 operand");
                }

                let address = instruction.operands[0] as usize;
                match memory.read(address) {
                    Ok(value) => stack.push(StackValue::Integer(value)),
                    Err(e) => panic!("Memory error: {}", e),
                }
                None
            }
            Opcode::StoreMemory => {
                if instruction.opcode.operand_count() != 1 {
                    panic!("StoreMemory instruction requires 1 operand");
                }

                let address = instruction.operands[0] as usize;
                let value = stack.pop().expect("Stack underflow");

                match value {
                    StackValue::Integer(i) => match memory.write(address, i) {
                        Ok(_) => {}
                        Err(e) => panic!("Memory error: {}", e),
                    },
                    _ => panic!("Can only store integers in memory"),
                }
                None
            }
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
                if let Some(value) = stack.pop() {
                    if value.is_zero() {
                        // Jump to the specified address
                        return Some(jump_address);
                    }
                } else {
                    panic!("Stack underflow in JumpIfZero");
                }
                None
            }
            Opcode::Divide => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Divide instruction requires 0 operands");
                }

                let a = stack.pop();
                let b = stack.pop();

                if let (Some(a), Some(b)) = (a, b) {
                    match (a, b) {
                        (StackValue::Integer(a_val), StackValue::Integer(b_val)) => {
                            if a_val == 0 {
                                panic!("Division by zero");
                            }
                            let result = self.alu.divide_int(b_val, a_val);
                            stack.push(StackValue::Integer(result));
                        }
                        (StackValue::Float(a_val), StackValue::Float(b_val)) => {
                            if a_val == 0.0 {
                                panic!("Division by zero");
                            }
                            let result = self.alu.divide_float(b_val, a_val);
                            stack.push(StackValue::Float(result));
                        }
                        _ => panic!("Type mismatch in Divide operation"),
                    }
                } else {
                    panic!("Stack underflow");
                }

                None
            }
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
                    Ok(value) => stack.push(StackValue::Integer(value)),
                    Err(e) => panic!("Memory error: {}", e),
                }
                None
            }
            Opcode::StoreToRegion => {
                let region_id = instruction.operands[0] as usize;
                let offset = instruction.operands[1] as usize;
                let value = stack.pop().unwrap_or(StackValue::Integer(0));

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

                match value {
                    StackValue::Integer(i) => {
                        match memory.write_to_region(region_type, offset, i) {
                            Ok(_) => None,
                            Err(e) => panic!("Memory error: {}", e),
                        }
                    }
                    _ => panic!("Can only store integers in memory regions"),
                }
            }
            Opcode::Call => {
                let address = instruction.operands[0] as usize;
                let local_count = instruction.operands[1] as usize;

                // Sicherstellen, dass genügend Elemente auf dem Stack liegen
                if stack.values.len() < local_count {
                    panic!(
                        "Stack underflow in Call: Not enough values on stack for {} parameters",
                        local_count
                    );
                }

                // Parameter sind bereits auf dem Stack
                let base_pointer = if local_count > 0 {
                    stack.values.len() - local_count
                } else {
                    stack.values.len()
                };

                // Das Return-Address-Handling sollte in der CPU passieren
                // Hier verwenden wir einen temporären Wert
                let return_address = address + 4;

                // Frame erstellen
                let frame = StackFrame {
                    return_address,
                    base_pointer,
                    local_count,
                };

                // Debug-Ausgabe
                println!(
                    "Call: addr={}, locals={}, base={}, stack={:?}",
                    address, local_count, base_pointer, stack.values
                );

                // Frame hinzufügen
                stack.frames.push(frame);
                stack.current_frame = Some(stack.frames.len() - 1);

                // Springe zur Funktionsadresse
                Some(address)
            }
            Opcode::Return => {
                // Muss mindestens einen Frame haben
                if stack.frames.is_empty() {
                    panic!("Return without call frame");
                }

                // Aktuellen Frame holen
                let frame_idx = stack.current_frame.unwrap();
                let current_frame = stack.frames[frame_idx].clone();
                let return_address = current_frame.return_address;
                let base_pointer = current_frame.base_pointer;

                // Ist ein Rückgabewert auf dem Stack?
                let return_value = if stack.values.len() > base_pointer {
                    Some(stack.values.last().unwrap().clone())
                } else {
                    None
                };

                // Debug-Ausgabe
                println!(
                    "Return: addr={}, base={}, stack={:?}, return_value={:?}",
                    return_address, base_pointer, stack.values, return_value
                );

                // Lokale Variablen und Parameter entfernen
                stack.values.truncate(base_pointer);

                // Frame entfernen
                stack.frames.pop();
                stack.current_frame = if stack.frames.is_empty() {
                    None
                } else {
                    Some(stack.frames.len() - 1)
                };

                // Rückgabewert (falls vorhanden) wieder auf den Stack legen
                if let Some(value) = return_value {
                    stack.values.push(value);
                }

                // Zur Rücksprungadresse zurückkehren
                Some(return_address)
            }
            Opcode::LoadLocal => {
                let local_index = instruction.operands[0] as usize;

                // Lokale Variablen vom aktuellen Frame laden
                if let Some(frame_idx) = stack.current_frame {
                    // Sicherstellen, dass der Frame-Index gültig ist
                    if frame_idx >= stack.frames.len() {
                        panic!("LoadLocal: Invalid frame index: {}", frame_idx);
                    }

                    let frame = &stack.frames[frame_idx];

                    // Sicherstellen, dass der lokale Index gültig ist
                    if local_index >= frame.local_count {
                        panic!(
                            "LoadLocal: Local index {} out of bounds (local_count={})",
                            local_index, frame.local_count
                        );
                    }

                    // Debug-Ausgabe
                    println!(
                        "LoadLocal: frame={}, idx={}, base={}, stack={:?}",
                        frame_idx, local_index, frame.base_pointer, stack.values
                    );

                    // Berechne den tatsächlichen Index im Stack
                    let stack_index = frame.base_pointer + local_index;

                    // Überprüfe, ob der berechnete Index im Stack-Bereich liegt
                    if stack_index >= stack.values.len() {
                        panic!(
                            "LoadLocal: Stack index {} out of bounds (stack_len={})",
                            stack_index,
                            stack.values.len()
                        );
                    }

                    // Lade den Wert und füge ihn zum Stack hinzu
                    let value = stack.values[stack_index];
                    stack.push(value);

                    None
                } else {
                    panic!("LoadLocal: No active stack frame");
                }
            }
            Opcode::StoreLocal => {
                let local_index = instruction.operands[0] as usize;
                let value = stack.pop().expect("Stack underflow");

                // Store value in local variable
                if let Some(frame_idx) = stack.current_frame {
                    if frame_idx < stack.frames.len() {
                        let frame = &stack.frames[frame_idx];
                        // Berechne den tatsächlichen Index im Stack
                        let stack_index = frame.base_pointer + local_index;

                        // Überprüfe, ob der Index gültig ist
                        if stack_index < stack.values.len() {
                            // Speichere den Wert in den Stack an der entsprechenden Position
                            stack.values[stack_index] = value;
                            None
                        } else {
                            panic!(
                                "Invalid local variable index: {} (stack index: {})",
                                local_index, stack_index
                            );
                        }
                    } else {
                        panic!("Invalid frame index");
                    }
                } else {
                    panic!("No active stack frame");
                }
            }
            Opcode::Modulo => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Modulo instruction requires 0 operands");
                }

                let a = stack.pop();
                let b = stack.pop();

                if let (Some(a), Some(b)) = (a, b) {
                    match (a, b) {
                        (StackValue::Integer(a_val), StackValue::Integer(b_val)) => {
                            let result = self.alu.modulo(b_val, a_val);
                            stack.push(StackValue::Integer(result));
                        }
                        _ => panic!("Type mismatch in Modulo operation"),
                    }
                } else {
                    panic!("Stack underflow");
                }
                None
            }
            Opcode::Power => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Power instruction requires 0 operands");
                }

                let a = stack.pop();
                let b = stack.pop();

                if let (Some(a), Some(b)) = (a, b) {
                    match (a, b) {
                        (StackValue::Integer(a_val), StackValue::Integer(b_val)) => {
                            let result = self.alu.power(b_val, a_val);
                            stack.push(StackValue::Integer(result));
                        }
                        _ => panic!("Type mismatch in Power operation"),
                    }
                } else {
                    panic!("Stack underflow");
                }
                None
            }
            Opcode::PickN => {
                if instruction.opcode.operand_count() != 1 {
                    panic!("PickN instruction requires 1 operand");
                }

                let n = instruction.operands[0] as usize;
                if n > stack.values.len() {
                    panic!("PickN: n is greater than the stack size");
                }

                stack.copy_nth_to_top(n);
                None
            }
            Opcode::Dup => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Dup instruction requires 0 operands");
                }

                let value = stack.peek().expect("Stack underflow");
                stack.push(*value);
                None
            }
            Opcode::Swap => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Swap instruction requires 0 operands");
                }

                let a = stack.pop().expect("Stack underflow");
                let b = stack.pop().expect("Stack underflow");
                stack.push(a);
                stack.push(b);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opcode::Opcode;

    #[test]
    fn test_instruction_fetcher() {
        // Create a simple program with correct byte ordering
        let program = vec![
            0x00, 0x00, 0x00, 0x01, // Add (0x01)
            0x00, 0x00, 0x00, 0x02, // Store (0x02)
        ];

        let mut fetcher = InstructionFetcher::new();
        fetcher.load_program(&program);

        // Fetch first instruction
        let instruction1 = fetcher.fetch();
        assert!(instruction1.is_some());
        let instruction1 = instruction1.unwrap();
        assert_eq!(instruction1.opcode(), Opcode::Add);

        // Fetch second instruction
        let instruction2 = fetcher.fetch();
        assert!(instruction2.is_some());
        let instruction2 = instruction2.unwrap();
        assert_eq!(instruction2.opcode(), Opcode::Store);

        // No more instructions
        let instruction3 = fetcher.fetch();
        assert!(instruction3.is_none());
    }

    #[test]
    fn test_instruction_decoder() {
        let decoder = InstructionDecoder;

        // Test decoding Add (no operands)
        let raw = RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x01);
        let instruction = decoder.decode(raw);
        assert_eq!(instruction.opcode, Opcode::Add);
        assert!(instruction.operands.is_empty());

        // Test decoding Store (with operand)
        // First byte is the operand (big endian)
        let raw = RawInstruction::from_bytes(0x2A, 0x00, 0x00, 0x02);
        let instruction = decoder.decode(raw);
        assert_eq!(instruction.opcode, Opcode::Store);
        assert_eq!(instruction.operands, vec![0x2A0000]);
    }
}
