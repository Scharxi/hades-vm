use std::collections::HashMap;
use std::io::{Write, Read, Cursor};
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use crate::error::{Error, Result};

/// Magic number identifying a Hades Executable file
pub const HEX_MAGIC: u32 = 0x48455820; // "HEX " in ASCII

/// Current version of the HEX format
pub const HEX_VERSION: u16 = 0x0001;

/// Architecture identifier for Hades VM
pub const HEX_ARCH: u16 = 0x4844; // "HD" (Hades)

/// Section types in the executable
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum SectionType {
    /// Null section (unused)
    Null = 0x0000,
    /// Executable code section  
    Code = 0x0001,
    /// Read-only data section
    Data = 0x0002,
    /// Constants pool
    Constants = 0x0003,
    /// Symbol table
    Symbols = 0x0004,
    /// String table
    Strings = 0x0005,
    /// Relocation table
    Relocations = 0x0006,
    /// Debug information
    Debug = 0x0007,
}

impl TryFrom<u16> for SectionType {
    type Error = Error;
    
    fn try_from(value: u16) -> Result<Self> {
        match value {
            0x0000 => Ok(SectionType::Null),
            0x0001 => Ok(SectionType::Code),
            0x0002 => Ok(SectionType::Data),
            0x0003 => Ok(SectionType::Constants),
            0x0004 => Ok(SectionType::Symbols),
            0x0005 => Ok(SectionType::Strings),
            0x0006 => Ok(SectionType::Relocations),
            0x0007 => Ok(SectionType::Debug),
            _ => Err(Error::ExecutableError(format!("Unknown section type: 0x{:04x}", value))),
        }
    }
}

/// Executable file header
#[derive(Debug, Clone)]
pub struct HexHeader {
    /// Magic number (should be HEX_MAGIC)
    pub magic: u32,
    /// File format version
    pub version: u16,
    /// Target architecture
    pub arch: u16,
    /// Entry point address
    pub entry_point: u32,
    /// Number of sections
    pub section_count: u16,
    /// Offset to section header table
    pub section_offset: u32,
    /// Size of section header entry
    pub section_size: u16,
    /// Offset to string table
    pub string_table_offset: u32,
    /// Reserved for future use
    pub reserved: [u32; 4],
}

impl HexHeader {
    pub fn new(entry_point: u32, section_count: u16) -> Self {
        Self {
            magic: HEX_MAGIC,
            version: HEX_VERSION,
            arch: HEX_ARCH,
            entry_point,
            section_count,
            section_offset: 0, // Will be set later
            section_size: std::mem::size_of::<SectionHeader>() as u16,
            string_table_offset: 0, // Will be set later
            reserved: [0; 4],
        }
    }

    /// Write header to a writer
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<LittleEndian>(self.magic)?;
        writer.write_u16::<LittleEndian>(self.version)?;
        writer.write_u16::<LittleEndian>(self.arch)?;
        writer.write_u32::<LittleEndian>(self.entry_point)?;
        writer.write_u16::<LittleEndian>(self.section_count)?;
        writer.write_u32::<LittleEndian>(self.section_offset)?;
        writer.write_u16::<LittleEndian>(self.section_size)?;
        writer.write_u32::<LittleEndian>(self.string_table_offset)?;
        
        for &reserved in &self.reserved {
            writer.write_u32::<LittleEndian>(reserved)?;
        }
        
        Ok(())
    }

    /// Read header from a reader
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let magic = reader.read_u32::<LittleEndian>()?;
        if magic != HEX_MAGIC {
            return Err(Error::ExecutableError(format!(
                "Invalid magic number: expected 0x{:08x}, got 0x{:08x}", 
                HEX_MAGIC, magic
            )));
        }

        let version = reader.read_u16::<LittleEndian>()?;
        let arch = reader.read_u16::<LittleEndian>()?;
        let entry_point = reader.read_u32::<LittleEndian>()?;
        let section_count = reader.read_u16::<LittleEndian>()?;
        let section_offset = reader.read_u32::<LittleEndian>()?;
        let section_size = reader.read_u16::<LittleEndian>()?;
        let string_table_offset = reader.read_u32::<LittleEndian>()?;
        
        let mut reserved = [0u32; 4];
        for reserved_slot in &mut reserved {
            *reserved_slot = reader.read_u32::<LittleEndian>()?;
        }

        Ok(Self {
            magic,
            version,
            arch,
            entry_point,
            section_count,
            section_offset,
            section_size,
            string_table_offset,
            reserved,
        })
    }

    /// Size of the header in bytes
    pub const fn size() -> usize {
        4 + 2 + 2 + 4 + 2 + 4 + 2 + 4 + (4 * 4) // 32 bytes total
    }
}

