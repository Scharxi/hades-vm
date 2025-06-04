use std::fs;
use std::path::Path;
use nyx_compiler::{HexCompiler, compile_source_to_hex_file};
use hades_ir::{hex_loader::{HexLoader, load_and_run_hex}, HadesExecutable, SectionType};

#[derive(Debug)]
pub enum HexToolError {
    IoError(std::io::Error),
    CompileError(String),
    LoaderError(String),
}

impl std::fmt::Display for HexToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HexToolError::IoError(e) => write!(f, "I/O error: {}", e),
            HexToolError::CompileError(e) => write!(f, "Compile error: {}", e),
            HexToolError::LoaderError(e) => write!(f, "Loader error: {}", e),
        }
    }
}

impl std::error::Error for HexToolError {}

impl From<std::io::Error> for HexToolError {
    fn from(err: std::io::Error) -> Self {
        HexToolError::IoError(err)
    }
}

/// Compile Nyx source code to a HEX executable
pub fn compile_to_hex<P: AsRef<Path>>(
    source_path: P,
    output_path: P,
    debug: bool,
) -> Result<(), HexToolError> {
    let source = fs::read_to_string(&source_path)?;
    
    if debug {
        println!("📖 Reading source file: {}", source_path.as_ref().display());
        println!("📄 Source code:");
        println!("{}", source);
        println!("{}", "─".repeat(50));
    }
    
    compile_source_to_hex_file(&source, output_path, debug)
        .map_err(|e| HexToolError::CompileError(e.to_string()))?;
    
    Ok(())
}

/// Run a HEX executable file
pub fn run_hex_file<P: AsRef<Path>>(
    hex_path: P,
    debug: bool,
) -> Result<(), HexToolError> {
    let hex_data = fs::read(&hex_path)?;
    
    if debug {
        println!("📖 Loading HEX file: {}", hex_path.as_ref().display());
        println!("📊 File size: {} bytes", hex_data.len());
    }
    
    load_and_run_hex(&hex_data, debug)
        .map_err(|e| HexToolError::LoaderError(e.to_string()))?;
    
    Ok(())
}

/// Compile and run Nyx source code in one step
pub fn compile_and_run<P: AsRef<Path>>(
    source_path: P,
    debug: bool,
) -> Result<(), HexToolError> {
    let source = fs::read_to_string(&source_path)?;
    
    if debug {
        println!("🔧 Compiling and running: {}", source_path.as_ref().display());
        println!("📄 Source code:");
        println!("{}", source);
        println!("{}", "─".repeat(50));
    }
    
    // Compile to temporary buffer
    let mut compiler = HexCompiler::new(debug)
        .map_err(|e| HexToolError::CompileError(e.to_string()))?;
    
    // Parse the source
    let mut parser = nyx_compiler::Parser::new(&source);
    let program = parser.parse_program()
        .map_err(|e| HexToolError::CompileError(format!("Parse error: {:?}", e)))?;
    
    // Compile to HEX
    let executable = compiler.compile_to_hex(&program)
        .map_err(|e| HexToolError::CompileError(e.to_string()))?;
    
    if debug {
        println!("✅ Compilation successful!");
        println!("📊 Generated executable with {} sections", executable.sections.len());
        println!("🎯 Entry point: 0x{:08x}", executable.header.entry_point);
    }
    
    // Serialize to bytes
    let mut exec_clone = executable.clone();
    let mut hex_data = Vec::new();
    exec_clone.write_to(&mut hex_data)
        .map_err(|e| HexToolError::CompileError(e.to_string()))?;
    
    if debug {
        println!("💾 Serialized executable: {} bytes", hex_data.len());
        println!("{}", "─".repeat(50));
        println!("🚀 Running executable...");
    }
    
    // Run the executable
    load_and_run_hex(&hex_data, debug)
        .map_err(|e| HexToolError::LoaderError(e.to_string()))?;
    
    Ok(())
}

