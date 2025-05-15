use crate::opcode::Opcode;
// Import the derive macro
#[cfg(feature = "derive")]
pub use intoraw_derive::IntoRaw;

use crate::opcode::InvalidOpcodeError; // Import the error type

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

    pub fn opcode(&self) -> Result<Opcode, InvalidOpcodeError> {
        Opcode::try_from((self.0 & 0xff) as u8)
    }

    pub fn operand(&self) -> i32 {
        (self.0 >> 8) & 0xffffff
    }

    pub fn get_operands(&self) -> Result<Vec<i32>, InvalidOpcodeError> {
        let mut operands = Vec::new();
        let operand_count = self.operand_count()?;
        if operand_count > 0 {
            let raw_operand = self.operand();
            if operand_count == 1 {
                operands.push(raw_operand);
            } else if operand_count == 2 {
                // For two operands, split the 24 bits into two 12-bit fields
                operands.push((raw_operand >> 12) & 0xfff); // First 12 bits
                operands.push(raw_operand & 0xfff); // Last 12 bits
            }
        }
        Ok(operands)
    }

    pub fn as_i32(&self) -> i32 {
        self.0
    }

    pub fn has_operands(&self) -> Result<bool, InvalidOpcodeError> {
        Ok(self.opcode()?.operand_count() > 0)
    }

    pub fn operand_count(&self) -> Result<usize, InvalidOpcodeError> {
        Ok(self.opcode()?.operand_count())
    }
}

impl TryFrom<RawInstruction> for Instruction {
    type Error = InvalidOpcodeError;

    fn try_from(raw: RawInstruction) -> Result<Self, Self::Error> {
        let opcode = raw.opcode()?;
        let operands = raw.get_operands()?;
        Ok(Self { opcode, operands })
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

#[derive(IntoRaw)]
#[opcode = 0x11] 
pub struct Modulo;

#[derive(IntoRaw)]
#[opcode = 0x12]
pub struct Power;

#[derive(IntoRaw)]
#[opcode = 0x20]
pub struct PickN(pub i32);

#[derive(IntoRaw)]
#[opcode = 0x21]
pub struct Dup;

#[derive(IntoRaw)]
#[opcode = 0x22]
pub struct Swap;

#[derive(IntoRaw)]
#[opcode = 0x23]
pub struct Drop;

#[derive(IntoRaw)]
#[opcode = 0x13]
pub struct BitAnd;

#[derive(IntoRaw)]
#[opcode = 0x14]
pub struct BitOr;

#[derive(IntoRaw)]
#[opcode = 0x15]
pub struct BitXor;

#[derive(IntoRaw)]
#[opcode = 0x16]
pub struct BitNot;

#[derive(IntoRaw)]
#[opcode = 0x17]
pub struct ShiftLeft;

#[derive(IntoRaw)]
#[opcode = 0x18]
pub struct ShiftRight;

#[derive(IntoRaw)]
#[opcode = 0x19]
pub struct And;

#[derive(IntoRaw)]
#[opcode = 0x1A]
pub struct Or;

#[derive(IntoRaw)]
#[opcode = 0x1B]
pub struct Xor;

#[derive(IntoRaw)]
#[opcode = 0x1C]
pub struct Not;

#[derive(IntoRaw)]
#[opcode = 0x1D]
pub struct Equal;

#[derive(IntoRaw)]
#[opcode = 0x1E]
pub struct NotEqual;

#[derive(IntoRaw)]
#[opcode = 0x1F]
pub struct LessThan;

#[derive(IntoRaw)]
#[opcode = 0x30]
pub struct StringConcat;

#[derive(IntoRaw)]
#[opcode = 0x31]
pub struct StringLength;

#[derive(IntoRaw)]
#[opcode = 0x32]
pub struct StringSubstring(pub i32, pub i32); // start, length

#[derive(IntoRaw)]
#[opcode = 0x33]
pub struct StringCompare;

#[derive(IntoRaw)]
#[opcode = 0x34]
pub struct StringContains;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modulo_instruction() {
        let modulo = Modulo;
        let raw = modulo.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Modulo);
        assert!(!raw.has_operands().expect("Failed to check operands"));
    }

