use std::{collections::HashMap, collections::HashSet, path::Path};

use heck::{ToKebabCase, ToPascalCase};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use serde::Deserialize;

const COLORLISTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/colorlists.json");
const COLORNAMES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/colornames.csv");

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const GOLDEN: u64 = 0x9e37_79b9_7f4a_7c15;

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
    println!("cargo:rerun-if-changed={COLORLISTS}");
    println!("cargo:rerun-if-changed={COLORNAMES}");

    let colors = load_list(list_key);
    let code = generate_list_code(list_key, &colors);

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    std::fs::write(Path::new(&out_dir).join("list.rs"), code.to_string()).unwrap();
}

fn load_list(list_key: &str) -> Vec<Color> {
    if matches!(list_key, "complete" | "short" | "best-of") {
        return load_derived(list_key);
    }

    let data: ColorData = serde_json::from_str(include_str!("../data/colorlists.json")).unwrap();
    data.lists
        .into_iter()
        .find(|(key, _)| key.to_kebab_case() == list_key)
        .map_or_else(
            || panic!("unknown color list: {list_key}"),
            |(_, colors)| {
                colors
                    .into_iter()
                    .map(|c| Color {
                        name: c.name,
                        hex: c.hex,
                    })
                    .collect()
            },
        )
}

fn load_derived(list_key: &str) -> Vec<Color> {
    let (mut complete, mut short, mut best_of) = (vec![], vec![], vec![]);
    for record in
        csv::Reader::from_reader(include_bytes!("../data/colornames.csv").as_slice()).deserialize()
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

    match list_key {
        "complete" => complete,
        "short" => short,
        "best-of" => best_of,
        _ => unreachable!(),
    }
}

