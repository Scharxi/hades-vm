//! # Memory Management Module
//! 
//! This module provides a flexible memory management system for the Hades VM.
//! It implements a segmented memory architecture with different memory regions,
//! each with specific access permissions. This design allows for memory 
//! protection and isolation between different parts of the VM.

use core::fmt;
use std::{collections::HashMap, ops::Range};

/// Represents different types of memory regions supported by the VM.
/// 
/// Each region type has a specific purpose and typically different access permissions.
/// The VM uses these regions to organize memory according to its intended use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryRegionType {
    /// Program code and instructions
    Code,
    /// Static/global data
    Data,
    /// Call and execution stack
    Stack,
    /// Dynamically allocated memory
    Heap,
    /// Memory-mapped I/O for device interaction
    IO,
    /// Read-only constants
    Constants,
}

impl fmt::Display for MemoryRegionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryRegionType::Code => write!(f, "CODE"),
            MemoryRegionType::Data => write!(f, "DATA"),
            MemoryRegionType::Stack => write!(f, "STACK"),
            MemoryRegionType::Heap => write!(f, "HEAP"),
            MemoryRegionType::IO => write!(f, "IO"),
            MemoryRegionType::Constants => write!(f, "CONSTANTS"),
        }
    }
}

/// Access permissions that can be applied to memory regions.
/// 
/// These permissions control what operations are allowed on a specific region,
/// providing memory protection and preventing unauthorized access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPermission {
    /// Permission to read from memory
    Read,
    /// Permission to write to memory
    Write,
    /// Permission to execute code from memory
    Execute,
}

/// Represents a contiguous region in memory with specific properties.
/// 
/// A MemoryRegion defines a section of memory with a specific type, size,
/// starting address, and access permissions. Regions are used to organize
/// the VM's memory space according to different usage patterns.
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    /// The type of this memory region
    pub region_type: MemoryRegionType,
    /// Starting address of the region
    pub start: usize,
    /// Size of the region in elements (words)
    pub size: usize,
    /// Access permissions for this region
    pub permissions: Vec<AccessPermission>,
    /// Human-readable description of the region
    pub description: String,
}

impl MemoryRegion {
    /// Creates a new memory region with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `region_type` - The type of memory region
    /// * `start` - Starting address of the region
    /// * `size` - Size of the region in elements (words)
    /// * `permissions` - Vector of access permissions for this region
    /// * `description` - Optional human-readable description (defaults to region type name)
    ///
    /// # Returns
    ///
    /// A new `MemoryRegion` instance
    pub fn new(
        region_type: MemoryRegionType,
        start: usize,
        size: usize,
        permissions: Vec<AccessPermission>,
        description: Option<String>,
    ) -> Self {
        Self {
            region_type,
            start,
            size,
            permissions,
            description: description.unwrap_or_else(|| format!("{} Region", region_type)),
        }
    }
    
    /// Returns the address range covered by this region.
    ///
    /// # Returns
    ///
    /// A Range from the start address (inclusive) to the end address (exclusive)
    pub fn address_range(&self) -> Range<usize> {
        self.start..(self.start + self.size)
    }
    
    /// Checks if a given address falls within this region.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to check
    ///
    /// # Returns
    ///
    /// `true` if the address is within this region, `false` otherwise
    pub fn contains_address(&self, address: usize) -> bool {
        self.address_range().contains(&address)
    }
    
    /// Checks if this region has a specific access permission.
    ///
    /// # Arguments
    ///
    /// * `permission` - The permission to check
    ///
    /// # Returns
    ///
    /// `true` if the region has the specified permission, `false` otherwise
    pub fn has_permission(&self, permission: AccessPermission) -> bool {
        self.permissions.contains(&permission)
    }
    
    /// Converts a global memory address to a region-local offset.
    ///
    /// # Arguments
    ///
    /// * `address` - The global address to convert
    ///
    /// # Returns
    ///
    /// `Some(offset)` if the address is within this region, `None` otherwise
    pub fn address_to_offset(&self, address: usize) -> Option<usize> {
        if self.contains_address(address) {
            Some(address - self.start)
        } else {
            None
        }
    }
    
