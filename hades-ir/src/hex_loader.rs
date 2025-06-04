use std::collections::HashMap;
use std::io::{Read, Cursor};
use vm::{vm::VirtualMachine};
use crate::{
    executable::{HadesExecutable, SectionType},
    error::Error,
};

#[derive(Debug)]
pub enum LoaderError {
    InvalidExecutable(String),
    MemoryError(String),
    UnsupportedSection(String),
    IoError(std::io::Error),
    VmError(String),
}

impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoaderError::InvalidExecutable(msg) => write!(f, "Invalid executable: {}", msg),
            LoaderError::MemoryError(msg) => write!(f, "Memory error: {}", msg),
            LoaderError::UnsupportedSection(msg) => write!(f, "Unsupported section: {}", msg),
            LoaderError::IoError(e) => write!(f, "I/O error: {}", e),
            LoaderError::VmError(e) => write!(f, "VM error: {}", e),
        }
    }
}

impl std::error::Error for LoaderError {}

impl From<std::io::Error> for LoaderError {
    fn from(err: std::io::Error) -> Self {
        LoaderError::IoError(err)
    }
}

impl From<Error> for LoaderError {
    fn from(err: Error) -> Self {
        LoaderError::InvalidExecutable(err.to_string())
    }
}

/// Loader for Hades Executable (HEX) files
pub struct HexLoader {
    /// The loaded executable
    executable: Option<HadesExecutable>,
    /// Section load addresses
    section_addresses: HashMap<String, usize>,
    /// Symbol table mapping symbol names to addresses
    symbol_table: HashMap<String, usize>,
    /// Entry point address
    entry_point: Option<usize>,
}

impl HexLoader {
    /// Create a new HEX loader
    pub fn new() -> Self {
        Self {
            executable: None,
            section_addresses: HashMap::new(),
            symbol_table: HashMap::new(),
            entry_point: None,
        }
    }

    /// Load a HEX executable from bytes
    pub fn load_from_bytes(&mut self, data: &[u8]) -> std::result::Result<(), LoaderError> {
        let mut cursor = Cursor::new(data);
        let executable = HadesExecutable::read_from(&mut cursor)?;
        
        self.validate_executable(&executable)?;
        self.entry_point = Some(executable.header.entry_point as usize);
        self.executable = Some(executable);
        
        Ok(())
    }

    /// Load a HEX executable from a reader
    pub fn load_from_reader<R: Read>(&mut self, reader: &mut R) -> std::result::Result<(), LoaderError> {
        let executable = HadesExecutable::read_from(reader)?;
        
        self.validate_executable(&executable)?;
        self.entry_point = Some(executable.header.entry_point as usize);
        self.executable = Some(executable);
        
        Ok(())
    }

    /// Validate the loaded executable
    fn validate_executable(&self, executable: &HadesExecutable) -> std::result::Result<(), LoaderError> {
        // Check magic number (already checked during parsing)
        if executable.header.magic != crate::executable::HEX_MAGIC {
            return Err(LoaderError::InvalidExecutable(
                "Invalid magic number".to_string()
            ));
        }

        // Check version compatibility
        if executable.header.version != crate::executable::HEX_VERSION {
            return Err(LoaderError::InvalidExecutable(
                format!("Unsupported version: {}", executable.header.version)
            ));
        }

        // Check architecture
        if executable.header.arch != crate::executable::HEX_ARCH {
            return Err(LoaderError::InvalidExecutable(
                format!("Unsupported architecture: 0x{:04x}", executable.header.arch)
            ));
        }

        // Verify we have at least a code section
        let has_code = executable.sections.iter()
            .any(|s| s.header.section_type == SectionType::Code);
        
        if !has_code {
            return Err(LoaderError::InvalidExecutable(
                "No code section found".to_string()
            ));
        }

        Ok(())
    }

    /// Load the executable into VM memory
    pub fn load_into_vm(&mut self, vm: &mut VirtualMachine) -> std::result::Result<(), LoaderError> {
        // Clone the sections to avoid borrow checker issues
        let sections = if let Some(executable) = &self.executable {
            executable.sections.clone()
        } else {
            return Err(LoaderError::InvalidExecutable("No executable loaded".to_string()));
        };
        
        // Load each section into appropriate memory regions
        for section in &sections {
            self.load_section_into_vm(section, vm)?;
        }

        Ok(())
    }

    /// Load a single section into VM memory
    fn load_section_into_vm(
        &mut self, 
        section: &crate::executable::Section, 
        vm: &mut VirtualMachine
    ) -> std::result::Result<(), LoaderError> {
        let virtual_address = section.header.virtual_address as usize;
        
        // Store section address for symbol resolution
        self.section_addresses.insert(
            format!("section_{}", section.header.section_type as u16),
            virtual_address
        );

        // Load section data into memory
        match section.header.section_type {
            SectionType::Code => {
                self.load_code_section(&section.data, vm)?;
            },
            SectionType::Constants => {
                self.load_constants_section(&section.data, vm, virtual_address)?;
            },
            SectionType::Data | SectionType::Strings => {
                self.load_data_section(&section.data, vm, virtual_address)?;
            },
            SectionType::Symbols | SectionType::Relocations | SectionType::Debug => {
                // Skip metadata sections during loading
            },
            SectionType::Null => {} // Skip null sections
        }

        Ok(())
    }

