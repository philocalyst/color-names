use std::{collections::{HashMap, HashSet}, env, error::Error, fs, path::Path};

use heck::{ToKebabCase, ToPascalCase};
use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct ColorData {
	lists: HashMap<String, Vec<Color>>,
	meta:  HashMap<String, ColorSetMeta>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Color {
	name: String,
	hex:  String,
	#[serde(default)]
	meta: Option<ColorMeta>,
}

#[derive(Debug, Deserialize)]
struct CompleteRecord {
	name:      String,
	hex:       String,
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
	title:       String,
	description: String,
	source:      String,
	key:         String,
	license:     String,
	#[serde(rename = "colorCount")]
	color_count: u32,
}

// Errors for color name parsing failures
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
			HexParseError::ColorNotFound => write!(f, "No color found matching the name"),
		}
	}
}

impl std::error::Error for HexParseError {}

fn main() -> Result<(), Box<dyn Error>> {
	println!("cargo:rerun-if-changed=./color-name-lists/dist/colorlists.json");

	// Read the JSON file
	let json_content = fs::read_to_string("./color-name-lists/dist/colorlists.json")
		.expect("Failed to read colors.json");

	let mut color_data: ColorData =
		serde_json::from_str(&json_content).expect("Failed to parse JSON");

	// Read the CSV file for the 'complete' list
	let mut complete_data = csv::Reader::from_path("./color-names/src/colornames.csv")?;

	let mut complete_colors: Vec<Color> = Vec::new();
	let mut short_colors: Vec<Color> = Vec::new();
	let mut elite_colors: Vec<Color> = Vec::new();

	// Load into the various children lists
	for record in complete_data.deserialize() {
		let record: CompleteRecord = record?;
		let color: Color = Color { name: record.name.clone(), hex: record.hex, meta: None };

		if record.name.len() <= 12 {
			short_colors.push(color.clone());
		}

		if record.good_name.is_some() {
			elite_colors.push(color.clone());
		}

		complete_colors.push(color);
	}

	color_data.lists.insert(String::from("complete"), complete_colors);

	color_data.lists.insert(String::from("bestOf"), elite_colors);

	color_data.lists.insert(String::from("short"), short_colors);

	let mut generated_code = TokenStream::new();

	// Add use statements at the top of the generated file
	generated_code.extend(quote! {
			use rgb;
			use color;
			use hex::ToHex;
	});

	// Add the error type at the almost top
	generated_code.extend(quote! {

			// Errors for color name parsing failures
			#[allow(dead_code)]
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
							HexParseError::ColorNotFound => write!(f, "No color found matching the name"),
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
			#[allow(dead_code)]
			#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
			pub enum ColorSet {
					#(#color_set_variants),*
			}
	});

	// Generate individual enums for each color set
	for (set_key, colors) in &color_data.lists {
		// Make sure to conditionally ignore certain sets and not generate if the
		// feature flag isn't on. If it is on, of course compile.
		match set_key.to_kebab_case().as_str() {
			"basic" if cfg!(feature = "basic") == false => continue,
			"html" if cfg!(feature = "html") == false => continue,
			"short" if cfg!(feature = "short") == false => continue,
			"best-of" if cfg!(feature = "best-of") == false => continue,
			"complete" if cfg!(feature = "complete") == false => continue,
			"japanese-traditional" if cfg!(feature = "japanese-traditional") == false => continue,
			"le-corbusier" if cfg!(feature = "le-corbusier") == false => continue,
			"nbs-iscc" if cfg!(feature = "nbs-iscc") == false => continue,
			"ntc" if cfg!(feature = "ntc") == false => continue,
			"osxcrayons" if cfg!(feature = "osxcrayons") == false => continue,
			"ral" if cfg!(feature = "ral") == false => continue,
			"ridgway" if cfg!(feature = "ridgway") == false => continue,
			"sanzo-wada-i" if cfg!(feature = "sanzo-wada-i") == false => continue,
			"thesaurus" if cfg!(feature = "thesaurus") == false => continue,
			"werner" if cfg!(feature = "werner") == false => continue,
			"windows" if cfg!(feature = "windows") == false => continue,
			"wikipedia" if cfg!(feature = "wikipedia") == false => continue,
			"french" if cfg!(feature = "french") == false => continue,
			"spanish" if cfg!(feature = "spanish") == false => continue,
			"german" if cfg!(feature = "german") == false => continue,
			"x11" if cfg!(feature = "x11") == false => continue,
			"xkcd" if cfg!(feature = "xkcd") == false => continue,
			"risograph" if cfg!(feature = "risograph") == false => continue,
			"chinese-traditional" if cfg!(feature = "chinese-traditional") == false => continue,
			"hindi" if cfg!(feature = "hindi") == false => continue,
			_ => println!("List {} is not enabled, skipping", set_key),
		}

		let enum_name = set_key.to_pascal_case();
		let enum_identifier = syn::Ident::new(&enum_name, proc_macro2::Span::call_site());

		let mut identifers = HashSet::new();

		let variants: Vec<_> = colors
			.iter()
			.filter_map(|color| {
				let variant_name = sanitize_identifier(&color.name).to_pascal_case();

				let variant_identitifer = syn::Ident::new(&variant_name, proc_macro2::Span::call_site());

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
				#[allow(dead_code)]
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
				let variant_identifier = syn::Ident::new(&variant_name, proc_macro2::Span::call_site());
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
				#[allow(unreachable_patterns)]
				#[allow(dead_code)]
				impl #enum_identifier {
						/// Returns the hex color value (including the # prefix)
						pub fn hex(&self) -> &'static str {
								match self {
										#(#hex_match_arms),*
								}
						}

						/// Returns the original color name
						pub fn color_name(&self) -> &'static str {
								match self {
										#(#name_match_arms),*
								}
						}

						/// Returns the color in a "correct" representation, initializing in the provided colorspace. Encoded as a [color](https://docs.rs/color/latest/color) type.
						pub fn color<CS>(&self) -> color::OpaqueColor<CS> {
								 let hex = hex.trim_start_matches('#');

								let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
								let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
								let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

								color::OpaqueColor {
										components: [
												r as f32 / 255.0,
												g as f32 / 255.0,
												b as f32 / 255.0,
										],
										cs: PhantomData,
								}
						}

						/// Returns the RGB values for the color. Encoded as an [RGB](https://docs.rs/rgb/latest/rgb/) type
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

		// Generate match arms for parsing by name
		let from_name_match_arms: Vec<TokenStream> = colors
			.iter()
			.filter_map(|color| {
				let variant_name = sanitize_identifier(&color.name).to_pascal_case();
				let variant_identifier = syn::Ident::new(&variant_name, proc_macro2::Span::call_site());

				// Skip if this variant was filtered out due to duplicate names
				if !identifers.contains(&variant_identifier) {
					return None;
				}

				let original_name = &color.name;

				Some(quote! {
						#original_name => Ok(#enum_identifier::#variant_identifier)
				})
			})
			.collect();

		// Parse from a string (by name)
		generated_code.extend(quote! {
				impl std::str::FromStr for #enum_identifier {
						type Err = HexParseError;

						fn from_str(s: &str) -> Result<Self, Self::Err> {
								match s {
										#(#from_name_match_arms),*,
										_ => Err(HexParseError::ColorNotFound)
								}
						}
				}
		});

		// Implement ToHex
		generated_code.extend(quote! {
				impl ToHex for #enum_identifier {
						fn encode_hex<T: FromIterator<char>>(&self) -> T {
								let hex = self.hex().trim_start_matches('#');
								hex.chars().collect()
						}

						fn encode_hex_upper<T: FromIterator<char>>(&self) -> T {
								let hex = self.hex().trim_start_matches('#').to_uppercase();
								hex.chars().collect()
						}
				}
		});

		// Parse from the rest of the string types (by name)
		generated_code.extend(quote! {
				impl TryFrom<&str> for #enum_identifier {
						type Error = HexParseError;

						fn try_from(name: &str) -> Result<Self, Self::Error> {
								name.parse()
						}
				}

				impl TryFrom<String> for #enum_identifier {
						type Error = HexParseError;

						fn try_from(name: String) -> Result<Self, Self::Error> {
								name.parse()
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
	let chars = name.chars();

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

		if ch == '₂' || ch == '²' {
			result.push('2');
		} else if ch == '№' {
			result.push('N');
			result.push('o');
		} else if ch == 'Ⅱ' {
			result.push('|')
		} else {
			result.push(ch);
		}
	}

	let num_as_words = Num2Words::new(num).to_words();

	if num != 0 {
		if let Ok(num_as_words) = num_as_words {
			// Add in the converted number
			result.insert_str(0, &num_as_words);
		}
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