    /// Converts a region-local offset to a global memory address.
    ///
    /// # Arguments
    ///
    /// * `offset` - The region-local offset to convert
    ///
    /// # Returns
    ///
    /// `Some(address)` if the offset is valid for this region, `None` otherwise
    pub fn offset_to_address(&self, offset: usize) -> Option<usize> {
        if offset < self.size {
            Some(self.start + offset)
        } else {
            None
        }
    }
}

/// Memory with defined regions for structured access and memory protection.
///
/// SegmentedMemory implements a memory model with distinct regions, each
/// having specific access permissions. This structure enables memory protection
/// and structured memory access patterns for the VM.
pub struct SegmentedMemory {
    /// The actual memory storage (array of i32 values)
    data: Vec<i32>,
    /// Mapping from region types to region definitions
    regions: HashMap<MemoryRegionType, MemoryRegion>,
    /// Lookup table for quick address-to-region resolution
    region_map: Vec<Option<MemoryRegionType>>,
}

impl SegmentedMemory {
    /// Creates a new segmented memory with the specified size.
    ///
    /// # Arguments
    ///
    /// * `initial_size` - The initial size of the memory in words
    ///
    /// # Returns
    ///
    /// A new `SegmentedMemory` instance with no defined regions
    pub fn new(initial_size: usize) -> Self {
        Self {
            data: vec![0; initial_size],
            regions: HashMap::new(),
            region_map: vec![None; initial_size],
        }
    }
    
    /// Defines a new memory region within this memory.
    ///
    /// This method adds a new region to the memory, ensuring it doesn't
    /// overlap with existing regions and fits within the memory size.
    ///
    /// # Arguments
    ///
    /// * `region` - The memory region to define
    ///
    /// # Returns
    ///
    /// `Ok(())` if the region was successfully defined, `Err` with a
    /// description of the issue otherwise
    pub fn define_region(&mut self, region: MemoryRegion) -> Result<(), String> {
        // Prüfen, ob die Region in den Speicher passt
        let end = region.start + region.size;
        if end > self.data.len() {
            return Err(format!(
                "Region {} would exceed memory size ({} > {})",
                region.region_type, end, self.data.len()
            ));
        }
        
        // Prüfen auf Überlappungen mit existierenden Regionen
        for (_, existing) in &self.regions {
            let existing_range = existing.address_range();
            let new_range = region.address_range();
            
            if existing_range.start < new_range.end && new_range.start < existing_range.end {
                return Err(format!(
                    "Region {} overlaps with existing region {}",
                    region.region_type, existing.region_type
                ));
            }
        }
        
        // Region im Mapping markieren
        for addr in region.address_range() {
            self.region_map[addr] = Some(region.region_type);
        }
        
        // Region speichern
        self.regions.insert(region.region_type, region);
        Ok(())
    }
    
    /// Reads a value from a specific region using an offset.
    ///
    /// # Arguments
    ///
    /// * `region_type` - The type of region to read from
    /// * `offset` - The offset within the region
    ///
    /// # Returns
    ///
    /// `Ok(value)` if the read was successful, `Err` with a
    /// description of the issue otherwise
    pub fn read_from_region(
        &self, 
        region_type: MemoryRegionType, 
        offset: usize
    ) -> Result<i32, String> {
        let region = self.get_region(region_type)?;
        
        // Prüfen, ob die Region Leserecht hat
        if !region.has_permission(AccessPermission::Read) {
            return Err(format!("Region {} is not readable", region_type));
        }
        
        // Offset in globale Adresse umwandeln
        let address = region.offset_to_address(offset)
            .ok_or_else(|| format!("Offset {} is out of bounds for region {}", offset, region_type))?;
        
        // Wert lesen
        Ok(self.data[address])
    }
    
