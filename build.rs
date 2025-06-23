use heck::ToPascalCase;
use rgb;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct ColorData {
    lists: HashMap<String, Vec<Color>>,
    meta: HashMap<String, ColorSetMeta>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Color {
    name: String,
    hex: String,
    #[serde(default)]
    meta: Option<ColorMeta>,
}

#[derive(Debug, Deserialize)]
struct CompleteRecord {
    name: String,
    hex: String,
    #[serde(rename = "good name")]
    good_name: Option<String>, // Some entries are empty, so optional
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ColorMeta {
    #[serde(default)]
    link: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ColorSetMeta {
    title: String,
    description: String,
    source: String,
    key: String,
    license: String,
    #[serde(rename = "colorCount")]
    color_count: u32,
}

// Errors for hex parsing failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexParseError {
    InvalidLength,
    InvalidCharacter,
    ColorNotFound,
}

impl std::fmt::Display for HexParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HexParseError::InvalidLength => write!(f, "Invalid hex string length"),
            HexParseError::InvalidCharacter => write!(f, "Invalid character in hex string"),
            HexParseError::ColorNotFound => write!(f, "No color found matching the hex value"),
        }
    }
}

impl std::error::Error for HexParseError {}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=colors.json");

    // Read the JSON file
    let json_content = fs::read_to_string("colors.json").expect("Failed to read colors.json");

    let mut color_data: ColorData =
        serde_json::from_str(&json_content).expect("Failed to parse JSON");

    // Read the CSV file for the 'complete' list
    let mut complete_data = csv::Reader::from_path("colornames.csv")?;

    let mut complete_colors: Vec<Color> = Vec::new();
    let mut short_colors: Vec<Color> = Vec::new();
    let mut elite_colors: Vec<Color> = Vec::new();

    // Load into the various children lists
    for record in complete_data.deserialize() {
        let record: CompleteRecord = record?;
        let color: Color = Color {
            name: record.name.clone(),
            hex: record.hex,
            meta: None,
        };

        if record.name.len() <= 12 {
            short_colors.push(color.clone());
        }

        if record.good_name.is_some() {
            elite_colors.push(color.clone());
        }

        complete_colors.push(color);
    }

    color_data
        .lists
        .insert(String::from("complete"), complete_colors);

    color_data
        .lists
        .insert(String::from("bestOf"), elite_colors);

    color_data.lists.insert(String::from("short"), short_colors);

    let mut generated_code = TokenStream::new();

    // Add use statements at the top of the generated file
    generated_code.extend(quote! {
        use rgb;
        use hex::FromHex;
    });

    // Add the error type at the almost top
    generated_code.extend(quote! {

        // Errors for hex parsing failures
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum HexParseError {
            InvalidLength,
            InvalidCharacter,
            ColorNotFound,
        }

        impl std::fmt::Display for HexParseError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                HexParseError::InvalidLength => write!(f, "Invalid hex string length"),
                HexParseError::InvalidCharacter => write!(f, "Invalid character in hex string"),
                HexParseError::ColorNotFound => write!(f, "No color found matching the hex value"),
                }
            }
        }

        impl std::error::Error for HexParseError {}

    });

    // Generate a master enum containing all color sets
    let color_set_variants: Vec<_> = color_data
        .meta
        .keys()
        .map(|key| {
            let variant_name = key.to_pascal_case();
            let variant_ident = syn::Ident::new(&variant_name, proc_macro2::Span::call_site());
            quote! { #variant_ident }
        })
        .collect();

    generated_code.extend(quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum ColorSet {
            #(#color_set_variants),*
        }
    });

    // Generate individual enums for each color set
    for (set_key, colors) in &color_data.lists {
        let enum_name = set_key.to_pascal_case();
        let enum_identifier = syn::Ident::new(&enum_name, proc_macro2::Span::call_site());

        let mut identifers = HashSet::new();

        let variants: Vec<_> = colors
            .iter()
            .filter_map(|color| {
                let variant_name = sanitize_identifier(&color.name).to_pascal_case();

                println!("{variant_name}");
                let variant_identitifer =
                    syn::Ident::new(&variant_name, proc_macro2::Span::call_site());

                // If the identifier is not unique, the insertion is skipped.
                if !identifers.insert(variant_identitifer.clone()) {
                    return None;
                }

                let hex_value = &color.hex;
                let original_name = &color.name;

                Some(quote! {
                    #[doc = #original_name]
                    #[doc = #hex_value]
                    #variant_identitifer
                })
            })
            .collect();

        // Generate the enum
        generated_code.extend(quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #enum_identifier {
                #(#variants),*
            }
        });

        // Generate implementation with hex values
        let hex_match_arms: Vec<TokenStream> = colors
            .iter()
            .map(|color| {
                let variant_name = sanitize_identifier(&color.name).to_pascal_case();
                let variant_identifier =
                    syn::Ident::new(&variant_name, proc_macro2::Span::call_site());
                let hex_value = &color.hex;

                quote! {
                    #enum_identifier::#variant_identifier => #hex_value
                }
            })
            .collect();

        let name_match_arms: Vec<TokenStream> = colors
            .iter()
            .map(|color| {
                let variant_name = sanitize_identifier(&color.name).to_pascal_case();
                let variant_ident = syn::Ident::new(&variant_name, proc_macro2::Span::call_site());
                let original_name = &color.name;

                quote! {
                    #enum_identifier::#variant_ident => #original_name
                }
            })
            .collect();

        generated_code.extend(quote! {
            impl #enum_identifier {
                /// Returns the hex color value (including the # prefix)
                pub fn hex(&self) -> &'static str {
                    match self {
                        #(#hex_match_arms),*
                    }
                }

                /// Returns the original color name
                pub fn name(&self) -> &'static str {
                    match self {
                        #(#name_match_arms),*
                    }
                }

                /// Returns the RGB values as a tuple (r, g, b)
                pub fn rgb(&self) -> rgb::Rgb<u8> {
                    let hex = self.hex().trim_start_matches('#');
                    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                    rgb::Rgb {
                        r,
                        g,
                        b
                    }
                }
            }
        });

        // Generate From implementation for easy conversion to hex
        generated_code.extend(quote! {
            impl From<self::#enum_identifier> for String {
                fn from(color: self::#enum_identifier) -> Self {
                    color.hex().to_string()
                }
            }


        });

        let from_hex_match_arms: Vec<TokenStream> = colors
            .iter()
            .filter_map(|color| {
                let variant_name = sanitize_identifier(&color.name).to_pascal_case();
                let variant_identifier =
                    syn::Ident::new(&variant_name, proc_macro2::Span::call_site());

                // Skip if this variant was filtered out due to duplicate names
                if !identifers.contains(&variant_identifier) {
                    return None;
                }

                let hex_value = &color.hex;
                // Remove the # prefix for matching
                let hex_without_prefix = hex_value.trim_start_matches('#');

                Some(quote! {
                    #hex_without_prefix | #hex_value => Ok(#enum_identifier::#variant_identifier)
                })
            })
            .collect();

        generated_code.extend(quote! {
            impl FromHex for #enum_identifier {
                type Error = HexParseError;

                fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
                    let hex_str = std::str::from_utf8(hex.as_ref())
                        .map_err(|_| HexParseError::InvalidCharacter)?;

                    // Normalize the hex string (remove # if present, convert to lowercase)
                    let normalized = hex_str.trim_start_matches('#').to_lowercase();

                    // Validate length (should be 6 characters for RGB)
                    if normalized.len() != 6 {
                        return Err(HexParseError::InvalidLength);
                    }

                    // Validate that all characters are valid hex
                    if !normalized.chars().all(|c| c.is_ascii_hexdigit()) {
                        return Err(HexParseError::InvalidCharacter);
                    }

                    // Match against known colors
                    match normalized.as_str() {
                        #(#from_hex_match_arms),*,
                        _ => Err(HexParseError::ColorNotFound)
                    }
                }
            }
        });

        // Parse from a string
        generated_code.extend(quote! {
            impl std::str::FromStr for #enum_identifier {
                type Err = HexParseError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    Self::from_hex(s.as_bytes())
                }
            }
        });

        // Parse from the rest of the string types
        generated_code.extend(quote! {
            impl TryFrom<&str> for #enum_identifier {
                type Error = HexParseError;

                fn try_from(hex: &str) -> Result<Self, Self::Error> {
                    Self::from_hex(hex.as_bytes())
                }
            }

            impl TryFrom<String> for #enum_identifier {
                type Error = HexParseError;

                fn try_from(hex: String) -> Result<Self, Self::Error> {
                    Self::from_hex(hex.as_bytes())
                }
            }
        });
    }

    // Write the generated code to the output file
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("colors.rs");
    fs::write(&dest_path, generated_code.to_string()).expect("Failed to write generated code");
    Ok(())
}

