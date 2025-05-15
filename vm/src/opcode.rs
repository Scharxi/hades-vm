#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    Add = 0x01,
    Store = 0x2,
    Sub = 0x3,
    LoadConstant = 0x04,
    Multiply = 0x5,
    Print = 0x6,
    LoadMemory = 0x7,
    StoreMemory = 0x8,
    JumpIfZero = 0x9,
    Divide = 0xA,
    LoadFromRegion = 0xB,
    StoreToRegion = 0xC,
    Call = 0x0D,
    Return = 0x0E,
    LoadLocal = 0x0F,
    StoreLocal = 0x10,
    Modulo = 0x11,
    Power = 0x12,
    BitAnd = 0x13,
    BitOr = 0x14,
    BitXor = 0x15,
    BitNot = 0x16,
    ShiftLeft = 0x17,
    ShiftRight = 0x18,
    And = 0x19,
    Or = 0x1A,
    Xor = 0x1B,
    Not = 0x1C,
    Equal = 0x1D,
    NotEqual = 0x1E,
    LessThan = 0x1F,
    PickN = 0x20,
    Dup = 0x21,
    Swap = 0x22,
    Drop = 0x23,
    StringConcat = 0x30,
    StringLength = 0x31,
    StringSubstring = 0x32,
    StringCompare = 0x33,
    StringContains = 0x34,
    Alloc = 0x50,
    Free = 0x51,
    LoadHeap = 0x52,  // Laden von Werten aus dem Heap
    StoreHeap = 0x53, // Speichern von Werten im Heap
    MemSet = 0x54,    // Initialisieren eines Speicherbereichs mit einem Wert
    Yield = 0xF0,
    TerminateProcess = 0xFF,
    CreateFrame = 0x40,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidOpcodeError(pub u8);

impl Opcode {
    pub fn operand_count(&self) -> usize {
        match self {
            Opcode::Add => 0,
            Opcode::Store => 1,
            Opcode::Sub => 0,
            Opcode::Divide => 0,
            Opcode::LoadFromRegion => 2,
            Opcode::StoreToRegion => 2,
            Opcode::LoadConstant => 1,
            Opcode::Multiply => 0,
            Opcode::Print => 0,
            Opcode::LoadMemory => 1,
            Opcode::StoreMemory => 1,
            Opcode::JumpIfZero => 1,
            Opcode::Call => 2,
            Opcode::Return => 0,
            Opcode::LoadLocal => 1,
            Opcode::StoreLocal => 1,
            Opcode::Modulo => 0,
            Opcode::Power => 0,
            Opcode::BitAnd => 0,
            Opcode::BitOr => 0,
            Opcode::BitXor => 0,
            Opcode::BitNot => 0,
            Opcode::ShiftLeft => 0,
            Opcode::ShiftRight => 0,
            Opcode::And => 0,
            Opcode::Or => 0,
            Opcode::Xor => 0,
            Opcode::Not => 0,
            Opcode::Equal => 0,
            Opcode::NotEqual => 0,
            Opcode::LessThan => 0,
            Opcode::PickN => 1,
            Opcode::Dup => 0,
            Opcode::Swap => 0,
            Opcode::Drop => 0,
            Opcode::StringConcat => 0,
            Opcode::StringLength => 0,
            Opcode::StringSubstring => 2,
            Opcode::StringCompare => 0,
            Opcode::StringContains => 0,
            Opcode::Alloc => 1,
            Opcode::Free => 0, // Takes no operands, pops address from stack
            Opcode::LoadHeap => 0, // Erwartet zwei Werte auf dem Stack: Adresse und Offset
            Opcode::StoreHeap => 0, // Erwartet drei Werte auf dem Stack: Adresse, Offset und Wert
            Opcode::MemSet => 0, // Erwartet drei Werte auf dem Stack: Adresse, Anzahl und Wert
            Opcode::Yield => 0,
            Opcode::TerminateProcess => 0,
            Opcode::CreateFrame => 1,
        }
    }
}