    /// Writes a value to a specific region using an offset.
    ///
    /// # Arguments
    ///
    /// * `region_type` - The type of region to write to
    /// * `offset` - The offset within the region
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// `Ok(())` if the write was successful, `Err` with a
    /// description of the issue otherwise
    pub fn write_to_region(
        &mut self, 
        region_type: MemoryRegionType, 
        offset: usize, 
        value: i32
    ) -> Result<(), String> {
        let region = self.get_region(region_type)?;
        
        // Prüfen, ob die Region Schreibrecht hat
        if !region.has_permission(AccessPermission::Write) {
            return Err(format!("Region {} is not writable", region_type));
        }
        
        // Offset in globale Adresse umwandeln
        let address = region.offset_to_address(offset)
            .ok_or_else(|| format!("Offset {} is out of bounds for region {}", offset, region_type))?;
        
        // Wert schreiben
        self.data[address] = value;
        Ok(())
    }
    
    /// Reads a value from an absolute memory address.
    ///
    /// This method respects memory regions and their access permissions.
    ///
    /// # Arguments
    ///
    /// * `address` - The absolute address to read from
    ///
    /// # Returns
    ///
    /// `Ok(value)` if the read was successful, `Err` with a
    /// description of the issue otherwise
    pub fn read(&self, address: usize) -> Result<i32, String> {
        // Prüfen, ob die Adresse gültig ist
        if address >= self.data.len() {
            return Err(format!("Address {} is out of bounds", address));
        }
        
        // Ermitteln, in welcher Region die Adresse liegt
        let region_type = self.region_map[address]
            .ok_or_else(|| format!("Address {} is not in any defined region", address))?;
        
        let region = self.get_region(region_type)?;
        
        // Prüfen, ob die Region Leserecht hat
        if !region.has_permission(AccessPermission::Read) {
            return Err(format!("Region {} is not readable", region_type));
        }
        
        // Wert lesen
        Ok(self.data[address])
    }
    
    /// Writes a value to an absolute memory address.
    ///
    /// This method respects memory regions and their access permissions.
    ///
    /// # Arguments
    ///
    /// * `address` - The absolute address to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// `Ok(())` if the write was successful, `Err` with a
    /// description of the issue otherwise
    pub fn write(&mut self, address: usize, value: i32) -> Result<(), String> {
        // Prüfen, ob die Adresse gültig ist
        if address >= self.data.len() {
            return Err(format!("Address {} is out of bounds", address));
        }
        
        // Ermitteln, in welcher Region die Adresse liegt
        let region_type = self.region_map[address]
            .ok_or_else(|| format!("Address {} is not in any defined region", address))?;
        
        let region = self.get_region(region_type)?;
        
        // Prüfen, ob die Region Schreibrecht hat
        if !region.has_permission(AccessPermission::Write) {
            return Err(format!("Region {} is not writable", region_type));
        }
        
        // Wert schreiben
        self.data[address] = value;
        Ok(())
    }
    
    /// Executes code at a specific memory address.
    ///
    /// This method checks if the memory at the given address
    /// is executable and returns the instruction value if it is.
    ///
    /// # Arguments
    ///
    /// * `address` - The absolute address to execute from
    ///
    /// # Returns
    ///
    /// `Ok(instruction)` if executable, `Err` with a
    /// description of the issue otherwise
    pub fn execute(&self, address: usize) -> Result<i32, String> {
        // Prüfen, ob die Adresse gültig ist
        if address >= self.data.len() {
            return Err(format!("Address {} is out of bounds", address));
        }
        
        // Ermitteln, in welcher Region die Adresse liegt
        let region_type = self.region_map[address]
            .ok_or_else(|| format!("Address {} is not in any defined region", address))?;
        
        let region = self.get_region(region_type)?;
        
        // Prüfen, ob die Region Ausführungsrecht hat
        if !region.has_permission(AccessPermission::Execute) {
            return Err(format!("Region {} is not executable", region_type));
        }
        
        // "Code" laden (hier einfach den Wert zurückgeben)
        Ok(self.data[address])
    }
    
