#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    Add = 0x1,
    Store = 0x2, 
    Sub = 0x3,
    LoadConstant = 0x4,
    Multiply = 0x5,
    Print = 0x6,
    LoadMemory = 0x7,
    StoreMemory = 0x8,
    JumpIfZero = 0x9,
    Divide = 0xA,
    LoadFromRegion = 0xB,
    StoreToRegion = 0xC,
    Call = 0xD,
    Return = 0xE,
    LoadLocal = 0xF,
    StoreLocal = 0x10,
    Modulo = 0x11,
    Power = 0x12, 
    PickN = 0x13, 
}

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
            Opcode::PickN => 1, 
        }
    }
}

impl Into<u8> for Opcode {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}


impl From<i32> for Opcode {
    fn from(value: i32) -> Self {
        Opcode::from(value as u8)
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
            "PickN" => Some(Opcode::PickN as u8),
            _ => None,
        }
    }
}