/// Section header
#[derive(Debug, Clone)]
pub struct SectionHeader {
    /// Section name (offset into string table)
    pub name_offset: u32,
    /// Section type
    pub section_type: SectionType,
    /// Section flags
    pub flags: u16,
    /// Virtual address where section should be loaded
    pub virtual_address: u32,
    /// Offset in file where section data starts
    pub file_offset: u32,
    /// Size of section in file
    pub file_size: u32,
    /// Size of section in memory (may be larger than file_size)
    pub memory_size: u32,
    /// Alignment requirement
    pub alignment: u32,
}

impl SectionHeader {
    pub fn new(
        name_offset: u32,
        section_type: SectionType,
        flags: u16,
        virtual_address: u32,
        file_offset: u32,
        file_size: u32,
        memory_size: u32,
        alignment: u32,
    ) -> Self {
        Self {
            name_offset,
            section_type,
            flags,
            virtual_address,
            file_offset,
            file_size,
            memory_size,
            alignment,
        }
    }

    /// Write section header to a writer
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<LittleEndian>(self.name_offset)?;
        writer.write_u16::<LittleEndian>(self.section_type as u16)?;
        writer.write_u16::<LittleEndian>(self.flags)?;
        writer.write_u32::<LittleEndian>(self.virtual_address)?;
        writer.write_u32::<LittleEndian>(self.file_offset)?;
        writer.write_u32::<LittleEndian>(self.file_size)?;
        writer.write_u32::<LittleEndian>(self.memory_size)?;
        writer.write_u32::<LittleEndian>(self.alignment)?;
        Ok(())
    }

    /// Read section header from a reader
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let name_offset = reader.read_u32::<LittleEndian>()?;
        let section_type = SectionType::try_from(reader.read_u16::<LittleEndian>()?)?;
        let flags = reader.read_u16::<LittleEndian>()?;
        let virtual_address = reader.read_u32::<LittleEndian>()?;
        let file_offset = reader.read_u32::<LittleEndian>()?;
        let file_size = reader.read_u32::<LittleEndian>()?;
        let memory_size = reader.read_u32::<LittleEndian>()?;
        let alignment = reader.read_u32::<LittleEndian>()?;

        Ok(Self {
            name_offset,
            section_type,
            flags,
            virtual_address,
            file_offset,
            file_size,
            memory_size,
            alignment,
        })
    }

    /// Size of section header in bytes
    pub const fn size() -> usize {
        4 + 2 + 2 + 4 + 4 + 4 + 4 + 4 // 32 bytes total
    }
}

/// Symbol entry in the symbol table
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name (offset into string table)
    pub name_offset: u32,
    /// Symbol value/address
    pub value: u32,
    /// Symbol size
    pub size: u32,
    /// Symbol type and binding
    pub info: u8,
    /// Symbol visibility
    pub visibility: u8,
    /// Section index where symbol is defined
    pub section_index: u16,
}

impl Symbol {
    pub fn new(
        name_offset: u32,
        value: u32,
        size: u32,
        info: u8,
        visibility: u8,
        section_index: u16,
    ) -> Self {
        Self {
            name_offset,
            value,
            size,
            info,
            visibility,
            section_index,
        }
    }

    /// Write symbol to a writer
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u32::<LittleEndian>(self.name_offset)?;
        writer.write_u32::<LittleEndian>(self.value)?;
        writer.write_u32::<LittleEndian>(self.size)?;
        writer.write_u8(self.info)?;
        writer.write_u8(self.visibility)?;
        writer.write_u16::<LittleEndian>(self.section_index)?;
        Ok(())
    }

    /// Read symbol from a reader
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let name_offset = reader.read_u32::<LittleEndian>()?;
        let value = reader.read_u32::<LittleEndian>()?;
        let size = reader.read_u32::<LittleEndian>()?;
        let info = reader.read_u8()?;
        let visibility = reader.read_u8()?;
        let section_index = reader.read_u16::<LittleEndian>()?;

        Ok(Self {
            name_offset,
            value,
            size,
            info,
            visibility,
            section_index,
        })
    }

    /// Size of symbol entry in bytes
    pub const fn size() -> usize {
        4 + 4 + 4 + 1 + 1 + 2 // 16 bytes total
    }
}