    /// Helper method to get a region by its type.
    ///
    /// # Arguments
    ///
    /// * `region_type` - The type of region to retrieve
    ///
    /// # Returns
    ///
    /// `Ok(region)` if the region exists, `Err` with a
    /// description of the issue otherwise
    fn get_region(&self, region_type: MemoryRegionType) -> Result<&MemoryRegion, String> {
        self.regions.get(&region_type)
            .ok_or_else(|| format!("Region {} is not defined", region_type))
    }
    
    /// Returns a string representation of the memory map.
    ///
    /// This method generates a human-readable description of all defined
    /// memory regions, including their addresses, sizes, permissions,
    /// and descriptions.
    ///
    /// # Returns
    ///
    /// A formatted string describing the memory map
    pub fn memory_map(&self) -> String {
        let mut result = String::new();
        result.push_str("Memory Map:\n");
        
        // Regionen nach Startadresse sortieren
        let mut regions: Vec<&MemoryRegion> = self.regions.values().collect();
        regions.sort_by_key(|r| r.start);
        
        for region in regions {
            let permissions = if region.has_permission(AccessPermission::Read) { "R" } else { "-" };
            let permissions = format!("{}{}{}", 
                permissions,
                if region.has_permission(AccessPermission::Write) { "W" } else { "-" },
                if region.has_permission(AccessPermission::Execute) { "X" } else { "-" }
            );
            
            result.push_str(&format!(
                "{:8} - {:8} ({:5} words): {} [{}] - {}\n",
                region.start,
                region.start + region.size - 1,
                region.size,
                region.region_type,
                permissions,
                region.description
            ));
        }
        
        result
    }
    
    /// Returns the total size of the memory in words.
    ///
    /// # Returns
    ///
    /// The size of the memory
    pub fn size(&self) -> usize {
        self.data.len()
    }
    
    /// Resizes the memory to a new size.
    ///
    /// This method can only increase the memory size, not decrease it.
    ///
    /// # Arguments
    ///
    /// * `new_size` - The new size for the memory
    ///
    /// # Returns
    ///
    /// `Ok(())` if resizing was successful, `Err` with a
    /// description of the issue otherwise
    pub fn resize(&mut self, new_size: usize) -> Result<(), String> {
        if new_size < self.data.len() {
            return Err("Cannot shrink memory below current size".to_string());
        }
        
        self.data.resize(new_size, 0);
        self.region_map.resize(new_size, None);
        Ok(())
    }
}


