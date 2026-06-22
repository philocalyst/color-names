use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use heck::{ToKebabCase, ToPascalCase};
use proc_macro2::Span;
use quote::quote;
use serde::Deserialize;

#[derive(Deserialize)]
struct ColorData {
    lists: HashMap<String, Vec<ColorEntry>>,
}

#[derive(Deserialize, Clone)]
struct ColorEntry {
    name: String,
    hex: String,
}

#[derive(Clone)]
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

#[allow(clippy::missing_panics_doc)]
pub fn generate(list_key: &str) {
    let data: ColorData = serde_json::from_str(include_str!("../data/colorlists.json")).unwrap();

    let (mut complete, mut short, mut best_of) = (vec![], vec![], vec![]);
    for record in
        csv::Reader::from_reader(include_str!("../data/colornames.csv").as_bytes()).deserialize()
    {
        let r: CompleteRecord = record.unwrap();
        let is_short = r.name.len() <= 12;
        let is_best = r.good_name.is_some();
        let c = Color {
            name: r.name,
            hex: r.hex,
        };
        if is_short {
            short.push(c.clone());
        }
        if is_best {
            best_of.push(c.clone());
        }
        complete.push(c);
    }

    let mut lists: HashMap<String, Vec<Color>> = HashMap::new();
    for (key, colors) in &data.lists {
        lists.insert(
            key.to_kebab_case(),
            colors
                .iter()
                .map(|c| Color {
                    name: c.name.clone(),
                    hex: c.hex.clone(),
                })
                .collect(),
        );
    }
    lists.insert("complete".into(), complete);
    lists.insert("short".into(), short);
    lists.insert("best-of".into(), best_of);

    let colors = lists.remove(list_key).unwrap();
    let code = generate_list_code(list_key, &colors);

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    std::fs::write(Path::new(&out_dir).join("list.rs"), code.to_string()).unwrap();
}

fn generate_list_code(list_key: &str, colors: &[Color]) -> proc_macro2::TokenStream {
    let enum_ident = syn::Ident::new(list_key.to_pascal_case().as_str(), Span::call_site());
    let mut seen = HashSet::new();
    let (mut variants, mut color_arms, mut rgba8_arms, mut from_str_arms) =
        (vec![], vec![], vec![], vec![]);

    for c in colors {
        let ident = syn::Ident::new(&sanitize(&c.name).to_pascal_case(), Span::call_site());
        let name = &c.name;

        from_str_arms.push(quote! { #name => Ok(Self::#ident) });

        if !seen.insert(ident.clone()) {
            continue;
        }

        let hex = &c.hex;
        let [r, g, b] = parse_hex(hex);
        let (rf, gf, bf): (f32, f32, f32) = (
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
        );
        let a: u8 = 255;

        variants.push(quote! { #[doc = #name] #[doc = #hex] #ident });
        color_arms.push(
            quote! { Self::#ident => OpaqueColor { components: [#rf, #gf, #bf], cs: PhantomData } },
        );
        rgba8_arms.push(quote! { Self::#ident => Rgba8 { r: #r, g: #g, b: #b, a: #a } });
    }

    quote! {
        use color::{OpaqueColor, Srgb};
        use std::marker::PhantomData;
        pub type Rgba8 = rgb::RGBA<u8>;

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct NameNotFound;

        impl std::fmt::Display for NameNotFound {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("no color found with that name")
            }
        }

        impl std::error::Error for NameNotFound {}

        #[allow(dead_code, clippy::doc_markdown)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum #enum_ident { #(#variants),* }

        #[allow(clippy::unreadable_literal)]
        impl #enum_ident {
            #[must_use]
            pub const fn color(self) -> OpaqueColor<Srgb> {
                match self { #(#color_arms),* }
            }

            #[must_use]
            pub const fn to_rgba8(self) -> Rgba8 {
                match self { #(#rgba8_arms),* }
            }
        }

        #[allow(unreachable_patterns)]
        impl std::str::FromStr for #enum_ident {
            type Err = NameNotFound;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#from_str_arms),*,
                    _ => Err(NameNotFound),
                }
            }
        }

        impl TryFrom<&str> for #enum_ident {
            type Error = NameNotFound;
            fn try_from(s: &str) -> Result<Self, Self::Error> { s.parse() }
        }

        impl TryFrom<String> for #enum_ident {
            type Error = NameNotFound;
            fn try_from(s: String) -> Result<Self, Self::Error> { s.parse() }
        }
    }
}

fn parse_hex(hex: &str) -> [u8; 3] {
    let h = hex.trim_start_matches('#');
    [
        u8::from_str_radix(&h[0..2], 16).unwrap_or(0),
        u8::from_str_radix(&h[2..4], 16).unwrap_or(0),
        u8::from_str_radix(&h[4..6], 16).unwrap_or(0),
    ]
}

fn sanitize(name: &str) -> String {
    use num2words::Num2Words;

    let split = name
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(name.len());
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

    if num != 0
        && let Ok(words) = Num2Words::new(num).to_words()
    {
        result.insert_str(0, &words);
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
