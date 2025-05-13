use crate::opcode::Opcode;
// Import the derive macro
#[cfg(feature = "derive")]
pub use intoraw_derive::IntoRaw;

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
        if self.has_operands() {
            operands.push(self.operand());
        }
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

/// This trait allows instruction types to be converted to RawInstruction
pub trait IntoRaw {
    fn into_raw(self) -> RawInstruction;
}

// Define instruction types with manual implementations
pub struct Add;

impl IntoRaw for Add {
    fn into_raw(self) -> RawInstruction {
        RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x01)
    }
}

pub struct Store(pub i32);

impl IntoRaw for Store {
    fn into_raw(self) -> RawInstruction {
        // Extract the operand from the first field
        let operand = self.0;
        let b0 = ((operand >> 16) & 0xFF) as u8;
        let b1 = ((operand >> 8) & 0xFF) as u8;
        let b2 = (operand & 0xFF) as u8;
        
        // Create instruction with the appropriate opcode
        RawInstruction::from_bytes(b0, b1, b2, 0x02)
    }
}

pub struct Sub;

impl IntoRaw for Sub {
    fn into_raw(self) -> RawInstruction {
        RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x03)
    }
}

pub struct LoadConstant(pub i32);

impl IntoRaw for LoadConstant {
    fn into_raw(self) -> RawInstruction {
        let operand = self.0;
        let b0 = ((operand >> 16) & 0xFF) as u8;
        let b1 = ((operand >> 8) & 0xFF) as u8;
        let b2 = (operand & 0xFF) as u8;
        
        RawInstruction::from_bytes(b0, b1, b2, 0x04)
    }
}

pub struct Multiply;

impl IntoRaw for Multiply {
    fn into_raw(self) -> RawInstruction {
        RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x05)
    }
}

pub struct Print;

impl IntoRaw for Print {
    fn into_raw(self) -> RawInstruction {
        RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x06)
    }
}

pub struct LoadMemory(pub i32);

impl IntoRaw for LoadMemory {
    fn into_raw(self) -> RawInstruction {
        let operand = self.0;
        let b0 = ((operand >> 16) & 0xFF) as u8;
        let b1 = ((operand >> 8) & 0xFF) as u8;
        let b2 = (operand & 0xFF) as u8;
        
        RawInstruction::from_bytes(b0, b1, b2, 0x07)
    }
}

pub struct StoreMemory(pub i32);

impl IntoRaw for StoreMemory {
    fn into_raw(self) -> RawInstruction {
        let operand = self.0;
        let b0 = ((operand >> 16) & 0xFF) as u8;
        let b1 = ((operand >> 8) & 0xFF) as u8;
        let b2 = (operand & 0xFF) as u8;
        
        RawInstruction::from_bytes(b0, b1, b2, 0x08)
    }
}

pub struct JumpIfZero(pub i32);

impl IntoRaw for JumpIfZero {
    fn into_raw(self) -> RawInstruction {
        let operand = self.0;
        let b0 = ((operand >> 16) & 0xFF) as u8;
        let b1 = ((operand >> 8) & 0xFF) as u8;
        let b2 = (operand & 0xFF) as u8;
        
        RawInstruction::from_bytes(b0, b1, b2, 0x09)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_into_raw() {
        // Test Add instruction (no operands)
        let add = Add;
        let raw = add.into_raw();
        assert_eq!(raw.opcode(), Opcode::Add);
        assert!(!raw.has_operands());
        
        // Test Store instruction with operand
        let store = Store(42);
        let raw = store.into_raw();
        assert_eq!(raw.opcode(), Opcode::Store);
        assert_eq!(raw.operand(), 42);
        assert!(raw.has_operands());
    }
}

// Example of using the derive macro when feature is enabled
#[cfg(feature = "derive")]
mod derive_tests {
    use super::*;
    
    // Types using the derive macro
    #[derive(IntoRaw)]
    #[opcode = "1"] // Must be literal values, not identifiers
    struct CustomAdd;
    
    #[derive(IntoRaw)]
    #[opcode = "2"]
    struct CustomStore(i32);
    
    // Example for custom opcode specification with attribute
    #[derive(IntoRaw)]
    #[opcode = "3"] // Sub opcode
    struct CustomSub;
    
    #[derive(IntoRaw)]
    #[opcode = "4"] // LoadConstant opcode with operand
    struct CustomLoadConstant(i32);
    
    #[derive(IntoRaw)]
    #[opcode = "5"] // Multiply opcode
    struct CustomMultiply;
    
    #[derive(IntoRaw)]
    #[opcode = "6"] // Print opcode
    struct CustomPrint;
    
    #[derive(IntoRaw)]
    #[opcode = "7"] // LoadMemory opcode with operand
    struct CustomLoadMemory(i32);
    
    #[derive(IntoRaw)]
    #[opcode = "8"] // StoreMemory opcode with operand
    struct CustomStoreMemory(i32);
    
    #[test]
    fn test_derive_macro() {
        // Test derived Add instruction
        let add = CustomAdd;
        let raw = add.into_raw();
        assert_eq!(raw.opcode(), Opcode::Add);
        
        // Test derived Store instruction
        let store = CustomStore(123);
        let raw = store.into_raw();
        assert_eq!(raw.opcode(), Opcode::Store);
        assert_eq!(raw.operand(), 123);
        
        // Test custom opcodes with attributes
        let sub = CustomSub;
        let raw = sub.into_raw();
        assert_eq!(raw.as_i32() & 0xFF, 3); // Check raw opcode
        
        let load = CustomLoadConstant(42);
        let raw = load.into_raw();
        assert_eq!(raw.as_i32() & 0xFF, 4); // Check raw opcode
        assert_eq!(raw.operand(), 42);      // Check operand
        
        // Test new custom opcodes
        let multiply = CustomMultiply;
        let raw = multiply.into_raw();
        assert_eq!(raw.as_i32() & 0xFF, 5);
        
        let print = CustomPrint;
        let raw = print.into_raw();
        assert_eq!(raw.as_i32() & 0xFF, 6);
        
        let load_mem = CustomLoadMemory(100);
        let raw = load_mem.into_raw();
        assert_eq!(raw.as_i32() & 0xFF, 7);
        assert_eq!(raw.operand(), 100);
        
        let store_mem = CustomStoreMemory(200);
        let raw = store_mem.into_raw();
        assert_eq!(raw.as_i32() & 0xFF, 8);
        assert_eq!(raw.operand(), 200);
    }
}