impl SegmentedMemory {
    /// Creates a standard memory layout for a general-purpose VM.
    ///
    /// This layout divides memory into the following regions:
    /// - Code (20%): For program instructions (Read + Execute)
    /// - Constants (10%): For read-only data (Read only)
    /// - Data (20%): For global variables (Read + Write)
    /// - Stack (25%): For call stack and local variables (Read + Write)
    /// - Heap (remainder): For dynamic allocations (Read + Write)
    ///
    /// # Arguments
    ///
    /// * `memory_size` - The total size of memory to allocate
    ///
    /// # Returns
    ///
    /// `Ok(memory)` with the configured memory layout, or `Err` with a 
    /// description if the memory size is too small
    pub fn create_standard_layout(memory_size: usize) -> Result<Self, String> {
        if memory_size < 1024 {
            return Err("Memory size too small for standard layout".to_string());
        }
        
        let mut memory = Self::new(memory_size);
        
        // Code-Region (20% des Speichers)
        let code_size = memory_size / 5;
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Code,
            0,
            code_size,
            vec![AccessPermission::Read, AccessPermission::Execute],
            Some("Program Code".to_string()),
        ))?;
        
        // Konstanten-Region (10% des Speichers)
        let const_size = memory_size / 10;
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Constants,
            code_size,
            const_size,
            vec![AccessPermission::Read],
            Some("Constants".to_string()),
        ))?;
        
        // Daten-Region (20% des Speichers)
        let data_size = memory_size / 5;
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Data,
            code_size + const_size,
            data_size,
            vec![AccessPermission::Read, AccessPermission::Write],
            Some("Global Data".to_string()),
        ))?;
        
        // Stack-Region (25% des Speichers)
        let stack_size = memory_size / 4;
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Stack,
            code_size + const_size + data_size,
            stack_size,
            vec![AccessPermission::Read, AccessPermission::Write],
            Some("Call Stack".to_string()),
        ))?;
        
        // Heap-Region (Rest des Speichers)
        let heap_start = code_size + const_size + data_size + stack_size;
        let heap_size = memory_size - heap_start;
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Heap,
            heap_start,
            heap_size,
            vec![AccessPermission::Read, AccessPermission::Write],
            Some("Dynamic Memory".to_string()),
        ))?;
        
        Ok(memory)
    }
    
    /// Creates a memory layout optimized for embedded systems.
    ///
    /// This layout is designed to mimic constraints of embedded hardware:
    /// - Flash memory (50%): Split between Code and Constants
    /// - RAM (40%): Split between Data and Stack
    /// - I/O (10%): For memory-mapped peripheral access
    ///
    /// # Arguments
    ///
    /// * `memory_size` - The total size of memory to allocate
    ///
    /// # Returns
    ///
    /// `Ok(memory)` with the configured embedded layout, or `Err` with a 
    /// description if the memory size is too small
    pub fn create_embedded_layout(memory_size: usize) -> Result<Self, String> {
        if memory_size < 512 {
            return Err("Memory size too small for embedded layout".to_string());
        }
        
        let mut memory = Self::new(memory_size);
        
        // Flash-Speicher (Code und Konstanten, 50%)
        let flash_size = memory_size / 2;
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Code,
            0,
            flash_size / 2,
            vec![AccessPermission::Read, AccessPermission::Execute],
            Some("Program Flash".to_string()),
        ))?;
        
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Constants,
            flash_size / 2,
            flash_size / 2,
            vec![AccessPermission::Read],
            Some("Constant Data".to_string()),
        ))?;
        
        // RAM (40%)
        let ram_size = (memory_size * 2) / 5;
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Data,
            flash_size,
            ram_size / 2,
            vec![AccessPermission::Read, AccessPermission::Write],
            Some("RAM Data".to_string()),
        ))?;
        
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Stack,
            flash_size + (ram_size / 2),
            ram_size / 2,
            vec![AccessPermission::Read, AccessPermission::Write],
            Some("Stack".to_string()),
        ))?;
        
        // I/O-Region (10%)
        let io_size = memory_size / 10;
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::IO,
            flash_size + ram_size,
            io_size,
            vec![AccessPermission::Read, AccessPermission::Write],
            Some("Memory-mapped I/O".to_string()),
        ))?;
        
        Ok(memory)
    }

    /// Prints the memory map to standard output.
    ///
    /// This is a convenience method for debugging and displaying
    /// the current memory configuration.
    pub fn print_memory_map(&self) {
        println!("{}", self.memory_map());
    }

    /// Creates a memory layout specifically for testing purposes.
    ///
    /// This layout uses the same proportions as the standard layout,
    /// but grants all access permissions (Read, Write, Execute) to all
    /// regions, making it easier to use in tests without permission errors.
    ///
    /// # Arguments
    ///
    /// * `memory_size` - The total size of memory to allocate
    ///
    /// # Returns
    ///
    /// `Ok(memory)` with the configured test layout, or `Err` with a 
    /// description if the memory size is too small
    pub fn create_test_layout(memory_size: usize) -> Result<Self, String> {
        if memory_size < 512 {
            return Err("Memory size too small for test layout".to_string());
        }
        
        let mut memory = Self::new(memory_size);
        
        // Code-Region (20% des Speichers)
        let code_size = memory_size / 5;
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Code,
            0,
            code_size,
            vec![AccessPermission::Read, AccessPermission::Execute, AccessPermission::Write],
            Some("Program Code".to_string()),
        ))?;
        
        // Konstanten-Region (10% des Speichers)
        let const_size = memory_size / 10;
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Constants,
            code_size,
            const_size,
            vec![AccessPermission::Read, AccessPermission::Write, AccessPermission::Execute],
            Some("Constants".to_string()),
        ))?;
        
        // Daten-Region (20% des Speichers)
        let data_size = memory_size / 5;
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Data,
            code_size + const_size,
            data_size,
            vec![AccessPermission::Read, AccessPermission::Write, AccessPermission::Execute],
            Some("Global Data".to_string()),
        ))?;
        
        // Stack-Region (25% des Speichers)
        let stack_size = memory_size / 4;
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Stack,
            code_size + const_size + data_size,
            stack_size,
            vec![AccessPermission::Read, AccessPermission::Write, AccessPermission::Execute],
            Some("Call Stack".to_string()),
        ))?;
        
        // Heap-Region (Rest des Speichers)
        let heap_start = code_size + const_size + data_size + stack_size;
        let heap_size = memory_size - heap_start;
        memory.define_region(MemoryRegion::new(
            MemoryRegionType::Heap,
            heap_start,
            heap_size,
            vec![AccessPermission::Read, AccessPermission::Write, AccessPermission::Execute],
            Some("Dynamic Memory".to_string()),
        ))?;
        
        Ok(memory)
    }
}

    #[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segmented_memory_basic() {
        let mut memory = SegmentedMemory::new(1000);
        
        // Code-Region definieren mit Schreibrecht für den Test
        let code_region = MemoryRegion::new(
            MemoryRegionType::Code,
            0,
            200,
            vec![AccessPermission::Read, AccessPermission::Execute, AccessPermission::Write],
            Some("Test Code".to_string()),
        );
        memory.define_region(code_region).unwrap();
        
        // Daten-Region definieren
        let data_region = MemoryRegion::new(
            MemoryRegionType::Data,
            200,
            300,
            vec![AccessPermission::Read, AccessPermission::Write],
            Some("Test Data".to_string()),
        );
        memory.define_region(data_region).unwrap();
        
        // In die Datenregion schreiben
        memory.write_to_region(MemoryRegionType::Data, 50, 42).unwrap();
        
        // Aus der Datenregion lesen
        let value = memory.read_from_region(MemoryRegionType::Data, 50).unwrap();
        assert_eq!(value, 42);
        
        // Absolute Adresse
        let addr = 200 + 50; // Start der Datenregion + Offset
        let value = memory.read(addr).unwrap();
        assert_eq!(value, 42);
        
        // In die Code-Region schreiben (für Test-Zwecke)
        memory.write_to_region(MemoryRegionType::Code, 10, 99).unwrap();
        
        // Ausführen von Code
        let instruction = memory.execute(10).unwrap();
        assert_eq!(instruction, 99);
    }
    
    #[test]
    fn test_segmented_memory_permissions() {
        let mut memory = SegmentedMemory::new(1000);
        
        // Konstanten-Region (nur lesbar)
        let const_region = MemoryRegion::new(
            MemoryRegionType::Constants,
            0,
            100,
            vec![AccessPermission::Read],
            None,
        );
        memory.define_region(const_region).unwrap();
        
        // Schreib-Initialisierung (intern, vor der Anwendung von Berechtigungen)
        memory.data[50] = 123;
        
        // Lesen sollte funktionieren
        let value = memory.read_from_region(MemoryRegionType::Constants, 50).unwrap();
        assert_eq!(value, 123);
        
        // Schreiben sollte fehlschlagen
        let result = memory.write_to_region(MemoryRegionType::Constants, 50, 456);
        assert!(result.is_err());
        
        // Ausführen sollte fehlschlagen
        let result = memory.execute(50);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_standard_layout() {
        let memory = SegmentedMemory::create_standard_layout(10000).unwrap();
        
        // Prüfen, ob alle erwarteten Regionen vorhanden sind
        assert!(memory.regions.contains_key(&MemoryRegionType::Code));
        assert!(memory.regions.contains_key(&MemoryRegionType::Constants));
        assert!(memory.regions.contains_key(&MemoryRegionType::Data));
        assert!(memory.regions.contains_key(&MemoryRegionType::Stack));
        assert!(memory.regions.contains_key(&MemoryRegionType::Heap));
        
        // Memory Map ausgeben (für Debug-Zwecke)
        println!("{}", memory.memory_map());
    }
}