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
///
/// There are two ways to implement this trait:
/// 1. Manually implement for custom instruction types
/// 2. Use the `#[derive(IntoRaw)]` attribute when the `derive` feature is enabled
///
/// When using the derive attribute, you can specify the opcode in several ways:
/// - Using a literal value: `#[opcode = 1]` or `#[opcode = 0x01]`
/// - Using a string literal: `#[opcode = "1"]`
///
/// The numeric values should correspond to the opcodes defined in the `Opcode` enum:
/// - 1 (0x01): Add
/// - 2 (0x02): Store
/// - 3 (0x03): Sub
/// - 4 (0x04): LoadConstant
/// - 5 (0x05): Multiply
/// - 6 (0x06): Print
/// - 7 (0x07): LoadMemory
/// - 8 (0x08): StoreMemory
/// - 9 (0x09): JumpIfZero
///
/// If no opcode is specified, the macro will try to infer it from the struct name.
/// The macro uses sophisticated name normalization to work with various naming conventions:
/// 
/// - Standard names: `Add`, `Store`, etc.
/// - With prefixes: `CustomAdd`, `DirectStore`, `MyAdd`, etc.
/// - With suffixes: `AddInstruction`, etc.
/// - Mixed case: `addInstruction`, `SUB_INSTRUCTION`, etc.
/// - Compound names: `MyCustomStoreInstruction`, etc.
///
/// For instructions with operands, create a tuple struct with a single field (e.g., `struct Store(i32)`).
/// For instructions without operands, create a unit struct (e.g., `struct Add;`).
pub trait IntoRaw {
    fn into_raw(self) -> RawInstruction;
}

// Define instruction types with manual implementations
#[derive(IntoRaw)]
pub struct Add;


#[derive(IntoRaw)]
#[opcode = 0x02]
pub struct Store(pub i32);


#[derive(IntoRaw)]
#[opcode = 0x03]    
pub struct Sub;

#[derive(IntoRaw)]
#[opcode = 0x04]
pub struct LoadConstant(pub i32);

#[derive(IntoRaw)]
#[opcode = 0x05]
pub struct Multiply;

#[derive(IntoRaw)]
#[opcode = 0x06]
pub struct Print;

#[derive(IntoRaw)]
#[opcode = 0x07]
pub struct LoadMemory(pub i32);

#[derive(IntoRaw)]
#[opcode = 0x08]
pub struct StoreMemory(pub i32);

#[derive(IntoRaw)]
#[opcode = 0x09]
pub struct JumpIfZero(pub i32);

#[derive(IntoRaw)]
#[opcode = 0x0A]
pub struct Divide;

// Neue Instruktionstypen für Speicherregionszugriff
#[derive(IntoRaw)]
#[opcode = 0x0B]
pub struct LoadFromRegion(pub i32, pub i32); // Region-ID, Offset

#[derive(IntoRaw)]
#[opcode = 0x0C]
pub struct StoreToRegion(pub i32, pub i32); // Region-ID, Offset

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
    #![allow(unused_imports)]
    #![allow(dead_code)]

    use super::*;
    use crate::opcode::Opcode;
    
    // Using opcodes by their numeric values
    #[derive(IntoRaw)]
    #[opcode = 0x01]
    struct DirectAdd;
    
    #[derive(IntoRaw)]
    #[opcode = 0x02]
    struct DirectStore(i32);
    
    #[derive(IntoRaw)]
    #[opcode = 0x03]
    struct DirectSub;
    
    #[derive(IntoRaw)]
    #[opcode = 0x04] // Use literal value instead of Opcode::LoadConstant
    struct DirectLoadConstant(i32);
    
    // Existing tests with string values
    #[derive(IntoRaw)]
    #[opcode = "1"] // Must be literal values, not identifiers
    struct CustomAdd;
    
    #[derive(IntoRaw)]
    #[opcode = "2"]
    struct CustomStore(i32);
    
    // Example for custom opcode specification with attribute
    #[derive(IntoRaw)]
    #[opcode = 3] // Sub opcode
    struct CustomSub;
    
    #[derive(IntoRaw)]
    #[opcode = 4] // LoadConstant opcode with operand
    struct CustomLoadConstant(i32);
    
    #[derive(IntoRaw)]
    #[opcode = 5] // Multiply opcode
    struct CustomMultiply;
    
    #[derive(IntoRaw)]
    #[opcode = 6] // Print opcode
    struct CustomPrint;
    
    #[derive(IntoRaw)]
    #[opcode = 7] // LoadMemory opcode with operand
    struct CustomLoadMemory(i32);
    
    #[derive(IntoRaw)]
    #[opcode = 8] // StoreMemory opcode with operand
    struct CustomStoreMemory(i32);
    
    #[test]
    fn test_direct_opcode_paths() {
        // Test DirectAdd
        let add = DirectAdd;
        let raw = add.into_raw();
        assert_eq!(raw.opcode(), Opcode::Add);
        
        // Test DirectStore
        let store = DirectStore(123);
        let raw = store.into_raw();
        assert_eq!(raw.opcode(), Opcode::Store);
        assert_eq!(raw.operand(), 123);
        
        // Test DirectSub
        let sub = DirectSub;
        let raw = sub.into_raw();
        assert_eq!(raw.opcode(), Opcode::Sub);
        
        // Test DirectLoadConstant
        let load = DirectLoadConstant(42);
        let raw = load.into_raw();
        assert_eq!(raw.opcode(), Opcode::LoadConstant);
        assert_eq!(raw.operand(), 42);
    }
    
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

    // Test flexible naming conventions
    #[derive(IntoRaw)]
    struct AddInstruction;  // No explicit opcode, will infer from name
    
    #[derive(IntoRaw)]
    struct MyCustomStoreInstruction(i32);  // Uses normalized name "Store"
    
    // Test various advanced name patterns
    #[test]
    fn test_flexible_naming() {
        // Test AddInstruction (name normalization)
        let add = AddInstruction;
        let raw = add.into_raw();
        assert_eq!(raw.opcode(), Opcode::Add);
        assert!(!raw.has_operands());
        
        // Test MyCustomStoreInstruction (with custom prefix and suffix)
        let store = MyCustomStoreInstruction(42);
        let raw = store.into_raw();
        assert_eq!(raw.opcode(), Opcode::Store);
        assert_eq!(raw.operand(), 42);
        assert!(raw.has_operands());
    }
}


