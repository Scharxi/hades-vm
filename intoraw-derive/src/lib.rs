use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Lit, Meta, NestedMeta, LitStr};

#[proc_macro_derive(IntoRaw, attributes(opcode))]
pub fn derive_into_raw(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    
    // Get the name of the struct
    let name = &input.ident;
    
    // Try to get the opcode from attributes
    let mut opcode = None;
    
    // Look for #[opcode = X] attribute
    for attr in &input.attrs {
        if attr.path.is_ident("opcode") {
            match attr.parse_meta() {
                Ok(Meta::NameValue(meta)) => {
                    match meta.lit {
                        Lit::Int(lit) => {
                            if let Ok(value) = lit.base10_parse::<u8>() {
                                opcode = Some(value);
                                break;
                            }
                        },
                        Lit::Str(lit_str) => {
                            // Parse string literals like "1"
                            if let Ok(value) = lit_str.value().parse::<u8>() {
                                opcode = Some(value);
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
                                    opcode = Some(value);
                                    break;
                                }
                            },
                            NestedMeta::Lit(Lit::Str(lit_str)) => {
                                if let Ok(value) = lit_str.value().parse::<u8>() {
                                    opcode = Some(value);
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
    
    // If no opcode attribute, try to determine from the name
    if opcode.is_none() {
        opcode = match name.to_string().as_str() {
            "Add" | "CustomAdd" => Some(0x01),
            "Store" | "CustomStore" => Some(0x02),
            "Sub" | "CustomSub" => Some(0x03),
            "LoadConstant" | "CustomLoadConstant" => Some(0x04),
            "Multiply" | "CustomMultiply" => Some(0x05),
            "Print" | "CustomPrint" => Some(0x06),
            "LoadMemory" | "CustomLoadMemory" => Some(0x07),
            "StoreMemory" | "CustomStoreMemory" => Some(0x08),
            _ => None,
        };
    }
    
    // If still no opcode, return an error
    let opcode = match opcode {
        Some(code) => code,
        None => {
            return syn::Error::new_spanned(
                name,
                format!("Unknown instruction type: {}. Add a #[opcode = \"X\"] attribute or implement IntoRaw manually.", name)
            ).to_compile_error().into();
        }
    };

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
    let generated_code = if has_operand {
        // For instructions with operands (like Store)
        quote! {
            impl ::vm::instruction::IntoRaw for #name {
                fn into_raw(self) -> ::vm::instruction::RawInstruction {
                    // Extract the operand from the first field
                    let operand = self.0;
                    let b0 = ((operand >> 16) & 0xFF) as u8;
                    let b1 = ((operand >> 8) & 0xFF) as u8;
                    let b2 = (operand & 0xFF) as u8;
                    
                    // Create instruction with the appropriate opcode
                    ::vm::instruction::RawInstruction::from_bytes(b0, b1, b2, #opcode)
                }
            }
        }
    } else {
        // For instructions without operands (like Add)
        quote! {
            impl ::vm::instruction::IntoRaw for #name {
                fn into_raw(self) -> ::vm::instruction::RawInstruction {
                    // Create instruction with only the opcode, no operands
                    ::vm::instruction::RawInstruction::from_bytes(0x00, 0x00, 0x00, #opcode)
                }
            }
        }
    };
    
    // Return the generated implementation
    generated_code.into()
}
