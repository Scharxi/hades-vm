/// Instruction Processing for the Hades VM.
///
/// This module contains components for fetching, decoding and executing
/// VM instructions, forming the core of the instruction cycle.
use std::io::{Stdout, Write, stdout};

use crate::{
    alu::ALU,
    instruction::{Instruction, RawInstruction},
    memory::{MemoryRegionType, SegmentedMemory},
    opcode::{Opcode, InvalidOpcodeError},
    stack::{Stack, StackFrame, StackValue},
};

/// Signal, das von der `execute`-Methode zurückgegeben wird, um den VM-Zyklus zu steuern.
#[derive(Debug, PartialEq)]
pub enum ExecutionSignal {
    /// Die Ausführung soll mit der nächsten Instruktion fortgesetzt werden.
    Continue,
    /// Ein Sprung zu einer bestimmten Speicheradresse ist erforderlich.
    Jump(usize),
    /// Der aktuelle Prozess gibt die Kontrolle frei (Kontextwechsel).
    Yield,
    /// Der aktuelle Prozess soll beendet werden.
    Terminate,
    // Hier könnten weitere Signale für Systemaufrufe, Fehlerbehandlung etc. folgen.
}

/// Mögliche Zustände eines Prozesses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessState {
    Ready,     // Bereit zur Ausführung
    Running,   // Wird aktuell ausgeführt
    Blocked,   // Wartet auf ein Ereignis (z.B. I/O)
    Terminated, // Prozess wurde beendet
}

/// Process Control Block (PCB) zur Speicherung des Zustands eines Prozesses.
pub struct ProcessControlBlock {
    pub pid: u32,                      // Eindeutige Prozess-ID
    pub pc: usize,                     // Program Counter des Prozesses
    pub stack: Stack,                  // Der Stack des Prozesses
    pub state: ProcessState,           // Aktueller Zustand des Prozesses
    // Optional: memory_context: Für komplexeres Speichermanagement pro Prozess
}

/// Fetches instructions from the program.
///
/// The InstructionFetcher maintains a program counter (PC) and
/// reads raw instructions from the program data.
pub struct InstructionFetcher {
    /// Current program counter (position in program)
    pub pc: usize,
    // Program data as a byte array - REMOVED
    // pub program: Vec<u8>,
}

impl InstructionFetcher {
    /// Creates a new InstructionFetcher with an empty program.
    pub fn new() -> Self {
        Self {
            // program: vec![], // REMOVED
            pc: 0,
        }
    }

    // Loads a new program and resets the program counter. - REMOVED
    // pub fn load_program(&mut self, program: &[u8]) {
    //     self.program = program.to_vec();
    //     self.pc = 0;
    // }

    /// Peeks at the next instruction without advancing the program counter.
    /// TODO: This needs to be properly refactored to use SegmentedMemory as well.
    /// For now, returning None to avoid issues with removed self.program.
    pub fn peek_next(&self, _memory: &SegmentedMemory) -> Option<RawInstruction> {
        // if self.pc + 3 < self.program.len() {
        //     let b0 = self.program[self.pc];
        //     let b1 = self.program[self.pc + 1];
        //     let b2 = self.program[self.pc + 2];
        //     let b3 = self.program[self.pc + 3];

        //     Some(RawInstruction::from_bytes(b0, b1, b2, b3))
        // } else {
        //     None
        // }
        None // Temporary fix
    }

    /// Fetches the next instruction and advances the program counter.
    pub fn fetch(&mut self, memory: &SegmentedMemory) -> Option<RawInstruction> {
        // Ensure PC is 4-byte aligned for word addressing, though set_pc should ensure this.
        if self.pc % 4 != 0 {
            // This case should ideally not be hit if set_pc is used correctly.
            return None;
        }
        let word_address = self.pc / 4;

        match memory.read(word_address) {
            Ok(instr_val_i32) => {
                // Assuming instructions are stored big-endian in memory,
                // matching RawInstruction::from_bytes behavior if it assumes MSB first.
                // And RawInstruction::as_i32() also implies a consistent endianness.
                let bytes = instr_val_i32.to_be_bytes();
                let raw_instr = RawInstruction::from_bytes(bytes[0], bytes[1], bytes[2], bytes[3]);
                self.pc += 4;
                Some(raw_instr)
            }
            Err(_) => {
                // Could be out of bounds, a protection fault, or uninitialized memory.
                // For fetching, any error means we can't get an instruction.
                None
            }
        }
    }