impl Into<u8> for Opcode {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for Opcode {
    type Error = InvalidOpcodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Opcode::Add),
            0x02 => Ok(Opcode::Store),
            0x03 => Ok(Opcode::Sub),
            0x04 => Ok(Opcode::LoadConstant),
            0x05 => Ok(Opcode::Multiply),
            0x06 => Ok(Opcode::Print),
            0x07 => Ok(Opcode::LoadMemory),
            0x08 => Ok(Opcode::StoreMemory),
            0x09 => Ok(Opcode::JumpIfZero),
            0x0A => Ok(Opcode::Divide),
            0x0B => Ok(Opcode::LoadFromRegion),
            0x0C => Ok(Opcode::StoreToRegion),
            0x0D => Ok(Opcode::Call),
            0x0E => Ok(Opcode::Return),
            0x0F => Ok(Opcode::LoadLocal),
            0x10 => Ok(Opcode::StoreLocal),
            0x11 => Ok(Opcode::Modulo),
            0x12 => Ok(Opcode::Power),
            0x13 => Ok(Opcode::BitAnd),
            0x14 => Ok(Opcode::BitOr),
            0x15 => Ok(Opcode::BitXor),
            0x16 => Ok(Opcode::BitNot),
            0x17 => Ok(Opcode::ShiftLeft),
            0x18 => Ok(Opcode::ShiftRight),
            0x19 => Ok(Opcode::And),
            0x1A => Ok(Opcode::Or),
            0x1B => Ok(Opcode::Xor),
            0x1C => Ok(Opcode::Not),
            0x1D => Ok(Opcode::Equal),
            0x1E => Ok(Opcode::NotEqual),
            0x1F => Ok(Opcode::LessThan),
            0x20 => Ok(Opcode::PickN),
            0x21 => Ok(Opcode::Dup),
            0x22 => Ok(Opcode::Swap),
            0x23 => Ok(Opcode::Drop),
            0x30 => Ok(Opcode::StringConcat),
            0x31 => Ok(Opcode::StringLength),
            0x32 => Ok(Opcode::StringSubstring),
            0x33 => Ok(Opcode::StringCompare),
            0x34 => Ok(Opcode::StringContains),
            0x50 => Ok(Opcode::Alloc),
            0x51 => Ok(Opcode::Free),
            0x52 => Ok(Opcode::LoadHeap),
            0x53 => Ok(Opcode::StoreHeap),
            0x54 => Ok(Opcode::MemSet),
            0xF0 => Ok(Opcode::Yield),
            0xFF => Ok(Opcode::TerminateProcess),
            0x40 => Ok(Opcode::CreateFrame),
            _ => Err(InvalidOpcodeError(value)),
        }
    }
}

impl TryFrom<i32> for Opcode {
    type Error = InvalidOpcodeError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 || value > u8::MAX as i32 {
            Err(InvalidOpcodeError(value as u8))
        } else {
            Opcode::try_from(value as u8)
        }
    }
}

/// Trait to allow mapping between opcode names and their values
pub trait OpcodeMapping {
    /// Get the opcode value from the type name
    fn get_opcode_from_name(name: &str) -> Option<u8>;
}

impl OpcodeMapping for Opcode {
    fn get_opcode_from_name(name: &str) -> Option<u8> {
        match name {
            "Add" => Some(Opcode::Add as u8),
            "Store" => Some(Opcode::Store as u8),
            "Sub" => Some(Opcode::Sub as u8),
            "LoadConstant" => Some(Opcode::LoadConstant as u8),
            "Multiply" => Some(Opcode::Multiply as u8),
            "Print" => Some(Opcode::Print as u8),
            "LoadMemory" => Some(Opcode::LoadMemory as u8),
            "StoreMemory" => Some(Opcode::StoreMemory as u8),
            "JumpIfZero" => Some(Opcode::JumpIfZero as u8),
            "Divide" => Some(Opcode::Divide as u8),
            "LoadFromRegion" => Some(Opcode::LoadFromRegion as u8),
            "StoreToRegion" => Some(Opcode::StoreToRegion as u8),
            "Call" => Some(Opcode::Call as u8),
            "Return" => Some(Opcode::Return as u8),
            "LoadLocal" => Some(Opcode::LoadLocal as u8),
            "StoreLocal" => Some(Opcode::StoreLocal as u8),
            "Modulo" => Some(Opcode::Modulo as u8),
            "Power" => Some(Opcode::Power as u8),
            "BitAnd" => Some(Opcode::BitAnd as u8),
            "BitOr" => Some(Opcode::BitOr as u8),
            "BitXor" => Some(Opcode::BitXor as u8),
            "BitNot" => Some(Opcode::BitNot as u8),
            "ShiftLeft" => Some(Opcode::ShiftLeft as u8),
            "ShiftRight" => Some(Opcode::ShiftRight as u8),
            "And" => Some(Opcode::And as u8),
            "Or" => Some(Opcode::Or as u8),
            "Xor" => Some(Opcode::Xor as u8),
            "Not" => Some(Opcode::Not as u8),
            "Equal" => Some(Opcode::Equal as u8),
            "NotEqual" => Some(Opcode::NotEqual as u8),
            "LessThan" => Some(Opcode::LessThan as u8),
            "PickN" => Some(Opcode::PickN as u8),
            "Alloc" => Some(Opcode::Alloc as u8),
            "Free" => Some(Opcode::Free as u8),
            "LoadHeap" => Some(Opcode::LoadHeap as u8),
            "StoreHeap" => Some(Opcode::StoreHeap as u8),
            "MemSet" => Some(Opcode::MemSet as u8),
            "Yield" => Some(Opcode::Yield as u8),
            "TerminateProcess" => Some(Opcode::TerminateProcess as u8),
            "StringConcat" => Some(Opcode::StringConcat as u8),
            "StringLength" => Some(Opcode::StringLength as u8),
            "StringSubstring" => Some(Opcode::StringSubstring as u8),
            "StringCompare" => Some(Opcode::StringCompare as u8),
            "StringContains" => Some(Opcode::StringContains as u8),
            "CreateFrame" => Some(Opcode::CreateFrame as u8),
            _ => None,
        }
    }
}