    #[test]
    fn test_power_instruction() {
        let power = Power;
        let raw = power.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Power);
        assert!(!raw.has_operands().expect("Failed to check operands"));
    }

    #[test]
    fn test_power_instruction_from_raw() {
        let raw = RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x12);
        let instruction: Instruction = raw.try_into().expect("Failed to convert raw to instruction");
        assert_eq!(instruction.opcode, Opcode::Power);
        assert_eq!(instruction.operands, vec![]);
    }

    #[test]
    fn test_pickn_instruction() {
        let pickn = PickN(1);
        let raw = pickn.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::PickN);
        assert_eq!(raw.operand(), 1);
    }

    #[test]
    fn test_dup_instruction() {
        let dup = Dup;
        let raw = dup.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Dup);
        assert!(!raw.has_operands().expect("Failed to check operands"));
    }

    #[test]
    fn test_swap_instruction() {
        let swap = Swap;
        let raw = swap.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Swap);
        assert!(!raw.has_operands().expect("Failed to check operands"));
    }

    #[test]
    fn test_drop_instruction() {
        let drop = Drop;
        let raw = drop.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Drop);
        assert!(!raw.has_operands().expect("Failed to check operands"));
    }

    #[test]
    fn test_raw_instruction() {
        // Create a Store instruction (0x02) with operand 0x010203
        let raw = RawInstruction::from_bytes(0x01, 0x02, 0x03, 0x02);
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Store);
        assert_eq!(raw.operand(), 0x010203);
        assert!(raw.has_operands().expect("Failed to check operands"));
    }

    #[test]
    fn test_raw_instruction_into_instruction() {
        // Test PickN instruction (0x20) with operand 0x010203
        let raw = RawInstruction::from_bytes(0x01, 0x02, 0x03, 0x20);
        let instruction: Instruction = raw.try_into().expect("Failed to convert raw to instruction");
        assert_eq!(instruction.opcode, Opcode::PickN);
        assert_eq!(instruction.operands, vec![0x010203]);

        // Test Dup instruction (0x21)
        let raw = RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x21);
        let instruction: Instruction = raw.try_into().expect("Failed to convert raw to instruction");
        assert_eq!(instruction.opcode, Opcode::Dup);
        assert!(instruction.operands.is_empty());

        // Test Swap instruction (0x22)
        let raw = RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x22);
        let instruction: Instruction = raw.try_into().expect("Failed to convert raw to instruction");
        assert_eq!(instruction.opcode, Opcode::Swap);
        assert!(instruction.operands.is_empty());

        // Test Drop instruction (0x23)
        let raw = RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x23);
        let instruction: Instruction = raw.try_into().expect("Failed to convert raw to instruction");
        assert_eq!(instruction.opcode, Opcode::Drop);
        assert!(instruction.operands.is_empty());
    }

    #[test]
    fn test_into_raw() {
        // Test Add instruction (no operands)
        let add = Add;
        let raw = add.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Add);
        assert!(!raw.has_operands().expect("Failed to check operands"));
        
        // Test Store instruction with operand
        let store = Store(42);
        let raw = store.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Store);
        assert_eq!(raw.operand(), 42);
        assert!(raw.has_operands().expect("Failed to check operands"));
    }

    #[test]
    fn test_bitwise_instructions() {
        // Test BitAnd
        let bitand = BitAnd;
        let raw = bitand.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::BitAnd);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test raw instruction for BitAnd (0x13)
        let raw = RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x13);
        let instruction: Instruction = raw.try_into().expect("Failed to convert raw to instruction");
        assert_eq!(instruction.opcode, Opcode::BitAnd);
        assert!(instruction.operands.is_empty());

        // Test BitOr
        let bitor = BitOr;
        let raw = bitor.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::BitOr);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test raw instruction for BitOr (0x14)
        let raw = RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x14);
        let instruction: Instruction = raw.try_into().expect("Failed to convert raw to instruction");
        assert_eq!(instruction.opcode, Opcode::BitOr);
        assert!(instruction.operands.is_empty());

        // Test BitXor
        let bitxor = BitXor;
        let raw = bitxor.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::BitXor);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test raw instruction for BitXor (0x15)
        let raw = RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x15);
        let instruction: Instruction = raw.try_into().expect("Failed to convert raw to instruction");
        assert_eq!(instruction.opcode, Opcode::BitXor);
        assert!(instruction.operands.is_empty());

        // Test BitNot
        let bitnot = BitNot;
        let raw = bitnot.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::BitNot);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test raw instruction for BitNot (0x16)
        let raw = RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x16);
        let instruction: Instruction = raw.try_into().expect("Failed to convert raw to instruction");
        assert_eq!(instruction.opcode, Opcode::BitNot);
        assert!(instruction.operands.is_empty());

        // Test ShiftLeft
        let shiftleft = ShiftLeft;
        let raw = shiftleft.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::ShiftLeft);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test raw instruction for ShiftLeft (0x17)
        let raw = RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x17);
        let instruction: Instruction = raw.try_into().expect("Failed to convert raw to instruction");
        assert_eq!(instruction.opcode, Opcode::ShiftLeft);
        assert!(instruction.operands.is_empty());

        // Test ShiftRight
        let shiftright = ShiftRight;
        let raw = shiftright.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::ShiftRight);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test raw instruction for ShiftRight (0x18)
        let raw = RawInstruction::from_bytes(0x00, 0x00, 0x00, 0x18);
        let instruction: Instruction = raw.try_into().expect("Failed to convert raw to instruction");
        assert_eq!(instruction.opcode, Opcode::ShiftRight);
        assert!(instruction.operands.is_empty());
    }

    #[test]
    fn test_boolean_instructions() {
        // Test And
        let and = And;
        let raw = and.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::And);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test Or
        let or = Or;
        let raw = or.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Or);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test Xor
        let xor = Xor;
        let raw = xor.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Xor);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test Not
        let not = Not;
        let raw = not.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Not);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test Equal
        let equal = Equal;
        let raw = equal.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Equal);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test NotEqual
        let not_equal = NotEqual;
        let raw = not_equal.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::NotEqual);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test LessThan
        let less_than = LessThan;
        let raw = less_than.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::LessThan);
        assert!(!raw.has_operands().expect("Failed to check operands"));
    }

    #[test]
    fn test_string_instructions() {
        // Test StringConcat
        let concat = StringConcat;
        let raw = concat.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::StringConcat);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test StringLength
        let length = StringLength;
        let raw = length.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::StringLength);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test StringSubstring
        let substring = StringSubstring(5, 10); // start=5, length=10
        let raw = substring.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::StringSubstring);
        assert!(raw.has_operands().expect("Failed to check operands"));
        let operands = raw.get_operands().expect("Failed to get operands");
        assert_eq!(operands.len(), 2);
        assert_eq!(operands[0], 5);
        assert_eq!(operands[1], 10);

        // Test StringCompare
        let compare = StringCompare;
        let raw = compare.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::StringCompare);
        assert!(!raw.has_operands().expect("Failed to check operands"));

        // Test StringContains
        let contains = StringContains;
        let raw = contains.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::StringContains);
        assert!(!raw.has_operands().expect("Failed to check operands"));
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
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Add);
        
        // Test DirectStore
        let store = DirectStore(123);
        let raw = store.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Store);
        assert_eq!(raw.operand(), 123);
        
        // Test DirectSub
        let sub = DirectSub;
        let raw = sub.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Sub);
        
        // Test DirectLoadConstant
        let load = DirectLoadConstant(42);
        let raw = load.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::LoadConstant);
        assert_eq!(raw.operand(), 42);
    }
    
    #[test]
    fn test_derive_macro() {
        // Test derived Add instruction
        let add = CustomAdd;
        let raw = add.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Add);
        
        // Test derived Store instruction
        let store = CustomStore(123);
        let raw = store.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Store);
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
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Add);
        assert!(!raw.has_operands().expect("Failed to check operands"));
        
        // Test MyCustomStoreInstruction (with custom prefix and suffix)
        let store = MyCustomStoreInstruction(42);
        let raw = store.into_raw();
        assert_eq!(raw.opcode().expect("Failed to get opcode"), Opcode::Store);
        assert_eq!(raw.operand(), 42);
        assert!(raw.has_operands().expect("Failed to check operands"));
    }
}