    /// Load code section into VM
    fn load_code_section(&self, data: &[u8], vm: &mut VirtualMachine) -> std::result::Result<(), LoaderError> {
        // Code should be in 4-byte instructions
        if data.len() % 4 != 0 {
            return Err(LoaderError::InvalidExecutable(
                "Code section size must be a multiple of 4 bytes".to_string()
            ));
        }

        // Load the code as a program
        vm.load_program(data);
        
        Ok(())
    }

    /// Load constants section into VM memory
    fn load_constants_section(
        &self, 
        data: &[u8], 
        vm: &mut VirtualMachine,
        _base_address: usize
    ) -> std::result::Result<(), LoaderError> {
        // Parse constant pool format
        let mut cursor = Cursor::new(data);
        
        // Read constant count
        let mut count_bytes = [0u8; 4];
        cursor.read_exact(&mut count_bytes)?;
        let count = u32::from_le_bytes(count_bytes) as usize;
        
        // Read each constant
        for _ in 0..count {
            let mut type_tag = [0u8; 1];
            cursor.read_exact(&mut type_tag)?;
            
            match type_tag[0] {
                1 => { // Integer constant
                    let mut value_bytes = [0u8; 8];
                    cursor.read_exact(&mut value_bytes)?;
                    let _value = i64::from_le_bytes(value_bytes);
                    
                    // In a complete implementation, we'd write to memory
                },
                2 => { // Float constant
                    let mut value_bytes = [0u8; 4];
                    cursor.read_exact(&mut value_bytes)?;
                    let _value = f32::from_le_bytes(value_bytes);
                },
                3 => { // Boolean constant
                    let mut value_byte = [0u8; 1];
                    cursor.read_exact(&mut value_byte)?;
                    let _value = value_byte[0] != 0;
                },
                4 => { // String constant
                    let mut len_bytes = [0u8; 4];
                    cursor.read_exact(&mut len_bytes)?;
                    let len = u32::from_le_bytes(len_bytes) as usize;
                    
                    let mut string_data = vec![0u8; len];
                    cursor.read_exact(&mut string_data)?;
                    let _string_value = String::from_utf8_lossy(&string_data);
                },
                _ => {
                    return Err(LoaderError::InvalidExecutable(
                        format!("Unknown constant type tag: {}", type_tag[0])
                    ));
                }
            }
        }
        
        Ok(())
    }

    /// Load data section into VM memory
    fn load_data_section(
        &self, 
        _data: &[u8], 
        _vm: &mut VirtualMachine,
        _base_address: usize
    ) -> std::result::Result<(), LoaderError> {
        // For now, we'll just note that the data exists
        // In a complete implementation, we'd write it to the appropriate memory region
        Ok(())
    }

    /// Get the entry point address
    pub fn get_entry_point(&self) -> Option<usize> {
        self.entry_point
    }

    /// Get a symbol address by name
    pub fn get_symbol_address(&self, name: &str) -> Option<usize> {
        self.symbol_table.get(name).copied()
    }

    /// List all loaded sections
    pub fn list_sections(&self) -> Vec<String> {
        self.section_addresses.keys().cloned().collect()
    }

    /// Get section information
    pub fn get_section_info(&self, name: &str) -> Option<(usize, usize)> {
        if let Some(executable) = &self.executable {
            for section in &executable.sections {
                let section_name = format!("section_{}", section.header.section_type as u16);
                if section_name == name {
                    return Some((
                        section.header.virtual_address as usize,
                        section.header.file_size as usize
                    ));
                }
            }
        }
        None
    }
}

impl Default for HexLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to load and run a HEX file
pub fn load_and_run_hex(data: &[u8], debug: bool) -> std::result::Result<(), LoaderError> {
    let mut loader = HexLoader::new();
    loader.load_from_bytes(data)?;
    
    let mut vm = VirtualMachine::new(debug);
    loader.load_into_vm(&mut vm)?;
    
    if debug {
        println!("Loaded HEX executable with entry point: {:?}", loader.get_entry_point());
        println!("Sections: {:?}", loader.list_sections());
    }
    
    vm.run_until_completion();
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executable::{HadesExecutable, SectionType};

    #[test]
    fn test_hex_loading() {
        // Create a simple executable
        let mut executable = HadesExecutable::new(0x1000);
        
        // Add a simple code section
        let code = vec![
            0x00, 0x00, 0x2A, 0x04, // LoadConstant 42
            0x00, 0x00, 0x00, 0x06, // Print
            0x00, 0x00, 0x00, 0x0E, // Return
        ];
        executable.add_section(".text", SectionType::Code, code);
        
        // Serialize to bytes
        let mut buffer = Vec::new();
        executable.write_to(&mut buffer).unwrap();
        
        // Load with HexLoader
        let mut loader = HexLoader::new();
        loader.load_from_bytes(&buffer).unwrap();
        
        assert_eq!(loader.get_entry_point(), Some(0x1000));
        assert!(loader.list_sections().len() > 0);
    }

    #[test]
    fn test_hex_vm_integration() {
        // Create a simple executable
        let mut executable = HadesExecutable::new(0x1000);
        
        let code = vec![
            0x00, 0x00, 0x2A, 0x04, // LoadConstant 42
            0x00, 0x00, 0x00, 0x06, // Print
        ];
        executable.add_section(".text", SectionType::Code, code);
        
        // Serialize to bytes
        let mut buffer = Vec::new();
        executable.write_to(&mut buffer).unwrap();
        
        // Load and run
        load_and_run_hex(&buffer, true).unwrap();
    }
} 