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
    
    /// Allocates a new block of memory of the specified size.
    ///
    /// This method searches for a free space in the heap region that can accommodate
    /// the requested size. It implements a simple first-fit allocation strategy and
    /// tracks allocations in memory to allow multiple allocations and reuse of freed memory.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the memory block to allocate in words
    ///
    /// # Returns
    ///
    /// `Some(address)` if allocation was successful, `None` if there is not enough space
    pub fn allocate(&mut self, size: usize) -> Option<usize> {
        if size == 0 {
            return None; // Can't allocate zero bytes
        }

        // Get the heap region
        let heap_region = match self.regions.get(&MemoryRegionType::Heap) {
            Some(region) => region,
            None => return None, // No heap region defined
        };
        
        // Check if the heap has write permissions
        if !heap_region.has_permission(AccessPermission::Write) {
            return None; // Heap is not writable
        }
        
        // For this implementation, we'll track our allocations by using the first 
        // few words of the heap itself as an allocation bitmap
        let heap_start = heap_region.start;
        let heap_size = heap_region.size;
        
        // The first word contains our allocation bitmap version (to detect initialization)
        // We use 0xALLOC as our bitmap marker
        const ALLOC_MARKER: i32 = 0xA110C;
        
        // Check if our allocation bitmap is initialized
        let bitmap_initialized = match self.read(heap_start) {
            Ok(marker) => marker == ALLOC_MARKER,
            Err(_) => false,
        };
        
        // If not initialized, set up our allocation bitmap
        if !bitmap_initialized {
            // Write our marker
            if let Err(_err) = self.write(heap_start, ALLOC_MARKER) {
                return None; // Can't initialize bitmap
            }
            
            // Reserve space for the bitmap itself: 1 word for marker + 1 word for allocation count
            // The rest of the bitmap will grow as needed
            const BITMAP_HEADER_SIZE: usize = 2;
            
            // Mark all memory as free in a single large block
            // Store the first free block's size right after the bitmap header
            if let Err(_err) = self.write(heap_start + BITMAP_HEADER_SIZE, (heap_size - BITMAP_HEADER_SIZE - 1) as i32) {
                return None; // Can't initialize free list
            }
            
            // Write 0 to indicate this block is free
            if let Err(_err) = self.write(heap_start + BITMAP_HEADER_SIZE + 1, 0) {
                return None;
            }
            
            // Store the number of blocks (just 1 initially)
            if let Err(_err) = self.write(heap_start + 1, 1) {
                return None;
            }
        }
        
        // Read the number of blocks
        let block_count = match self.read(heap_start + 1) {
            Ok(count) => count as usize,
            Err(_) => return None,
        };
        
        // The bitmap header size: marker + count
        const BITMAP_HEADER_SIZE: usize = 2;
        
        // Each block entry in the bitmap uses 3 words:
        // 1. Block start address
        // 2. Block size
        // 3. Allocation flag (0 = free, 1 = allocated)
        const BLOCK_ENTRY_SIZE: usize = 3;
        
        // Scan through the blocks to find a suitable free block using first-fit
        for i in 0..block_count {
            let block_entry_offset = BITMAP_HEADER_SIZE + i * BLOCK_ENTRY_SIZE;
            
            // Read block information
            let block_addr = match self.read(heap_start + block_entry_offset) {
                Ok(addr) => addr as usize,
                Err(_) => continue, // Skip this block if we can't read it
            };
            
            let block_size = match self.read(heap_start + block_entry_offset + 1) {
                Ok(size) => size as usize,
                Err(_) => continue,
            };
            
            let is_allocated = match self.read(heap_start + block_entry_offset + 2) {
                Ok(flag) => flag != 0,
                Err(_) => continue,
            };
            
            // If the block is free and large enough, use it
            if !is_allocated && block_size >= size {
                // If the block is significantly larger than needed, split it
                if block_size > size + BLOCK_ENTRY_SIZE + 1 {
                    // Update the current block's size
                    if let Err(_) = self.write(heap_start + block_entry_offset + 1, size as i32) {
                        continue;
                    }
                    
                    // Mark the current block as allocated
                    if let Err(_) = self.write(heap_start + block_entry_offset + 2, 1) {
                        continue;
                    }
                    
                    // Create a new block entry for the remainder
                    let new_block_addr = block_addr + size;
                    let new_block_size = block_size - size;
                    
                    // Write the new block's information
                    let new_block_offset = BITMAP_HEADER_SIZE + block_count * BLOCK_ENTRY_SIZE;
                    
                    if let Err(_) = self.write(heap_start + new_block_offset, new_block_addr as i32) {
                        // Just use the whole block if we can't split it
                        return Some(block_addr);
                    }
                    
                    if let Err(_) = self.write(heap_start + new_block_offset + 1, new_block_size as i32) {
                        return Some(block_addr);
                    }
                    
                    if let Err(_) = self.write(heap_start + new_block_offset + 2, 0) {
                        return Some(block_addr);
                    }
                    
                    // Update the block count
                    if let Err(_) = self.write(heap_start + 1, (block_count + 1) as i32) {
                        return Some(block_addr);
                    }
                    
                    return Some(block_addr);
                } else {
                    // Just mark the whole block as allocated
                    if let Err(_) = self.write(heap_start + block_entry_offset + 2, 1) {
                        continue;
                    }
                    
                    return Some(block_addr);
                }
            }
        }
        
        // If we get here, no suitable block was found
        // Try to allocate from the end of the bitmap
        let bitmap_size = BITMAP_HEADER_SIZE + block_count * BLOCK_ENTRY_SIZE;
        
        // Check if we have enough space for the allocation
        if heap_size > bitmap_size + size + BLOCK_ENTRY_SIZE {
            let available_size = heap_size - bitmap_size - size;
            
            // Make sure we have enough space for both the bitmap expansion and the allocation
            if available_size >= BLOCK_ENTRY_SIZE {
                // Create a new block entry
                let new_block_addr = heap_start + bitmap_size;
                
                // Write the new block's information
                if let Err(_) = self.write(new_block_addr, (new_block_addr + BLOCK_ENTRY_SIZE) as i32) {
                    return None;
                }
                
                if let Err(_) = self.write(new_block_addr + 1, size as i32) {
                    return None;
                }
                
                if let Err(_) = self.write(new_block_addr + 2, 1) { // Mark as allocated
                    return None;
                }
                
                // Update the block count
                if let Err(_) = self.write(heap_start + 1, (block_count + 1) as i32) {
                    return None;
                }
                
                return Some(new_block_addr + BLOCK_ENTRY_SIZE);
            }
        }
        
        // No suitable block found and no space to create a new one
        None
    }
    
    /// Deallocates a previously allocated memory block.
    ///
    /// This method marks a previously allocated block as free in the heap's allocation bitmap.
    /// It finds the block by its address and, if found and marked as allocated, sets it to free.
    /// This operation does not actually clear the memory contents; it just makes the space
    /// available for future allocations.
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the memory block to deallocate
    ///
    /// # Returns
    ///
    /// `true` if deallocation was successful, `false` if the address was not allocated
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use vm::memory::{SegmentedMemory, MemoryRegionType};
    /// # let mut memory = SegmentedMemory::create_test_layout(1000).unwrap();
    /// // Allocate some memory
    /// let address = memory.allocate(100).unwrap();
    ///
    /// // Use the memory...
    ///
    /// // Free it when done
    /// let success = memory.deallocate(address);
    /// assert!(success);
    /// ```
    pub fn deallocate(&mut self, address: usize) -> bool {
        // Get the heap region
        let heap_region = match self.regions.get(&MemoryRegionType::Heap) {
            Some(region) => region,
            None => return false, // No heap region defined
        };
        
        let heap_start = heap_region.start;
        
        // Check if our allocation bitmap is initialized
        const ALLOC_MARKER: i32 = 0xA110C;
        let bitmap_initialized = match self.read(heap_start) {
            Ok(marker) => marker == ALLOC_MARKER,
            Err(_) => false,
        };
        
        if !bitmap_initialized {
            return false; // Bitmap not initialized, nothing to deallocate
        }
        
        // Read the number of blocks
        let block_count = match self.read(heap_start + 1) {
            Ok(count) => count as usize,
            Err(_) => return false,
        };
        
        // The bitmap header size: marker + count
        const BITMAP_HEADER_SIZE: usize = 2;
        
        // Each block entry in the bitmap uses 3 words:
        // 1. Block start address
        // 2. Block size
        // 3. Allocation flag (0 = free, 1 = allocated)
        const BLOCK_ENTRY_SIZE: usize = 3;
        
        // Scan through the blocks to find the one with the given address
        for i in 0..block_count {
            let block_entry_offset = BITMAP_HEADER_SIZE + i * BLOCK_ENTRY_SIZE;
            
            // Read block information
            let block_addr = match self.read(heap_start + block_entry_offset) {
                Ok(addr) => addr as usize,
                Err(_) => continue,
            };
            
            // Check if this is the block we're looking for
            if block_addr == address {
                // Check if it's allocated
                let is_allocated = match self.read(heap_start + block_entry_offset + 2) {
                    Ok(flag) => flag != 0,
                    Err(_) => continue,
                };
                
                if is_allocated {
                    // Mark the block as free
                    if let Err(_) = self.write(heap_start + block_entry_offset + 2, 0) {
                        return false;
                    }
                    
                    return true;
                }
            }
        }
        
        // Block not found or not allocated
        false
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

    #[test]
    fn test_memory_allocation() {
        // Create a memory with a test layout that includes a heap region
        let mut memory = SegmentedMemory::create_test_layout(1000).unwrap();
        
        // Get the heap region to check its starting address for validation
        // We need to clone the heap region to avoid borrowing issues
        let heap_region = memory.regions.get(&MemoryRegionType::Heap).unwrap().clone();
        
        // Try to allocate some memory
        let allocation = memory.allocate(100);
        assert!(allocation.is_some());
        
        // The first allocation should be at the start of the heap plus some metadata overhead
        let expected_start = heap_region.start + 2; // Bitmap marker + count
        assert!(allocation.unwrap() >= expected_start);
        
        // Try to allocate too much memory
        let too_large = memory.allocate(2000);
        assert!(too_large.is_none());
    }

    #[test]
    fn test_memory_allocation_and_deallocation() {
        // Create a memory with a test layout that includes a heap region
        let mut memory = SegmentedMemory::create_test_layout(1000).unwrap();
        
        // Allocate memory blocks of different sizes
        let addr1 = memory.allocate(50).unwrap(); // First allocation
        let addr2 = memory.allocate(25).unwrap(); // Second allocation
        let addr3 = memory.allocate(10).unwrap(); // Third allocation
        
        // Verify all allocations succeeded
        assert!(addr1 != 0);
        assert!(addr2 != 0);
        assert!(addr3 != 0);
        
        // They should be different addresses
        assert_ne!(addr1, addr2);
        assert_ne!(addr1, addr3);
        assert_ne!(addr2, addr3);
        
        // Deallocate the middle block
        let success = memory.deallocate(addr2);
        assert!(success);
        
        // Try to deallocate the same block again (should fail)
        let failed = memory.deallocate(addr2);
        assert!(!failed);
        
        // Try to allocate a block of size 20 - should fit in the freed space
        let addr4 = memory.allocate(20).unwrap();
        
        // Verify it worked
        assert!(addr4 != 0);
        assert_ne!(addr4, addr1);
        assert_ne!(addr4, addr3);
        
        // It might reuse the space from addr2, but we can't assume that
        // since the implementation might use different allocation strategies
        
        // Deallocate all remaining blocks
        assert!(memory.deallocate(addr1));
        assert!(memory.deallocate(addr3));
        assert!(memory.deallocate(addr4));
    }
}