use std::{
	collections::{HashMap, HashSet},
	env,
	error::Error,
	fs,
	path::Path,
};

use heck::{ToKebabCase, ToPascalCase};
use proc_macro2::Span;
use quote::quote;
use serde::Deserialize;

#[derive(Deserialize)]
struct ColorData {
	lists: HashMap<String, Vec<Color>>,
}

#[derive(Deserialize, Clone)]
struct Color {
	name: String,
	hex: String,
}

#[derive(Deserialize)]
struct CompleteRecord {
	name: String,
	hex: String,
	#[serde(rename = "good name")]
	good_name: Option<String>,
}

fn feature_enabled(key: &str) -> bool {
	let var = format!("CARGO_FEATURE_{}", key.to_uppercase().replace('-', "_"));
	env::var(&var).is_ok()
}

fn parse_hex(hex: &str) -> [u8; 3] {
	let h = hex.trim_start_matches('#');
	[
		u8::from_str_radix(&h[0..2], 16).unwrap_or(0),
		u8::from_str_radix(&h[2..4], 16).unwrap_or(0),
		u8::from_str_radix(&h[4..6], 16).unwrap_or(0),
	]
}

fn main() -> Result<(), Box<dyn Error>> {
	println!("cargo:rerun-if-changed=./color-name-lists/dist/colorlists.json");
	println!("cargo:rerun-if-changed=./color-names/src/colornames.csv");

	let mut data: ColorData =
		serde_json::from_str(&fs::read_to_string("./color-name-lists/dist/colorlists.json")?)?;

	let (mut complete, mut short, mut best_of) = (vec![], vec![], vec![]);
	for record in csv::Reader::from_path("./color-names/src/colornames.csv")?.deserialize() {
		let r: CompleteRecord = record?;
		let is_short = r.name.len() <= 12;
		let is_best = r.good_name.is_some();
		let c = Color { name: r.name, hex: r.hex };
		if is_short {
			short.push(c.clone());
		}
		if is_best {
			best_of.push(c.clone());
		}
		complete.push(c);
	}
	data.lists.extend([
		("complete".into(), complete),
		("bestOf".into(), best_of),
		("short".into(), short),
	]);

	let mut code = quote! {
		use color::{OpaqueColor, Srgb};
		use std::marker::PhantomData;
		pub type Rgba8 = rgb::RGBA<u8>;
	};

	for (set_key, colors) in &data.lists {
		if !feature_enabled(&set_key.to_kebab_case()) {
			continue;
		}

		let enum_ident = syn::Ident::new(&set_key.to_pascal_case(), Span::call_site());
		let mut seen = HashSet::new();
		let (mut variants, mut color_arms, mut rgba8_arms) = (vec![], vec![], vec![]);

		for c in colors {
			let ident = syn::Ident::new(&sanitize(&c.name).to_pascal_case(), Span::call_site());
			if !seen.insert(ident.clone()) {
				continue;
			}

			let (name, hex) = (&c.name, &c.hex);
			let [r, g, b] = parse_hex(hex);
			let (rf, gf, bf) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
			let a: u8 = 255;

			variants.push(quote! { #[doc = #name] #[doc = #hex] #ident });
			color_arms.push(
				quote! { Self::#ident => OpaqueColor { components: [#rf, #gf, #bf], cs: PhantomData } },
			);
			rgba8_arms.push(quote! { Self::#ident => Rgba8 { r: #r, g: #g, b: #b, a: #a } });
		}

		code.extend(quote! {
			#[allow(dead_code)]
			#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
			#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
			pub enum #enum_ident { #(#variants),* }

			impl #enum_ident {
				/// Returns the color as [`OpaqueColor<Srgb>`]; convert with `.convert::<CS>()`.
				pub fn color(self) -> OpaqueColor<Srgb> {
					match self { #(#color_arms),* }
				}

				/// Returns pre-decoded RGBA bytes (alpha = 255). Fast path for renderers.
				pub fn to_rgba8(self) -> Rgba8 {
					match self { #(#rgba8_arms),* }
				}
			}
		});
	}

	fs::write(Path::new(&env::var_os("OUT_DIR").unwrap()).join("colors.rs"), code.to_string())?;
	Ok(())
}

fn sanitize(name: &str) -> String {
	use num2words::Num2Words;

	let split = name.find(|c: char| !c.is_ascii_digit()).unwrap_or(name.len());
	let num: u32 = name[..split].parse().unwrap_or(0);

	let mut result = String::new();
	for ch in name[split..].chars() {
		match ch {
			'₂' | '²' => result.push('2'),
			'№' => result.push_str("No"),
			'Ⅱ' => result.push('|'),
			c => result.push(c),
		}
	}

	if num != 0 {
		if let Ok(words) = Num2Words::new(num).to_words() {
			result.insert_str(0, &words);
		}
	}

	if result.ends_with('_') {
		result.pop();
	}
	if result.is_empty() {
		result = "Unknown".into();
	}

	match result.as_str() {
		"type" => "Type_".into(),
		"match" => "Match_".into(),
		"loop" => "Loop_".into(),
		"move" => "Move_".into(),
		"self" => "Self_".into(),
		"super" => "Super_".into(),
		"_" => "Underscore_".into(),
		"true" => "True_".into(),
		"false" => "False_".into(),
		_ => result,
	}
}
