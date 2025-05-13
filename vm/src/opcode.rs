#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    Add = 0x1,
    Store = 0x2, 
}

impl Opcode {
    pub fn operand_count(&self) -> usize {
        match self {
            Opcode::Add => 0,
            Opcode::Store => 1, 
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