/// A section in the executable
#[derive(Debug, Clone)]
pub struct Section {
    pub header: SectionHeader,
    pub data: Vec<u8>,
}

impl Section {
    pub fn new(header: SectionHeader, data: Vec<u8>) -> Self {
        Self { header, data }
    }
}

/// The main executable structure
#[derive(Debug, Clone)]
pub struct HadesExecutable {
    pub header: HexHeader,
    pub sections: Vec<Section>,
    pub symbols: Vec<Symbol>,
    pub string_table: Vec<u8>,
    pub string_offsets: HashMap<String, u32>,
}

impl HadesExecutable {
    /// Create a new executable
    pub fn new(entry_point: u32) -> Self {
        let mut exe = Self {
            header: HexHeader::new(entry_point, 0),
            sections: Vec::new(),
            symbols: Vec::new(),
            string_table: vec![0], // Start with null string
            string_offsets: HashMap::new(),
        };
        
        // Add the null string
        exe.string_offsets.insert(String::new(), 0);
        exe
    }

    /// Add a string to the string table and return its offset
    pub fn add_string(&mut self, s: &str) -> u32 {
        if let Some(&offset) = self.string_offsets.get(s) {
            return offset;
        }

        let offset = self.string_table.len() as u32;
        self.string_table.extend_from_slice(s.as_bytes());
        self.string_table.push(0); // Null terminator
        self.string_offsets.insert(s.to_string(), offset);
        offset
    }

    /// Add a section to the executable
    pub fn add_section(&mut self, name: &str, section_type: SectionType, data: Vec<u8>) -> u32 {
        let name_offset = self.add_string(name);
        let section_index = self.sections.len() as u32;
        
        let header = SectionHeader::new(
            name_offset,
            section_type,
            0, // flags
            0, // virtual_address (will be set during layout)
            0, // file_offset (will be set during layout)
            data.len() as u32, // file_size
            data.len() as u32, // memory_size
            4, // alignment
        );

        self.sections.push(Section::new(header, data));
        self.header.section_count = self.sections.len() as u16;
        
        section_index
    }

    /// Add a symbol to the executable
    pub fn add_symbol(&mut self, name: &str, value: u32, size: u32, section_index: u16) {
        let name_offset = self.add_string(name);
        let symbol = Symbol::new(
            name_offset,
            value,
            size,
            0, // info
            0, // visibility
            section_index,
        );
        self.symbols.push(symbol);
    }

    /// Layout the executable in memory and calculate offsets
    fn layout(&mut self) {
        // First, add symbol and string table sections if needed
        self.finalize_sections();
        
        // Now calculate offsets with all sections known
        let section_headers_size = self.sections.len() as u32 * SectionHeader::size() as u32;
        let mut offset = HexHeader::size() as u32 + section_headers_size;
        
        // Layout all sections
        for section in &mut self.sections {
            // Align to section alignment
            let alignment = section.header.alignment;
            if alignment > 1 {
                offset = (offset + alignment - 1) & !(alignment - 1);
            }
            
            section.header.file_offset = offset;
            section.header.virtual_address = offset; // For simplicity, file and virtual addresses are the same
            
            // Set string table offset if this is the string table section
            if section.header.section_type == SectionType::Strings {
                self.header.string_table_offset = offset;
            }
            
            offset += section.header.file_size;
        }
        
        // Update header information
        self.header.section_count = self.sections.len() as u16;
        self.header.section_offset = HexHeader::size() as u32; // Section headers come right after main header
    }
    
