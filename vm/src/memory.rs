pub struct Memory {
    // We'll use a vector to represent memory
    data: Vec<i32>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    pub fn load(&self, address: usize) -> Result<i32, &'static str> {
        self.data.get(address).copied().ok_or("Memory access out of bounds")
    }

    pub fn store(&mut self, address: usize, value: i32) -> Result<(), &'static str> {
        if address < self.data.len() {
            self.data[address] = value;
            Ok(())
        } else {
            Err("Memory access out of bounds")
        }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_operations() {
        let mut memory = Memory::new(256);
        
        // Test storing a value
        memory.store(42, 123).unwrap();
        
        // Test loading a value
        assert_eq!(memory.load(42).unwrap(), 123);
        
        // Test out of bounds
        assert!(memory.load(1000).is_err());
        assert!(memory.store(1000, 456).is_err());
    }
}