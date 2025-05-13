use core::fmt;
use std::{collections::HashMap, ops::Range};


/// Typen von Speicherregionen, die in der VM unterstützt werden
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryRegionType {
    Code,       // Für Programmcode/Instruktionen
    Data,       // Für statische Daten
    Stack,      // Für den Aufrufstack
    Heap,       // Für dynamisch allozierte Daten
    IO,         // Für memory-mapped I/O
    Constants,  // Für Konstanten (schreibgeschützt)
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

/// Zugriffsrechte für Speicherregionen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPermission {
    Read,
    Write,
    Execute,
}

/// Repräsentiert eine zusammenhängende Region im Speicher
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub region_type: MemoryRegionType,
    pub start: usize,               // Startadresse
    pub size: usize,                // Größe in Elementen
    pub permissions: Vec<AccessPermission>, // Zugriffsrechte
    pub description: String,        // Optionale Beschreibung
}

impl MemoryRegion {
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
    
    /// Gibt den Adressbereich der Region zurück
    pub fn address_range(&self) -> Range<usize> {
        self.start..(self.start + self.size)
    }
    
    /// Prüft, ob eine Adresse in dieser Region liegt
    pub fn contains_address(&self, address: usize) -> bool {
        self.address_range().contains(&address)
    }
    
    /// Prüft, ob eine Zugriffsberechtigung besteht
    pub fn has_permission(&self, permission: AccessPermission) -> bool {
        self.permissions.contains(&permission)
    }
    
    /// Konvertiert eine globale Adresse in einen regionslokalen Offset
    pub fn address_to_offset(&self, address: usize) -> Option<usize> {
        if self.contains_address(address) {
            Some(address - self.start)
        } else {
            None
        }
    }
    
    /// Konvertiert einen regionslokalen Offset in eine globale Adresse
    pub fn offset_to_address(&self, offset: usize) -> Option<usize> {
        if offset < self.size {
            Some(self.start + offset)
        } else {
            None
        }
    }
}

/// Speicher mit definierten Regionen für strukturierten Zugriff
pub struct SegmentedMemory {
    data: Vec<i32>,                                // Unterlegende Speicherdaten
    regions: HashMap<MemoryRegionType, MemoryRegion>, // Region-Definitionen
    region_map: Vec<Option<MemoryRegionType>>,     // Lookup-Tabelle für Adressen
}

impl SegmentedMemory {
    /// Erstellt einen neuen segmentierten Speicher
    pub fn new(initial_size: usize) -> Self {
        Self {
            data: vec![0; initial_size],
            regions: HashMap::new(),
            region_map: vec![None; initial_size],
        }
    }
    
    /// Definiert eine neue Speicherregion
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
    
    /// Liest einen Wert aus einer bestimmten Region (per Offset)
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
    
    /// Schreibt einen Wert in eine bestimmte Region (per Offset)
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
    
    /// Liest einen Wert aus einer absoluten Adresse
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
    
    /// Schreibt einen Wert an eine absolute Adresse
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
    
    /// Führt Code an einer Adresse aus
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
    
    /// Hilfsmethode: Region abrufen
    fn get_region(&self, region_type: MemoryRegionType) -> Result<&MemoryRegion, String> {
        self.regions.get(&region_type)
            .ok_or_else(|| format!("Region {} is not defined", region_type))
    }
    
    /// Gibt Informationen zur Speicheraufteilung zurück
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
    
    /// Gibt die Größe des Speichers zurück
    pub fn size(&self) -> usize {
        self.data.len()
    }
    
    /// Erweitert den Speicher auf die angegebene Größe
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
    /// Erstellt ein Standard-Speicherlayout für eine einfache VM
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
    
    /// Erstellt ein Speicherlayout für eine eingebettete VM
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

    pub fn print_memory_map(&self) {
        println!("{}", self.memory_map());
    }

    /// Erstellt ein Speicherlayout für Tests, bei dem alle Regionen alle Zugriffsrechte haben
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