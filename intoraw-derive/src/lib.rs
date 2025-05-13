use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Lit, Meta, NestedMeta};
use convert_case::{Case, Casing};

fn normalize_instruction_name(name: &str) -> String {
    // Entferne bekannte Präfixe und Suffixe
    let name = name.replace("Instruction", "");
    
    // Konvertiere zu PascalCase
    let pascal_case = name.to_case(Case::Pascal);
    
    // Liste aller bekannten Opcodes
    let opcodes = [
        "Add", "Store", "Sub", "LoadConstant", "Multiply", 
        "Print", "LoadMemory", "StoreMemory", "JumpIfZero"
    ];
    
    // Finde den Opcode im Namen
    for opcode in opcodes.iter() {
        if pascal_case.contains(opcode) {
            return opcode.to_string();
        }
    }
    
    // Wenn kein bekannter Opcode gefunden wurde, gib den bereinigten Namen zurück
    pascal_case
}

#[proc_macro_derive(IntoRaw, attributes(opcode))]
pub fn derive_into_raw(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    
    // Get the name of the struct
    let name = &input.ident;
    let struct_name = name.to_string();
    let normalized_name = normalize_instruction_name(&struct_name);
    
    // Try to get the opcode from attributes
    let mut opcode_value = None;
    
    // Look for #[opcode = X] attribute
    for attr in &input.attrs {
        if attr.path.is_ident("opcode") {
            match attr.parse_meta() {
                Ok(Meta::NameValue(meta)) => {
                    match meta.lit {
                        Lit::Int(lit) => {
                            // Handle decimal, hex, etc.
                            if let Ok(value) = lit.base10_parse::<u8>() {
                                opcode_value = Some(value);
                                break;
                            }
                        },
                        Lit::Str(lit_str) => {
                            // Parse string literals like "1"
                            if let Ok(value) = lit_str.value().parse::<u8>() {
                                opcode_value = Some(value);
                                break;
                            }
                        },
                        _ => {}
                    }
                },
                Ok(Meta::List(meta)) => {
                    for nested in meta.nested {
                        match nested {
                            NestedMeta::Lit(Lit::Int(lit)) => {
                                if let Ok(value) = lit.base10_parse::<u8>() {
                                    opcode_value = Some(value);
                                    break;
                                }
                            },
                            NestedMeta::Lit(Lit::Str(lit_str)) => {
                                if let Ok(value) = lit_str.value().parse::<u8>() {
                                    opcode_value = Some(value);
                                    break;
                                }
                            },
                            _ => {}
                        }
                    }
                },
                _ => {}
            }
        }
    }
    
    // If no opcode attribute, infer from the normalized name using the OpcodeMapping trait
    if opcode_value.is_none() {
        // Statische Zuordnung als Fallback
        opcode_value = match normalized_name.as_str() {
            "Add" => Some(0x01),
            "Store" => Some(0x02), 
            "Sub" => Some(0x03),
            "LoadConstant" => Some(0x04),
            "Multiply" => Some(0x05),
            "Print" => Some(0x06),
            "LoadMemory" => Some(0x07),
            "StoreMemory" => Some(0x08),
            "JumpIfZero" => Some(0x09),
            _ => None,
        };
    }

    // Check if we're dealing with a struct
    let is_struct = matches!(&input.data, Data::Struct(_));

    if !is_struct {
        return syn::Error::new_spanned(
            name,
            "IntoRaw can only be derived for structs"
        ).to_compile_error().into();
    }

    // Determine if the struct has a field that should be used as an operand
    let has_operand = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => !fields.named.is_empty(),
            Fields::Unnamed(fields) => !fields.unnamed.is_empty(),
            Fields::Unit => false,
        },
        _ => false, // This shouldn't happen due to the check above
    };
    
    // Generate the implementation
    let generated_code = if let Some(opcode) = opcode_value {
        if has_operand {
            // For instructions with operands (like Store)
            quote! {
                impl crate::instruction::IntoRaw for #name {
                    fn into_raw(self) -> crate::instruction::RawInstruction {
                        // Extract the operand from the first field
                        let operand = self.0;
                        let b0 = ((operand >> 16) & 0xFF) as u8;
                        let b1 = ((operand >> 8) & 0xFF) as u8;
                        let b2 = (operand & 0xFF) as u8;
                        
                        // Create instruction with the appropriate opcode
                        crate::instruction::RawInstruction::from_bytes(b0, b1, b2, #opcode)
                    }
                }
            }
        } else {
            // For instructions without operands (like Add)
            quote! {
                impl crate::instruction::IntoRaw for #name {
                    fn into_raw(self) -> crate::instruction::RawInstruction {
                        // Create instruction with only the opcode, no operands
                        crate::instruction::RawInstruction::from_bytes(0x00, 0x00, 0x00, #opcode)
                    }
                }
            }
        }
    } else {
        // If still no opcode, return an error with a more helpful message
        return syn::Error::new_spanned(
            name,
            format!("Unknown instruction type: {}. Add a #[opcode = X] attribute or implement IntoRaw manually.", name)
        ).to_compile_error().into();
    };
    
    // Return the generated implementation
    generated_code.into()
}