    /// Finalize sections by adding symbol and string table sections
    fn finalize_sections(&mut self) {
        let mut sections_to_add = Vec::new();
        
        // Add symbol table section if we have symbols
        if !self.symbols.is_empty() {
            let symbol_data_size = (self.symbols.len() * Symbol::size()) as u32;
            let symbol_name_offset = self.add_string(".symtab");
            
            let symbol_header = SectionHeader::new(
                symbol_name_offset,
                SectionType::Symbols,
                0,
                0, // Will be set in layout
                0, // Will be set in layout
                symbol_data_size,
                symbol_data_size,
                4,
            );
            
            // Create symbol data
            let mut symbol_data = Vec::new();
            for symbol in &self.symbols {
                symbol.write_to(&mut Cursor::new(&mut symbol_data)).unwrap();
            }
            
            sections_to_add.push(Section::new(symbol_header, symbol_data));
        }
        
        // Add string table section
        let strings_copy = self.string_table.clone();
        let string_table_len = strings_copy.len() as u32;
        let string_name_offset = self.add_string(".strtab");
        let string_header = SectionHeader::new(
            string_name_offset,
            SectionType::Strings,
            0,
            0, // Will be set in layout
            0, // Will be set in layout
            string_table_len,
            string_table_len,
            1,
        );
        
        sections_to_add.push(Section::new(string_header, strings_copy));
        
        // Add all new sections
        for section in sections_to_add {
            self.sections.push(section);
        }
    }

    /// Write the executable to a writer
    pub fn write_to<W: Write>(&mut self, writer: &mut W) -> Result<()> {
        // Layout the executable first
        self.layout();
        
        // Write header
        self.header.write_to(writer)?;
        
        // Write section headers immediately after main header
        for section in &self.sections {
            section.header.write_to(writer)?;
        }
        
        // Write section data
        for section in &self.sections {
            writer.write_all(&section.data)?;
        }
        
        Ok(())
    }

    /// Read an executable from a reader
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        // Read header
        let header = HexHeader::read_from(reader)?;
        
        // Validate header
        if header.version != HEX_VERSION {
            return Err(Error::ExecutableError(format!(
                "Unsupported version: expected {}, got {}", 
                HEX_VERSION, header.version
            )));
        }
        
        if header.arch != HEX_ARCH {
            return Err(Error::ExecutableError(format!(
                "Unsupported architecture: expected 0x{:04x}, got 0x{:04x}", 
                HEX_ARCH, header.arch
            )));
        }
        
        // Read section headers
        let mut sections = Vec::new();
        for _ in 0..header.section_count {
            let section_header = SectionHeader::read_from(reader)?;
            sections.push(Section::new(section_header, Vec::new()));
        }
        
        // Read section data
        for section in &mut sections {
            let mut data = vec![0u8; section.header.file_size as usize];
            reader.read_exact(&mut data)?;
            section.data = data;
        }
        
        // TODO: Parse symbols and string table from sections
        
        Ok(Self {
            header,
            sections,
            symbols: Vec::new(),
            string_table: Vec::new(),
            string_offsets: HashMap::new(),
        })
    }

    /// Get a section by type
    pub fn get_section_by_type(&self, section_type: SectionType) -> Option<&Section> {
        self.sections.iter().find(|s| s.header.section_type == section_type)
    }

    /// Get the code section
    pub fn get_code_section(&self) -> Option<&Section> {
        self.get_section_by_type(SectionType::Code)
    }

    /// Get the constants section
    pub fn get_constants_section(&self) -> Option<&Section> {
        self.get_section_by_type(SectionType::Constants)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_header_serialization() {
        let header = HexHeader::new(0x1000, 3);
        let mut buffer = Vec::new();
        header.write_to(&mut buffer).unwrap();
        
        let mut cursor = Cursor::new(&buffer);
        let decoded = HexHeader::read_from(&mut cursor).unwrap();
        
        assert_eq!(header.magic, decoded.magic);
        assert_eq!(header.entry_point, decoded.entry_point);
        assert_eq!(header.section_count, decoded.section_count);
    }

    #[test]
    fn test_executable_creation() {
        let mut exe = HadesExecutable::new(0x1000);
        
        // Add a code section
        let code = vec![0x00, 0x00, 0x2A, 0x04]; // LoadConstant 42
        exe.add_section(".text", SectionType::Code, code);
        
        // Add a symbol
        exe.add_symbol("main", 0x1000, 4, 0);
        
        // Write to buffer
        let mut buffer = Vec::new();
        exe.write_to(&mut buffer).unwrap();
        
        // Should be able to read it back
        let mut cursor = Cursor::new(&buffer);
        let decoded = HadesExecutable::read_from(&mut cursor).unwrap();
        
        assert_eq!(exe.header.entry_point, decoded.header.entry_point);
        assert_eq!(exe.sections.len(), decoded.sections.len());
    }
} 