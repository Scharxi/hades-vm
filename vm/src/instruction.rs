use crate::opcode::Opcode;


#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: Opcode, 
    pub operands: Vec<i32>, 
}

/// RawInstruction is a wrapper around an i32 that represents a raw bytecode instruction
/// It provides convenience methods for working with instructions in their raw form
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawInstruction(i32);

impl RawInstruction {
    pub fn new(instruction: i32) -> Self {
        Self(instruction)
    }

    pub fn from_bytes(b0: u8, b1: u8, b2: u8, b3: u8) -> Self {
        let value = ((b0 as i32) << 24) | ((b1 as i32) << 16) | ((b2 as i32) << 8) | (b3 as i32);
        Self(value)
    }

    pub fn opcode(&self) -> Opcode {
        Opcode::from(self.0 & 0xff)
    }

    pub fn operand(&self) -> i32 {
        (self.0 >> 8) & 0xffffff
    }

    pub fn get_operands(&self) -> Vec<i32> {
        let mut operands = Vec::new();
        let mut value = self.0 >> 8;
        for _ in 0..self.operand_count() {
            operands.push(value & 0xffffff);
            value >>= 24;
        }
        operands.reverse();
        operands
    }

    pub fn as_i32(&self) -> i32 {
        self.0
    }

    pub fn has_operands(&self) -> bool {
        self.opcode().operand_count() > 0
    }

    pub fn operand_count(&self) -> usize {
        self.opcode().operand_count()
    }

}

impl From<RawInstruction> for Instruction {
    fn from(raw: RawInstruction) -> Self {
        let opcode = raw.opcode();
        let operands = raw.get_operands();
        Self { opcode, operands }
    }
}


pub trait IntoRaw {
    fn into_raw(self) -> RawInstruction;
}



mod instructions {
    use super::{IntoRaw, RawInstruction};

    pub struct Store(pub i32);

    impl IntoRaw for Store {
        fn into_raw(self) -> RawInstruction {
            // Extract individual bytes from the operand value
            let operand = self.0;
            let b0 = ((operand >> 16) & 0xFF) as u8;
            let b1 = ((operand >> 8) & 0xFF) as u8;
            let b2 = (operand & 0xFF) as u8;
            
            // Create instruction with opcode 0x02 (Store)
            RawInstruction::from_bytes(b0, b1, b2, 0x02)
        }
    }

    pub struct Add; 

    impl IntoRaw for Add {
        fn into_raw(self) -> RawInstruction {
            RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x01)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::instructions::Store;

    use super::{instructions::Add, *};

    #[test]
    fn test_raw_instruction() {
        // Create a Store instruction (0x02) with operand 0x010203
        let raw = RawInstruction::from_bytes(0x01, 0x02, 0x03, 0x02);
        assert_eq!(raw.opcode(), Opcode::Store);
        assert_eq!(raw.operand(), 0x010203);
        assert!(raw.has_operands());
    }

    #[test]
    fn test_raw_instruction_into_instruction() {
        // Create a Store instruction (0x02) with operand 0x010203
        let raw = RawInstruction::from_bytes(0x01, 0x02, 0x03, 0x02);
        let instruction: Instruction = raw.into();
        assert_eq!(instruction.opcode, Opcode::Store);
        assert_eq!(instruction.operands, vec![0x010203]);
    }

    #[test]
    fn test_instructions_into_raw_instruction() {
        let instruction: Store = Store(0x010203);
        let raw: RawInstruction = instruction.into_raw();
        assert_eq!(raw.opcode(), Opcode::Store);
        assert_eq!(raw.operand(), 0x010203); 
    }

    #[test]
    fn test_instructions_into_raw_instruction_add() {
        let instruction: Add = Add;
        let raw: RawInstruction = instruction.into_raw();
        assert_eq!(raw.opcode(), Opcode::Add);
        assert_eq!(raw.operand(), 0x000000);
    }
}