#[allow(clippy::too_many_lines)]
fn generate_list_code(list_key: &str, colors: &[Color]) -> TokenStream {
    let enum_ident = Ident::new(&list_key.to_pascal_case(), Span::call_site());

    let mut variant_defs = Vec::new();
    let mut variant_names = Vec::new();
    let mut variants_array = Vec::new();
    let mut color_values = Vec::new();
    let mut palette_values = Vec::new();
    let mut rgba8_values = Vec::new();

    let mut idents: Vec<String> = Vec::new();
    let mut name_keys: Vec<String> = Vec::new();
    let mut name_idents: Vec<Ident> = Vec::new();

    let mut seen_idents = HashSet::new();
    let mut seen_names = HashSet::new();

    for c in colors {
        let ident_text = sanitize(&c.name).to_pascal_case();
        let ident = Ident::new(&ident_text, Span::call_site());

        if seen_names.insert(c.name.clone()) {
            name_keys.push(c.name.clone());
            name_idents.push(ident.clone());
        }

        if !seen_idents.insert(ident_text.clone()) {
            continue;
        }

        let [r, g, b] = parse_hex(&c.hex);
        let (rf, gf, bf): (f32, f32, f32) = (
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
        );
        let name_lit = Literal::string(&c.name);
        let hex_lit = Literal::string(&c.hex);

        variant_defs.push(quote! { #[doc = #name_lit] #[doc = #hex_lit] #ident });
        variant_names.push(Literal::string(&ident_text));
        variants_array.push(quote! { #enum_ident::#ident });
        color_values.push(quote! {
            color::OpaqueColor {
                components: [#rf, #gf, #bf],
                cs: std::marker::PhantomData,
            }
        });
        palette_values.push(quote! { palette::Srgb::new(#rf, #gf, #bf) });
        rgba8_values.push(quote! { Rgba8 { r: #r, g: #g, b: #b, a: 255 } });
        idents.push(ident_text);
    }

    let variant_count = idents.len();
    let name_phf = build_phf(&name_keys);

    let name_seed = name_phf.seed;
    let name_buckets = name_phf.bucket_count;
    let name_slots = name_phf.table_size;
    let name_displace = name_phf
        .displace
        .iter()
        .map(|d| quote! { #d })
        .collect::<Vec<_>>();
    let name_table = name_phf
        .slots
        .iter()
        .map(|&slot| {
            slot.map_or_else(
                || quote! { None },
                |i| {
                    let key = Literal::string(&name_keys[i]);
                    let ident = &name_idents[i];
                    quote! { Some((#key, #enum_ident::#ident)) }
                },
            )
        })
        .collect::<Vec<_>>();

    let (ident_seed, ident_buckets, ident_slots, ident_displace, ident_table) =
        if std::env::var_os("CARGO_FEATURE_SERDE").is_some() {
            let ident_phf = build_phf(&idents);
            let seed = ident_phf.seed;
            let buckets = ident_phf.bucket_count;
            let slots = ident_phf.table_size;
            let displace = ident_phf
                .displace
                .iter()
                .map(|d| quote! { #d })
                .collect::<Vec<_>>();
            let table = ident_phf
                .slots
                .iter()
                .map(|&slot| {
                    slot.map_or_else(
                        || quote! { None },
                        |i| {
                            let key = Literal::string(&idents[i]);
                            let ident = Ident::new(&idents[i], Span::call_site());
                            quote! { Some((#key, #enum_ident::#ident)) }
                        },
                    )
                })
                .collect::<Vec<_>>();
            (seed, buckets, slots, displace, table)
        } else {
            (0u64, 0usize, 0usize, Vec::new(), Vec::new())
        };

    quote! {
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
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub enum #enum_ident { #(#variant_defs),* }

        impl std::fmt::Debug for #enum_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(VARIANT_NAMES[*self as usize])
            }
        }

        static VARIANT_NAMES: [&str; #variant_count] = [#(#variant_names),*];

        #[cfg(feature = "serde")]
        static VARIANTS: [#enum_ident; #variant_count] = [#(#variants_array),*];

        #[cfg(feature = "color")]
        #[allow(clippy::unreadable_literal)]
        const COLORS: [color::OpaqueColor<color::Srgb>; #variant_count] = [#(#color_values),*];

        #[cfg(feature = "palette")]
        #[allow(clippy::unreadable_literal)]
        const PALETTE: [palette::Srgb; #variant_count] = [#(#palette_values),*];

        #[allow(clippy::unreadable_literal)]
        const RGBA8: [Rgba8; #variant_count] = [#(#rgba8_values),*];

        impl #enum_ident {
            #[cfg(feature = "color")]
            #[must_use]
            pub const fn color(self) -> color::OpaqueColor<color::Srgb> {
                COLORS[self as usize]
            }

            #[cfg(feature = "palette")]
            #[must_use]
            pub const fn palette(self) -> palette::Srgb {
                PALETTE[self as usize]
            }

            #[must_use]
            pub const fn to_rgba8(self) -> Rgba8 {
                RGBA8[self as usize]
            }
        }

        #[inline]
        fn phf_splitmix(mut z: u64) -> u64 {
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        }

        #[inline]
        fn phf_lookup<const M: usize, const N: usize>(
            seed: u64,
            displace: &[u32; M],
            table: &[Option<(&str, #enum_ident)>; N],
            s: &str,
        ) -> Option<#enum_ident> {
            let mut hash = 0xcbf2_9ce4_8422_2325u64;
            for &byte in s.as_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
            let bucket = (phf_splitmix(hash ^ seed) as usize) % M;
            let offset = (phf_splitmix(hash ^ seed ^ 0x9e37_79b9_7f4a_7c15) as usize) % N;
            let index = (offset + displace[bucket] as usize) % N;
            match table[index] {
                Some((name, value)) if name == s => Some(value),
                _ => None,
            }
        }

        const NAME_SEED: u64 = #name_seed;
        static NAME_DISPLACE: [u32; #name_buckets] = [#(#name_displace),*];
        static NAME_TABLE: [Option<(&str, #enum_ident)>; #name_slots] = [#(#name_table),*];

        #[cfg(feature = "serde")]
        const IDENT_SEED: u64 = #ident_seed;
        #[cfg(feature = "serde")]
        static IDENT_DISPLACE: [u32; #ident_buckets] = [#(#ident_displace),*];
        #[cfg(feature = "serde")]
        static IDENT_TABLE: [Option<(&str, #enum_ident)>; #ident_slots] = [#(#ident_table),*];

        impl std::str::FromStr for #enum_ident {
            type Err = NameNotFound;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match phf_lookup(NAME_SEED, &NAME_DISPLACE, &NAME_TABLE, s) {
                    Some(value) => Ok(value),
                    None => Err(NameNotFound),
                }
            }
        }

        impl TryFrom<&str> for #enum_ident {
            type Error = NameNotFound;

            fn try_from(s: &str) -> Result<Self, Self::Error> {
                s.parse()
            }
        }

        impl TryFrom<String> for #enum_ident {
            type Error = NameNotFound;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                s.parse()
            }
        }

        #[cfg(feature = "serde")]
        impl serde::Serialize for #enum_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_unit_variant(
                    stringify!(#enum_ident),
                    *self as u32,
                    VARIANT_NAMES[*self as usize],
                )
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for #enum_ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct VariantIdent;

                impl<'de> serde::de::Visitor<'de> for VariantIdent {
                    type Value = #enum_ident;

                    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.write_str(concat!("variant identifier for ", stringify!(#enum_ident)))
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        VARIANTS.get(value as usize).copied().ok_or_else(|| {
                            E::invalid_value(serde::de::Unexpected::Unsigned(value), &self)
                        })
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match phf_lookup(IDENT_SEED, &IDENT_DISPLACE, &IDENT_TABLE, value) {
                            Some(variant) => Ok(variant),
                            None => Err(E::unknown_variant(value, &VARIANT_NAMES)),
                        }
                    }

                    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        match std::str::from_utf8(value) {
                            Ok(s) => self.visit_str(s),
                            Err(_) => Err(E::invalid_value(
                                serde::de::Unexpected::Bytes(value),
                                &self,
                            )),
                        }
                    }
                }

                struct VariantSeed;

                impl<'de> serde::de::DeserializeSeed<'de> for VariantSeed {
                    type Value = #enum_ident;

                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                    {
                        deserializer.deserialize_identifier(VariantIdent)
                    }
                }

                struct EnumVisitor;

                impl<'de> serde::de::Visitor<'de> for EnumVisitor {
                    type Value = #enum_ident;

                    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.write_str(concat!("enum ", stringify!(#enum_ident)))
                    }

                    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
                    where
                        A: serde::de::EnumAccess<'de>,
                    {
                        use serde::de::VariantAccess as _;
                        let (value, variant) = data.variant_seed(VariantSeed)?;
                        variant.unit_variant()?;
                        Ok(value)
                    }
                }

                deserializer.deserialize_enum(stringify!(#enum_ident), &[], EnumVisitor)
            }
        }
    }
}

struct PerfectHash {
    seed: u64,
    bucket_count: usize,
    table_size: usize,
    displace: Vec<u32>,
    slots: Vec<Option<usize>>,
}

#[allow(clippy::cast_possible_truncation)]
fn build_phf(keys: &[String]) -> PerfectHash {
    assert!(!keys.is_empty(), "cannot build a perfect hash from no keys");

    let bases: Vec<u64> = keys.iter().map(|k| fnv1a(k.as_bytes())).collect();
    let bucket_count = keys.len();
    let mut table_size = (keys.len() + keys.len() / 2).max(2);
    let mut seed = 0u64;

    loop {
        let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); bucket_count];
        for (index, &base) in bases.iter().enumerate() {
            buckets[(splitmix(base ^ seed) as usize) % bucket_count].push(index);
        }

        let mut order: Vec<usize> = (0..bucket_count).collect();
        order.sort_by_key(|&bucket| std::cmp::Reverse(buckets[bucket].len()));

        let mut used = vec![false; table_size];
        let mut displace = vec![0u32; bucket_count];
        let mut slots: Vec<Option<usize>> = vec![None; table_size];
        let mut placed = true;

        'buckets: for &bucket in &order {
            let members = &buckets[bucket];
            if members.is_empty() {
                continue;
            }

            let offsets: Vec<usize> = members
                .iter()
                .map(|&index| (splitmix(bases[index] ^ seed ^ GOLDEN) as usize) % table_size)
                .collect();

            let mut candidate = 0usize;
            loop {
                if candidate >= table_size {
                    placed = false;
                    break 'buckets;
                }

                let mut targets = Vec::with_capacity(members.len());
                let mut free = true;
                for &offset in &offsets {
                    let slot = (offset + candidate) % table_size;
                    if used[slot] || targets.contains(&slot) {
                        free = false;
                        break;
                    }
                    targets.push(slot);
                }

                if free {
                    for (member, &slot) in members.iter().zip(&targets) {
                        used[slot] = true;
                        slots[slot] = Some(*member);
                    }
                    displace[bucket] = candidate as u32;
                    break;
                }

                candidate += 1;
            }
        }

        if placed {
            return PerfectHash {
                seed,
                bucket_count,
                table_size,
                displace,
                slots,
            };
        }

        seed += 1;
        if seed > 10_000 {
            assert!(table_size < 1 << 24, "failed to construct a perfect hash");
            table_size *= 2;
            seed = 0;
        }
    }
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

const fn splitmix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
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
