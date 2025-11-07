This Rust crate, `color-names`, provides a comprehensive and efficiently searchable collection of color names. It's designed to help developers easily associate hexadecimal color values or RGB triplets with human-readable names, drawing from a vast, curated dataset.

### Core Features

-   **Extensive Color Name Dataset:** The crate leverages a meticulously assembled list of over 29,956 unique color names, sourced from various references and user contributions, provided by the `meodai/color-names` project.
-   **Multiple Color Sets:** It organizes colors into different sets (e.g., "complete", "bestOf", "short"), allowing for focused searches and reduced bundle size if only specific sets are needed.
-   **Efficient Nearest Color Search (Implied):** While the provided Rust code snippet doesn't explicitly detail the nearest color search algorithm, the reference to `meodai/color-names` and `vycdev/ColorNamesSharp` suggests that efficient methods like K-D trees and CIELAB color space conversions are either implemented or intended for high accuracy and performance when finding the closest named color to a given input.
-   **Flexible Color Representation:** Colors can be represented and retrieved as:
    -   Hexadecimal strings (`#RRGGBB`).
    -   RGB tuples (`(r, g, b)`).
    -   `color::OpaqueColor` for integration with the `color` crate.
    -   `rgb::Rgb` for integration with the `rgb` crate.
-   **Parsing from Strings:** The crate allows parsing color names directly from strings, making it convenient to convert user input or configuration values into `color-names` enums.
-   **Serde Integration (Optional):** With the `serde` feature enabled, color enums can be serialized and deserialized, facilitating their use in data structures and network communication.
-   **Generated Code:** The crate uses a `build.rs` script to generate Rust enums and associated implementations directly from the color data, ensuring up-to-date and type-safe access to the color names.

### Key Components

-   **`colors.rs` (Generated):** This module contains all the generated Rust enums for each color set, along with `impl` blocks for retrieving hex values, color names, and converting to `color::OpaqueColor` or `rgb::Rgb`. It also includes `FromStr` and `TryFrom` implementations for parsing color names from strings.
-   **`ColorSet` Enum:** A master enum that enumerates all available color sets within the library.
-   **Color Enums (e.g., `Complete`, `BestOf`, `Short`):** Each color set has its own dedicated enum, where each variant represents a specific color by its sanitized name.
-   **`HexParseError` Enum:** Defines the error types that can occur during color name parsing, such as invalid length or character in a hex string, or when a color name is not found.
-   **`sanitize_identifier` Function:** A utility function used during code generation to convert human-readable color names into valid Rust identifiers, handling special characters and leading digits.

### Usage

**Installation:**

Add `color-names` to your `Cargo.toml`:

```toml
[dependencies]
color-names = "0.1.0" # Use the latest version
rgb = "0.8" # For RGB color representation
color = "0.1" # For a more generalized color representation
serde = { version = "1.0", features = ["derive"], optional = true }
```

**Basic Example:**

```rust
use color_names::{Complete, HexParseError};
use hex::ToHex;

fn main() -> Result<(), HexParseError> {
    // Get hex value
    let white_hex = Complete::White.hex();
    println!("White hex: {}", white_hex); // => #ffffff

    // Get color name
    let classic_rose_name = Complete::ClassicRose.color_name();
    println!("Classic Rose name: {}", classic_rose_name); // => Classic Rose

    // Parse from string
    let parsed_color: Complete = "Eigengrau".parse()?;
    println!("Parsed color hex: {}", parsed_color.hex()); // => #16161d

    // Convert to RGB
    let fairy_tale_rgb = Complete::FairyTale.rgb();
    println!("Fairy Tale RGB: R:{}, G:{}, B:{}", fairy_tale_rgb.r, fairy_tale_rgb.g, fairy_tale_rgb.b); // => R:241, G:193, B:209

    // Convert to a generic color type (using the 'color' crate)
    let stoic_white_color: color::OpaqueColor<color::Srgb> = Complete::StoicWhite.color();
    println!("Stoic White Color (normalized components): {:?}", stoic_white_color.components); // => [0.8784314, 0.8784314, 1.0]

    // Get uppercase hex
    let silky_pink_hex_upper: String = Complete::SilkyPink.encode_hex_upper();
    println!("Silky Pink uppercase hex: {}", silky_pink_hex_upper); // => FFDBF0

    Ok(())
}
```
