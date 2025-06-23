use heck::ToPascalCase;
use rgb;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
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

#[derive(Debug, Deserialize, Serialize)]
struct Color {
    name: String,
    hex: String,
    #[serde(default)]
    meta: Option<ColorMeta>,
}

#[derive(Debug, Deserialize, Serialize)]
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

fn main() {
    println!("cargo:rerun-if-changed=colors.json");

    // Read the JSON file
    let json_content = fs::read_to_string("colors.json").expect("Failed to read colors.json");

    let color_data: ColorData = serde_json::from_str(&json_content).expect("Failed to parse JSON");

    let mut generated_code = TokenStream::new();

    // Add use statements at the top of the generated file
    generated_code.extend(quote! {
        use rgb;
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
        let hex_match_arms: Vec<_> = colors
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

        let name_match_arms: Vec<_> = colors
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
            impl From<#enum_identifier> for String {
                fn from(color: #enum_identifier) -> Self {
                    color.hex().to_string()
                }
            }
        });
    }

    // Write the generated code to the output file
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("colors.rs");
    fs::write(&dest_path, generated_code.to_string()).expect("Failed to write generated code");
}

/// Sanitize color names to be valid Rust identifiers
fn sanitize_identifier(name: &str) -> String {
    let mut result = String::new();
    let mut chars = name.chars().peekable();

    // Handle leading digits
    if chars.peek().map_or(false, |c| c.is_ascii_digit()) {
        result.push('_');
    }

    for ch in chars {
        result.push(ch)
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
