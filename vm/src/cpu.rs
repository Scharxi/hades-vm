/// CPU implementation for the Hades VM.
///
/// This module contains the central processing unit (CPU) implementation,
/// which is responsible for the fetch-decode-execute cycle.

use crate::{
    memory::{MemoryRegionType, SegmentedMemory},
    instruction_processor::{InstructionFetcher, InstructionDecoder, InstructionExecutor, ProcessControlBlock, ProcessState},
    stack::Stack,
    instruction::RawInstruction,
    opcode::Opcode,
    instruction_processor::ExecutionSignal,
};

/// The central processing unit of the virtual machine.
///
/// The CPU is responsible for executing the instruction cycle:
/// 1. Fetch instructions from memory (via InstructionFetcher)
/// 2. Decode instructions (via InstructionDecoder)
/// 3. Execute instructions (via InstructionExecutor)
///
/// It also manages the stack and memory access for the currently running process.
pub struct CPU {
    /// Fetches raw instructions from the program
    pub fetcher: InstructionFetcher, 
    /// Decodes raw instructions into executable instructions
    pub decoder: InstructionDecoder, 
    /// Executes decoded instructions
    pub executor: InstructionExecutor,
    /// The stack for the *currently running* process.
    pub stack: Stack, 
    /// The memory system with segmented regions (shared among processes for now)
    pub memory: SegmentedMemory, 
    /// List of all process control blocks.
    pub processes: Vec<ProcessControlBlock>,
    /// Index of the currently running process in the `processes` Vec.
    pub current_process_idx: Option<usize>,
    /// Counter for assigning unique PIDs.
    next_pid: u32,
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
            processes: Vec::new(),
            current_process_idx: None,
            next_pid: 0,
        }
    }

    /// Creates a new process and adds it to the process list.
    /// The program for the process should already be loaded into memory.
    /// The initial PC points to the start of its code in memory (e.g., Code region start for the first process).
    fn create_process(&mut self, initial_pc: usize, initial_stack_size: usize) -> u32 {
        let pid = self.next_pid;
        self.next_pid += 1;

        let pcb = ProcessControlBlock {
            pid,
            pc: initial_pc,
            stack: Stack::new(initial_stack_size),
            state: ProcessState::Ready,
        };
        self.processes.push(pcb);
        pid
    }

    /// Loads a program into memory, creates the initial process, and sets it as running.
    pub fn load_program(&mut self, program: &[u8]) {
        self.load_program_to_memory(program);
        
        // Ersten Prozess erstellen
        let initial_pc = 0; 
        let main_process_pid = self.create_process(initial_pc, 256);

        // Den ersten Prozess als laufend setzen
        if let Some(first_process_pcb) = self.processes.iter_mut().find(|p| p.pid == main_process_pid) {
            first_process_pcb.state = ProcessState::Running;
            self.fetcher.set_pc(first_process_pcb.pc);
            self.stack = first_process_pcb.stack.clone();
            self.current_process_idx = self.processes.iter().position(|p| p.pid == main_process_pid);
        } else {
            panic!("Konnte den initialen Prozess nicht als laufend setzen.");
        }
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

    /// Executes a single instruction cycle (fetch, decode, execute) for the current process.
    ///
    /// Returns true if an instruction was executed or a context switch happened,
    /// false if there are no more runnable processes.
    pub fn step(&mut self) -> bool {
        let current_idx = match self.current_process_idx {
            Some(idx) => idx,
            None => return false, // Kein laufender Prozess, VM stoppt
        };

        // Sicherstellen, dass der Index gültig ist und der Prozess läuft oder bereit ist
        if current_idx >= self.processes.len() || self.processes[current_idx].state == ProcessState::Terminated {
            // Versuchen, einen neuen Prozess zu schedulen, wenn der aktuelle ungültig/terminiert ist
            return self.schedule_and_switch_context(None); 
        }
        
        // PC und Stack für den aktuellen Prozess aus PCB in die CPU-Komponenten laden
        // (Stack wird beim Kontextwechsel geladen, PC hier für den fetcher)
        self.fetcher.set_pc(self.processes[current_idx].pc);

        if let Some(instruction_raw) = self.fetcher.fetch(&self.memory) {
            match self.decoder.decode(instruction_raw) {
                Ok(instruction) => {
                    let pc_of_current_instruction = self.fetcher.pc.saturating_sub(4); // PC after fetch

                    let execution_signal = self.executor.execute(
                        &instruction,
                        &mut self.stack, 
                        &mut self.memory,
                        pc_of_current_instruction,
                    );

                    self.processes[current_idx].pc = self.fetcher.pc;

                    match execution_signal {
                        ExecutionSignal::Continue => {}
                        ExecutionSignal::Jump(address) => {
                            if instruction.opcode == Opcode::Call { // Check opcode on successful decode
                                if let Some(frame_idx) = self.stack.current_frame {
                                    if frame_idx < self.stack.frames.len() {
                                        self.stack.frames[frame_idx].return_address = self.processes[current_idx].pc;
                                    }
                                }
                            }
                            self.processes[current_idx].pc = address;
                            self.fetcher.set_pc(address);
                        }
                        ExecutionSignal::Yield => {
                            self.processes[current_idx].state = ProcessState::Ready;
                            self.processes[current_idx].stack = self.stack.clone();
                            return self.schedule_and_switch_context(Some(current_idx));
                        }
                        ExecutionSignal::Terminate => {
                            self.processes[current_idx].state = ProcessState::Terminated;
                            self.processes[current_idx].stack = self.stack.clone(); 
                            return self.schedule_and_switch_context(Some(current_idx)); 
                        }
                    }
                    true // An instruction was executed or signal handled
                }
                Err(decode_error) => {
                    // Handle decode error, e.g., print error and terminate process
                    // Use a more specific error log if DecodeError contains more info
                    eprintln!(
                        "PID {}: Failed to decode instruction at PC {}. Error: {:?}. Terminating process.",
                        self.processes[current_idx].pid,
                        self.processes[current_idx].pc, // PC before fetch attempt
                        decode_error
                    );
                    self.processes[current_idx].state = ProcessState::Terminated;
                    self.processes[current_idx].stack = self.stack.clone(); // Save stack state
                    self.schedule_and_switch_context(Some(current_idx)) // Attempt to switch context
                }
            }
        } else {
            // No instruction fetched (e.g., end of program memory for this process's PC)
            self.processes[current_idx].state = ProcessState::Terminated;
            self.processes[current_idx].stack = self.stack.clone(); 
            self.schedule_and_switch_context(Some(current_idx))
        }
    }

    /// Wählt den nächsten `Ready`-Prozess aus (Round-Robin) und wechselt den Kontext.
    /// Gibt `true` zurück, wenn ein neuer Prozess gestartet wurde, sonst `false`.
    fn schedule_and_switch_context(&mut self, previous_process_idx_opt: Option<usize>) -> bool {
        let mut next_pid_to_run: Option<u32> = None;
        let mut search_offset = 0;

        if let Some(previous_idx) = previous_process_idx_opt {
            // Starte die Suche nach dem vorherigen Prozess für Round Robin
            search_offset = (previous_idx + 1) % self.processes.len();
        }

        for i in 0..self.processes.len() {
            let current_search_idx = (search_offset + i) % self.processes.len();
            if self.processes[current_search_idx].state == ProcessState::Ready {
                next_pid_to_run = Some(self.processes[current_search_idx].pid);
                break;
            }
        }

        if let Some(pid_to_run) = next_pid_to_run {
            if let Some(new_process_idx) = self.processes.iter().position(|p| p.pid == pid_to_run) {
                self.processes[new_process_idx].state = ProcessState::Running;
                self.stack = self.processes[new_process_idx].stack.clone(); // Stack laden
                self.fetcher.set_pc(self.processes[new_process_idx].pc);    // PC laden
                self.current_process_idx = Some(new_process_idx);
                return true;
            }
        }
        
        // Kein weiterer Ready-Prozess gefunden
        self.current_process_idx = None;
        false
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
        // Mit Prozessmanagement kann step() false zurückgeben, wenn kein Prozess mehr lauffähig ist.
        // Der Test muss ggf. angepasst werden, wenn Terminate implementiert ist.
        // Für einen einzelnen Prozess ohne Yield/Terminate sollte es nach Programmende false sein.
        while cpu.step() {} // Lasse die CPU laufen, bis keine Schritte mehr möglich sind
        assert_eq!(cpu.current_process_idx, None); // Kein Prozess sollte mehr laufen
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
        // Programmende, nächster Step sollte false liefern oder zu anderem Prozess wechseln
        // In diesem Testfall (Einzelprozess) erwarten wir false, nachdem der Prozess terminiert.
        assert!(!cpu.step()); 
        assert_eq!(cpu.processes[cpu.current_process_idx.unwrap_or(0)].state, ProcessState::Terminated);
    }

    // TODO: Fügen Sie hier Tests für kooperatives Multitasking hinzu.
    // Z.B. zwei Prozesse, die `Yield` verwenden und abwechselnd Werte auf den Stack legen oder ausgeben.
    
    #[test]
    fn test_cooperative_multitasking_two_processes() {
        let mut cpu = CPU::new(1000);

        // Programm 1: Zählt von 1 hoch und yielded
        // Opcodes: LoadConstant(1)=0x04, Add=0x01, Yield=0xF0 (Annahme), JumpIfZero (zum Loop)=0x0A (Opcode für Jump angenommmen)
        // Real müssten Sie Sprünge für Loops implementieren oder die Logik anders gestalten.
        // Für diesen Test vereinfacht: LoadC 1, Yield, LoadC 2, Yield, LoadC 3, Terminate
        let program1 = vec![
            0x00, 0x00, 0x01, Opcode::LoadConstant as u8, // Load 1
            0x00, 0x00, 0x00, Opcode::Yield as u8,        // Yield
            0x00, 0x00, 0x02, Opcode::LoadConstant as u8, // Load 2
            0x00, 0x00, 0x00, Opcode::Yield as u8,        // Yield
            0x00, 0x00, 0x03, Opcode::LoadConstant as u8, // Load 3
            0x00, 0x00, 0x00, Opcode::TerminateProcess as u8, // Terminate
        ];

        // Programm 2: Zählt von 10 hoch und yielded
        let program2 = vec![
            0x00, 0x00, 0x0A, Opcode::LoadConstant as u8, // Load 10 (0x0A)
            0x00, 0x00, 0x00, Opcode::Yield as u8,        // Yield
            0x00, 0x00, 0x0B, Opcode::LoadConstant as u8, // Load 11 (0x0B)
            0x00, 0x00, 0x00, Opcode::Yield as u8,        // Yield
            0x00, 0x00, 0x0C, Opcode::LoadConstant as u8, // Load 12 (0x0C)
            0x00, 0x00, 0x00, Opcode::TerminateProcess as u8, // Terminate
        ];

        // Wichtig: Die VM lädt Programme in einen globalen Speicher.
        // Hier müssen wir sicherstellen, dass sich die Programme nicht überschreiben
        // oder die CPU muss wissen, wo jedes Programm im Speicher liegt.
        // Für diesen Test laden wir nur Programm 1, um den ersten Prozess zu erstellen.
        // Dann erstellen wir manuell einen zweiten Prozess, der später "geladen" wird.
        // Eine bessere VM würde das Laden mehrerer Programme in getrennte Bereiche unterstützen.

        cpu.load_program(&program1);
        let p1_idx = cpu.current_process_idx.unwrap();
        assert_eq!(cpu.processes[p1_idx].pid, 0);

        // Manuellen zweiten Prozess erstellen, der program2 ausführt.
        // Wir müssen den Code von program2 in den Speicher schreiben an einer anderen Stelle.
        // Angenommen, program1 belegt pc 0 bis (program1.len() - 1)
        // Wir laden program2 an pc, z.B. 100 (muss groß genug sein)
        let program2_start_pc = 100 * 4; // In Bytes, jede Instruktion 4 Bytes
        for (offset, chunk) in program2.chunks(4).enumerate() {
            if chunk.len() == 4 {
                let raw_instr = RawInstruction::from_bytes(chunk[0], chunk[1], chunk[2], chunk[3]);
                // Direkter Schreibzugriff in den Speicher - gefährlich ohne Regionenmanagement, aber für Test ok
                cpu.memory.write(program2_start_pc / 4 + offset, raw_instr.as_i32()).unwrap();
            }
        }
        let _p2_pid = cpu.create_process(program2_start_pc, 256);

        // Ausführungsschritte und Überprüfungen:
        // P1: Load 1
        assert!(cpu.step()); 
        assert_eq!(cpu.processes[p1_idx].pc, 4); // Nach LoadConstant
        assert_eq!(cpu.stack.peek().unwrap(), &StackValue::Integer(1));
        let p1_original_stack_len = cpu.stack.len();

        // P1: Yield -> Wechsel zu P2 (angenommen P2 ist der nächste Ready Prozess)
        assert!(cpu.step()); 
        let p2_idx = cpu.current_process_idx.unwrap();
        assert_ne!(p1_idx, p2_idx); // Sicherstellen, dass Prozess gewechselt hat
        assert_eq!(cpu.processes[p2_idx].pc, program2_start_pc); // P2 startet bei seinem PC
        assert_eq!(cpu.processes[p1_idx].state, ProcessState::Ready); // P1 ist jetzt Ready
        assert_eq!(cpu.processes[p1_idx].stack.len(), p1_original_stack_len); // Stack von P1 wurde gesichert

        // P2: Load 10
        assert!(cpu.step());
        assert_eq!(cpu.processes[p2_idx].pc, program2_start_pc + 4);
        assert_eq!(cpu.stack.peek().unwrap(), &StackValue::Integer(10));
        let p2_original_stack_len = cpu.stack.len();

        // P2: Yield -> Wechsel zu P1
        assert!(cpu.step());
        assert_eq!(cpu.current_process_idx.unwrap(), p1_idx); // Zurück zu P1
        assert_eq!(cpu.processes[p1_idx].pc, 4); // P1 setzt bei PC nach Yield fort (also vor Load 2)
        assert_eq!(cpu.processes[p2_idx].state, ProcessState::Ready); // P2 ist jetzt Ready
        assert_eq!(cpu.processes[p2_idx].stack.len(), p2_original_stack_len);

        // P1: Load 2
        assert!(cpu.step());
        assert_eq!(cpu.processes[p1_idx].pc, 4 + 4);
        assert_eq!(cpu.stack.peek().unwrap(), &StackValue::Integer(2));

        // P1: Yield -> Wechsel zu P2
        assert!(cpu.step());
        assert_eq!(cpu.current_process_idx.unwrap(), p2_idx);
        assert_eq!(cpu.processes[p2_idx].pc, program2_start_pc + 4); // P2 setzt fort

        // P2: Load 11
        assert!(cpu.step());
        assert_eq!(cpu.stack.peek().unwrap(), &StackValue::Integer(11));

        // P2: Yield -> Wechsel zu P1
        assert!(cpu.step());
        assert_eq!(cpu.current_process_idx.unwrap(), p1_idx);

        // P1: Load 3
        assert!(cpu.step());
        assert_eq!(cpu.stack.peek().unwrap(), &StackValue::Integer(3));

        // P1: Terminate -> Wechsel zu P2 (P1 wird Terminated)
        assert!(cpu.step());
        assert_eq!(cpu.current_process_idx.unwrap(), p2_idx);
        assert_eq!(cpu.processes[p1_idx].state, ProcessState::Terminated);

        // P2: Load 12
        assert!(cpu.step());
        assert_eq!(cpu.stack.peek().unwrap(), &StackValue::Integer(12));

        // P2: Terminate -> Kein Prozess mehr Ready
        assert!(cpu.step()); 
        assert_eq!(cpu.processes[p2_idx].state, ProcessState::Terminated);
        
        // Kein lauffähiger Prozess mehr übrig
        assert!(!cpu.step());
        assert_eq!(cpu.current_process_idx, None);
    }
} 