/// Display information about a HEX file
pub fn hex_info<P: AsRef<Path>>(
    hex_path: P,
) -> Result<(), HexToolError> {
    let hex_data = fs::read(&hex_path)?;
    
    let mut loader = HexLoader::new();
    loader.load_from_bytes(&hex_data)
        .map_err(|e| HexToolError::LoaderError(e.to_string()))?;
    
    // Parse the executable to get detailed info
    let mut cursor = std::io::Cursor::new(&hex_data);
    let executable = HadesExecutable::read_from(&mut cursor)
        .map_err(|e| HexToolError::LoaderError(e.to_string()))?;
    
    println!("📋 HEX File Information");
    println!("═══════════════════════");
    println!("📁 File: {}", hex_path.as_ref().display());
    println!("📊 Size: {} bytes", hex_data.len());
    println!();
    
    println!("📋 Header Information");
    println!("─────────────────────");
    println!("🔮 Magic: 0x{:08x}", executable.header.magic);
    println!("📦 Version: {}", executable.header.version);
    println!("🏗️  Architecture: 0x{:04x}", executable.header.arch);
    println!("🎯 Entry Point: 0x{:08x}", executable.header.entry_point);
    println!("📑 Sections: {}", executable.header.section_count);
    println!();
    
    println!("📋 Sections");
    println!("───────────");
    for (i, section) in executable.sections.iter().enumerate() {
        let section_type_name = match section.header.section_type {
            SectionType::Null => "NULL",
            SectionType::Code => "CODE",
            SectionType::Data => "DATA",
            SectionType::Constants => "CONSTANTS",
            SectionType::Symbols => "SYMBOLS",
            SectionType::Strings => "STRINGS",
            SectionType::Relocations => "RELOCATIONS",
            SectionType::Debug => "DEBUG",
        };
        
        println!("  {}. {} ({:?})", i + 1, section_type_name, section.header.section_type);
        println!("     Virtual Address: 0x{:08x}", section.header.virtual_address);
        println!("     File Offset: 0x{:08x}", section.header.file_offset);
        println!("     Size: {} bytes", section.header.file_size);
        
        if section.header.section_type == SectionType::Code && !section.data.is_empty() {
            println!("     Bytecode preview:");
            let preview_len = std::cmp::min(16, section.data.len());
            print!("     ");
            for (j, &byte) in section.data[..preview_len].iter().enumerate() {
                if j > 0 && j % 4 == 0 {
                    print!(" ");
                }
                print!("{:02x}", byte);
            }
            if section.data.len() > preview_len {
                print!("...");
            }
            println!();
        }
        println!();
    }
    
    if !executable.symbols.is_empty() {
        println!("📋 Symbols");
        println!("──────────");
        for symbol in &executable.symbols {
            println!("  Symbol: offset {}", symbol.name_offset);
            println!("    Value: 0x{:08x}", symbol.value);
            println!("    Size: {} bytes", symbol.size);
            println!("    Section: {}", symbol.section_index);
            println!();
        }
    }
    
    Ok(())
}

/// Create a test HEX file
pub fn create_test_hex<P: AsRef<Path>>(
    output_path: P,
) -> Result<(), HexToolError> {
    let mut executable = HadesExecutable::new(0x1000);
    
    // Create a simple test program
    let code = vec![
        0x00, 0x00, 0x2A, 0x04, // LoadConstant 42
        0x00, 0x00, 0x00, 0x06, // Print
        0x00, 0x00, 0x00, 0x0E, // Return
    ];
    
    executable.add_section(".text", SectionType::Code, code);
    executable.add_symbol("main", 0x1000, 12, 0);
    
    // Write to file
    let mut file = fs::File::create(&output_path)?;
    executable.write_to(&mut file)
        .map_err(|e| HexToolError::CompileError(e.to_string()))?;
    
    println!("✅ Created test HEX file: {}", output_path.as_ref().display());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_info() {
        let temp_dir = tempdir().unwrap();
        let hex_path = temp_dir.path().join("test.hex");
        
        // Create test HEX file
        create_test_hex(&hex_path).unwrap();
        
        // Verify file exists
        assert!(hex_path.exists());
        
        // Test info display
        hex_info(&hex_path).unwrap();
    }
} 