/// Sanitize color names to be valid Rust identifiers
fn sanitize_identifier(name: &str) -> String {
    use num2words::Num2Words;
    let mut result = String::new();
    let mut chars = name.chars().peekable();

    let mut num: u32 = 0;
    let mut number_prefix = true;
    // Handle leading digits
    for ch in chars {
        if ch.is_ascii_digit() && number_prefix {
            // Simply multiply by 10 and add the new digit
            num = num * 10 + ch.to_digit(10).unwrap();

            // Skip adding to result
            continue;
        } else {
            // Once the prefix ends we ignore the operations
            number_prefix = false;
        }
        result.push(ch);
    }

    // Remove trailing underscore
    if result.ends_with('_') {
        result.pop();
    }

    // Ensure we have a valid identifier
    if result.is_empty() {
        result = "Unknown".to_string();
    }

    // Handle Rust keywords
    match result.as_str() {
        "type" => "Type_".to_string(),
        "match" => "Match_".to_string(),
        "loop" => "Loop_".to_string(),
        "move" => "Move_".to_string(),
        "self" => "Self_".to_string(),
        "super" => "Super_".to_string(),
        "_" => "Underscore_".to_string(),
        "true" => "True_".to_string(),
        "false" => "False_".to_string(),
        _ => result,
    }
}