    /// Sets the program counter to a new address.
    pub fn set_pc(&mut self, address: usize) {
        if address % 4 != 0 {
            panic!("Program counter must be aligned to 4 bytes. PC: {}", address);
        }
        // Bounds checking is effectively deferred to `fetch` when it tries to read from memory.
        self.pc = address;
    }
}

/// Error type for instruction decoding failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    InvalidOpcode(InvalidOpcodeError),
    // Potentially other decode errors in the future
}

impl From<InvalidOpcodeError> for DecodeError {
    fn from(err: InvalidOpcodeError) -> Self {
        DecodeError::InvalidOpcode(err)
    }
}

/// Decodes raw instructions into executable instructions.
///
/// The InstructionDecoder extracts the opcode and operands from
/// raw instruction data.
pub struct InstructionDecoder;

impl InstructionDecoder {
    /// Decodes a raw instruction into an executable instruction.
    pub fn decode(&self, instruction_raw: RawInstruction) -> Result<Instruction, DecodeError> {
        // RawInstruction::try_into() handles opcode validation and operand extraction.
        Instruction::try_from(instruction_raw).map_err(DecodeError::from)
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
        current_pc_of_instruction: usize,
    ) -> ExecutionSignal {
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
                ExecutionSignal::Continue
            }
            Opcode::Store => {
                if instruction.opcode.operand_count() != 1 {
                    panic!("Store instruction requires 1 operand");
                }

                let value = instruction.operands[0];

                // push the value to the stack as Integer
                stack.push(StackValue::Integer(value));
                ExecutionSignal::Continue
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
                ExecutionSignal::Continue
            }
            Opcode::LoadConstant => {
                if instruction.opcode.operand_count() != 1 {
                    panic!("LoadConstant instruction requires 1 operand");
                }

                let value = instruction.operands[0];
                // push the value to the stack as Integer
                stack.push(StackValue::Integer(value));
                ExecutionSignal::Continue
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
                ExecutionSignal::Continue
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
                ExecutionSignal::Continue
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
                ExecutionSignal::Continue
            }
            Opcode::StoreMemory => {
                if instruction.opcode.operand_count() != 1 {
                    panic!("StoreMemory instruction requires 1 operand");
                }

                let address = instruction.operands[0] as usize;
                let value = stack.pop().expect("Stack underflow");

                match value {
                    StackValue::Integer(i) => match memory.write(address, i) {
                        Ok(_) => ExecutionSignal::Continue,
                        Err(e) => panic!("Memory error: {}", e),
                    },
                    _ => panic!("Can only store integers in memory"),
                }
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
                        return ExecutionSignal::Jump(jump_address);
                    }
                } else {
                    panic!("Stack underflow in JumpIfZero");
                }
                ExecutionSignal::Continue
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

                ExecutionSignal::Continue
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
                    _ => panic!("Invalid region ID for LoadFromRegion: {}", region_id),
                };

                let value = memory.read_from_region(region_type, offset);
                match value {
                    Ok(value) => stack.push(StackValue::Integer(value)),
                    Err(e) => panic!("Memory error: {}", e),
                }
                ExecutionSignal::Continue
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
                    _ => panic!("Invalid region ID for StoreToRegion: {}", region_id),
                };

                match value {
                    StackValue::Integer(i) => {
                        match memory.write_to_region(region_type, offset, i) {
                            Ok(_) => ExecutionSignal::Continue,
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
                let return_address = current_pc_of_instruction + 4;

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
                ExecutionSignal::Jump(address)
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
                stack.values.clear();

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
                ExecutionSignal::Jump(return_address)
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

                    ExecutionSignal::Continue
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
                            ExecutionSignal::Continue
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
                ExecutionSignal::Continue
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
                ExecutionSignal::Continue
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
                ExecutionSignal::Continue
            }
            Opcode::Dup => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Dup instruction requires 0 operands");
                }

                let value = stack.peek().expect("Stack underflow");
                stack.push(*value);
                ExecutionSignal::Continue
            }
            Opcode::Swap => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Swap instruction requires 0 operands");
                }

                let a = stack.pop().expect("Stack underflow");
                let b = stack.pop().expect("Stack underflow");
                stack.push(a);
                stack.push(b);
                ExecutionSignal::Continue
            },
            Opcode::Drop => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Drop instruction requires 0 operands");
                }

                stack.pop();
                ExecutionSignal::Continue
            }, 
            Opcode::Alloc => {
                if instruction.opcode.operand_count() != 1 {
                    panic!("Alloc instruction requires 1 operand");
                }

                let size = instruction.operands[0] as usize;
                let pointer = memory.allocate(size);
                
                // If allocation succeeds, push the address as a reference
                // If it fails, push a null reference (0)
                match pointer {
                    Some(addr) => stack.push(StackValue::Reference(addr)),
                    None => stack.push(StackValue::Reference(0)), // Null pointer
                }
                
                ExecutionSignal::Continue
            }
            Opcode::Free => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("Free instruction requires 0 operands");
                }

                // Pop the address from the stack
                match stack.pop() {
                    Some(StackValue::Reference(addr)) => {
                        if addr == 0 {
                            // Null pointer, nothing to free
                            return ExecutionSignal::Continue;
                        }
                        
                        // Try to deallocate the memory
                        let success = memory.deallocate(addr);
                        
                        // Push success/failure indicator to the stack
                        stack.push(StackValue::Boolean(success));
                    }
                    Some(_) => panic!("Free: Expected a reference on the stack"),
                    None => panic!("Stack underflow in Free instruction"),
                }
                
                ExecutionSignal::Continue
            }
            Opcode::LoadHeap => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("LoadHeap instruction requires 0 operands");
                }

                // Pop offset and address from the stack
                let offset = match stack.pop() {
                    Some(StackValue::Integer(offset)) => offset as usize,
                    Some(_) => panic!("LoadHeap: Expected integer offset on the stack"),
                    None => panic!("Stack underflow in LoadHeap instruction"),
                };

                let addr = match stack.pop() {
                    Some(StackValue::Reference(addr)) => addr,
                    Some(_) => panic!("LoadHeap: Expected a reference on the stack"),
                    None => panic!("Stack underflow in LoadHeap instruction"),
                };

                if addr == 0 {
                    panic!("LoadHeap: Null pointer dereference");
                }

                if !memory.is_allocated(addr) {
                    panic!("LoadHeap: Invalid address, not allocated");
                }

                // Calculate the absolute address
                let absolute_addr = addr + offset;

                // Read from memory
                match memory.read(absolute_addr) {
                    Ok(value) => {
                        stack.push(StackValue::Integer(value));
                    },
                    Err(e) => {
                        panic!("LoadHeap: Memory error: {}", e);
                    }
                }

                ExecutionSignal::Continue
            }
            Opcode::StoreHeap => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("StoreHeap instruction requires 0 operands");
                }

                // Pop value, offset and address from the stack
                let value = match stack.pop() {
                    Some(StackValue::Integer(value)) => value,
                    Some(_) => panic!("StoreHeap: Expected integer value on the stack"),
                    None => panic!("Stack underflow in StoreHeap instruction"),
                };

                let offset = match stack.pop() {
                    Some(StackValue::Integer(offset)) => offset as usize,
                    Some(_) => panic!("StoreHeap: Expected integer offset on the stack"),
                    None => panic!("Stack underflow in StoreHeap instruction"),
                };

                let addr = match stack.pop() {
                    Some(StackValue::Reference(addr)) => addr,
                    Some(_) => panic!("StoreHeap: Expected a reference on the stack"),
                    None => panic!("Stack underflow in StoreHeap instruction"),
                };

                if addr == 0 {
                    panic!("StoreHeap: Null pointer dereference");
                }

                // Calculate the absolute address
                let absolute_addr = addr + offset;

                // Write to memory
                match memory.write(absolute_addr, value) {
                    Ok(_) => {},
                    Err(e) => {
                        panic!("StoreHeap: Memory error: {}", e);
                    }
                }

                ExecutionSignal::Continue
            }
            Opcode::MemSet => {
                if instruction.opcode.operand_count() > 0 {
                    panic!("MemSet instruction requires 0 operands");
                }

                // Pop value, count and address from the stack
                let value = match stack.pop() {
                    Some(StackValue::Integer(value)) => value,
                    Some(_) => panic!("MemSet: Expected integer value on the stack"),
                    None => panic!("Stack underflow in MemSet instruction"),
                };

                let count = match stack.pop() {
                    Some(StackValue::Integer(count)) => count as usize,
                    Some(_) => panic!("MemSet: Expected integer count on the stack"),
                    None => panic!("Stack underflow in MemSet instruction"),
                };

                let addr = match stack.pop() {
                    Some(StackValue::Reference(addr)) => addr,
                    Some(_) => panic!("MemSet: Expected a reference on the stack"),
                    None => panic!("Stack underflow in MemSet instruction"),
                };

                if addr == 0 {
                    panic!("MemSet: Null pointer dereference");
                }

                // Setze den Speicherbereich (in einer Schleife)
                for i in 0..count {
                    match memory.write(addr + i, value) {
                        Ok(_) => {}
                        Err(e) => {
                            panic!("MemSet: Memory error at offset {}: {}", i, e);
                        }
                    }
                }

                ExecutionSignal::Continue
            }
            // --- Platzhalter für neue Opcodes ---
            // Sie müssen diese Opcodes zu Ihrer Opcode-Enum hinzufügen (vermutlich in opcode.rs)
            Opcode::Yield => {
                // Signalisiert der VM-Hauptschleife, dass ein Kontextwechsel stattfinden soll.
                // Keine Operanden erwartet.
                if instruction.opcode.operand_count() > 0 {
                    panic!("Yield instruction requires 0 operands");
                }
                ExecutionSignal::Yield
            }
            Opcode::TerminateProcess => {
                // Signalisiert der VM-Hauptschleife, dass der aktuelle Prozess beendet werden soll.
                // Keine Operanden erwartet.
                if instruction.opcode.operand_count() > 0 {
                    panic!("TerminateProcess instruction requires 0 operands");
                }
                ExecutionSignal::Terminate
            }
            // Fügen Sie hier weitere Opcodes hinzu, falls erforderlich.
            // Der _-Arm ist für den Fall gedacht, dass die Opcode-Enum erweitert wird
            // und nicht alle neuen Opcodes hier sofort behandelt werden.
            // Wenn alle existierenden Opcodes oben abgedeckt sind, ist dieser Arm aktuell unerreichbar.
            _ => panic!("Unbekannter oder nicht implementierter Opcode: {:?}", instruction.opcode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        memory::SegmentedMemory,
        opcode::Opcode,
        stack::{Stack, StackValue},
    };

    #[test]
    fn test_instruction_fetcher() {
        // Create a simple program with correct byte ordering
        // This program will be loaded into memory, not the fetcher directly.
        let program_bytes = vec![
            0x00, 0x00, 0x00, 0x01, // Add (0x01)
            0x00, 0x00, 0x00, 0x02, // Store (0x02)
        ];

        let mut memory = SegmentedMemory::create_test_layout(100).expect("Failed to create memory");
        // Load program_bytes into memory at address 0
        for (i, chunk) in program_bytes.chunks(4).enumerate() {
            if chunk.len() == 4 {
                let val = i32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                memory.write(i, val).expect("Memory write failed");
            }
        }

        let mut fetcher = InstructionFetcher::new();

        // Fetch first instruction
        fetcher.set_pc(0);
        let instruction1_opt = fetcher.fetch(&memory); // instruction1_opt is Option<RawInstruction>
        assert!(instruction1_opt.is_some());
        let instruction1_raw = instruction1_opt.unwrap(); // instruction1_raw is RawInstruction
        assert_eq!(instruction1_raw.opcode().expect("Opcode decoding failed for instruction1"), Opcode::Add);
        assert_eq!(fetcher.pc, 4); // PC should advance

        // Fetch second instruction
        let instruction2_opt = fetcher.fetch(&memory); // instruction2_opt is Option<RawInstruction>
        assert!(instruction2_opt.is_some());
        let instruction2_raw = instruction2_opt.unwrap(); // instruction2_raw is RawInstruction
        assert_eq!(instruction2_raw.opcode().expect("Opcode decoding failed for instruction2"), Opcode::Store);
        assert_eq!(fetcher.pc, 8);

        // No more instructions
        let instruction3 = fetcher.fetch(&memory);
        assert!(instruction3.is_none());
    }

    #[test]
    fn test_instruction_decoder() {
        let decoder = InstructionDecoder;

        // Test decoding Add (no operands)
        let raw_add = RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x01);
        let decoded_add_result = decoder.decode(raw_add);
        assert!(decoded_add_result.is_ok());
        let instruction_add = decoded_add_result.unwrap();
        assert_eq!(instruction_add.opcode, Opcode::Add);
        assert!(instruction_add.operands.is_empty());

        // Test decoding Store (with operand)
        // First byte is the operand (big endian)
        let raw_store = RawInstruction::from_bytes(0x2A, 0x00, 0x00, 0x02);
        let decoded_store_result = decoder.decode(raw_store);
        assert!(decoded_store_result.is_ok());
        let instruction_store = decoded_store_result.unwrap();
        assert_eq!(instruction_store.opcode, Opcode::Store);
        // RawInstruction::get_operands() extracts the operand correctly based on opcode definition
        // For Store (operand_count = 1), it should extract 0x2A0000
        assert_eq!(instruction_store.operands, vec![0x2A0000]);

        // Test decoding an invalid opcode
        let raw_invalid = RawInstruction::from_bytes(0x00, 0x00, 0x00, 0xFE); // 0xFE is not a valid opcode
        let decoded_invalid_result = decoder.decode(raw_invalid);
        assert!(decoded_invalid_result.is_err());
        match decoded_invalid_result.err().unwrap() {
            DecodeError::InvalidOpcode(InvalidOpcodeError(val)) => assert_eq!(val, 0xFE),
            // _ => panic!("Expected InvalidOpcode error"), // Not needed if only one variant
        }
    }
    
    #[test]
    fn test_heap_operations() {
        // Set up the executor and memory
        let mut executor = InstructionExecutor::new();
        let mut memory = SegmentedMemory::create_test_layout(1000).unwrap();
        let mut stack = Stack::new(100);
        let current_pc = 0; // Dummy PC für Tests
        
        // Test Alloc instruction
        let alloc_instr = Instruction {
            opcode: Opcode::Alloc,
            operands: vec![10], // Allocate 10 words
        };
        
        let signal = executor.execute(&alloc_instr, &mut stack, &mut memory, current_pc);
        assert_eq!(signal, ExecutionSignal::Continue);
        
        // Check if we have a reference on the stack
        let addr = match stack.peek() {
            Some(StackValue::Reference(addr)) => *addr,
            _ => panic!("Expected a reference on the stack after allocation"),
        };
        
        assert!(addr > 0, "Allocation should return a non-zero address");
        
        // Test MemSet instruction - first set up the stack with required values
        // Stack needs: [reference, count, value]
        stack.pop(); // Remove the reference
        stack.push(StackValue::Reference(addr)); // Push it back
        stack.push(StackValue::Integer(10)); // Size of allocated memory
        stack.push(StackValue::Integer(42)); // Value to set
        
        let memset_instr = Instruction {
            opcode: Opcode::MemSet,
            operands: vec![],
        };
        
        let signal = executor.execute(&memset_instr, &mut stack, &mut memory, current_pc);
        assert_eq!(signal, ExecutionSignal::Continue);
        
        // The stack should be empty after MemSet
        assert_eq!(stack.len(), 0);
        
        // Test StoreHeap instruction - set up the stack with required values
        // Stack needs: [reference, offset, value]
        stack.push(StackValue::Reference(addr));
        stack.push(StackValue::Integer(5)); // Offset 5
        stack.push(StackValue::Integer(99)); // New value
        
        let store_heap_instr = Instruction {
            opcode: Opcode::StoreHeap,
            operands: vec![],
        };
        
        let signal = executor.execute(&store_heap_instr, &mut stack, &mut memory, current_pc);
        assert_eq!(signal, ExecutionSignal::Continue);
        
        // The stack should be empty after StoreHeap
        assert_eq!(stack.len(), 0);
        
        // Test LoadHeap instruction - first set up the stack with required values
        // Stack needs: [reference, offset]
        stack.push(StackValue::Reference(addr));
        stack.push(StackValue::Integer(5)); // Offset 5
        
        let load_heap_instr = Instruction {
            opcode: Opcode::LoadHeap,
            operands: vec![],
        };
        
        let signal = executor.execute(&load_heap_instr, &mut stack, &mut memory, current_pc);
        assert_eq!(signal, ExecutionSignal::Continue);
        
        // Check if we loaded the right value (99)
        let loaded_value = match stack.peek() {
            Some(StackValue::Integer(value)) => *value,
            _ => panic!("Expected an integer on the stack after LoadHeap"),
        };
        
        assert_eq!(loaded_value, 99, "Should have loaded value 99 from offset 5");
        
        // Test LoadHeap from another offset - should be 42 from MemSet
        stack.pop(); // Remove the loaded value
        stack.push(StackValue::Reference(addr));
        stack.push(StackValue::Integer(3)); // Different offset
        
        let signal = executor.execute(&load_heap_instr, &mut stack, &mut memory, current_pc);
        assert_eq!(signal, ExecutionSignal::Continue);
        
        // Check if we loaded the right value (42 from MemSet)
        let loaded_value = match stack.peek() {
            Some(StackValue::Integer(value)) => *value,
            _ => panic!("Expected an integer on the stack after LoadHeap"),
        };
        
        assert_eq!(loaded_value, 42, "Should have loaded value 42 from offset 3");
        
        // Test Free instruction - first set up the stack with the reference
        stack.clear();
        stack.push(StackValue::Reference(addr));
        
        let free_instr = Instruction {
            opcode: Opcode::Free,
            operands: vec![],
        };
        
        let signal = executor.execute(&free_instr, &mut stack, &mut memory, current_pc);
        assert_eq!(signal, ExecutionSignal::Continue);
        
        // Check if we got a success boolean
        let success = match stack.peek() {
            Some(StackValue::Boolean(success)) => *success,
            _ => panic!("Expected a boolean on the stack after Free"),
        };
        
        assert!(success, "Deallocation should succeed");
    }

    #[test]
    #[should_panic]
    fn test_invalid_heap_access() {
        let mut executor = InstructionExecutor::new();
        let mut memory = SegmentedMemory::create_test_layout(1000).unwrap();
        let mut stack = Stack::new(100);
        let current_pc = 0; // Dummy PC für Tests

        // Allocate some memory
        let addr = memory.allocate(10).unwrap();

        // Deallocate the memory
        let deallocated = memory.deallocate(addr);
        assert!(deallocated);

        // Test LoadHeap from an unallocated address
        stack.push(StackValue::Reference(addr)); // Use Reference type instead of Integer
        stack.push(StackValue::Integer(0)); // Offset
        
        let load_heap_instr = Instruction {
            opcode: Opcode::LoadHeap,
            operands: vec![],
        };

        executor.execute(&load_heap_instr, &mut stack, &mut memory, current_pc);
    }

    #[test]
    fn test_jump_if_zero_jumps() {
        let mut executor = InstructionExecutor::new();
        let mut stack = Stack::new(100);
        let mut memory = SegmentedMemory::create_test_layout(1000).unwrap(); // Erhöhte Speichergröße
        let current_pc = 0;
        let jump_target_address = 42 * 4; // Adresse muss durch 4 teilbar sein

        // Push 0 onto the stack to trigger the jump
        stack.push(StackValue::Integer(0));

        let jz_instr = Instruction {
            opcode: Opcode::JumpIfZero,
            operands: vec![jump_target_address as i32], // Operand ist die Sprungadresse
        };

        let signal = executor.execute(&jz_instr, &mut stack, &mut memory, current_pc);
        assert_eq!(signal, ExecutionSignal::Jump(jump_target_address));
        assert_eq!(stack.len(), 0); // Value should be popped
    }

    #[test]
    fn test_jump_if_zero_continues() {
        let mut executor = InstructionExecutor::new();
        let mut stack = Stack::new(100);
        let mut memory = SegmentedMemory::create_test_layout(1000).unwrap(); // Erhöhte Speichergröße
        let current_pc = 0;
        let jump_target_address = 42 * 4;

        // Push a non-zero value onto the stack
        stack.push(StackValue::Integer(10));

        let jz_instr = Instruction {
            opcode: Opcode::JumpIfZero,
            operands: vec![jump_target_address as i32], // Operand ist die Sprungadresse
        };

        let signal = executor.execute(&jz_instr, &mut stack, &mut memory, current_pc);
        assert_eq!(signal, ExecutionSignal::Continue);
        assert_eq!(stack.len(), 0); // Value should be popped
    }
}
