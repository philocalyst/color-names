pub use rgb::RGBA;
pub type Rgba8 = RGBA<u8>;

use std::str::FromStr;

// Re-export individual list enums
#[cfg(feature = "basic")]
pub use colorsnamed_basic::Basic;

#[cfg(feature = "html")]
pub use colorsnamed_html::Html;

#[cfg(feature = "xkcd")]
pub use colorsnamed_xkcd::Xkcd;

#[cfg(feature = "x11")]
pub use colorsnamed_x11::X11;

#[cfg(feature = "ral")]
pub use colorsnamed_ral::Ral;

#[cfg(feature = "ridgway")]
pub use colorsnamed_ridgway::Ridgway;

#[cfg(feature = "werner")]
pub use colorsnamed_werner::Werner;

#[cfg(feature = "windows")]
pub use colorsnamed_windows::Windows;

#[cfg(feature = "wikipedia")]
pub use colorsnamed_wikipedia::Wikipedia;

#[cfg(feature = "french")]
pub use colorsnamed_french::French;

#[cfg(feature = "spanish")]
pub use colorsnamed_spanish::Spanish;

#[cfg(feature = "german")]
pub use colorsnamed_german::German;

#[cfg(feature = "hindi")]
pub use colorsnamed_hindi::Hindi;

#[cfg(feature = "risograph")]
pub use colorsnamed_risograph::Risograph;

#[cfg(feature = "chinese-traditional")]
pub use colorsnamed_chinese_traditional::ChineseTraditional;

#[cfg(feature = "japanese-traditional")]
pub use colorsnamed_japanese_traditional::JapaneseTraditional;

#[cfg(feature = "le-corbusier")]
pub use colorsnamed_le_corbusier::LeCorbusier;

#[cfg(feature = "nbs-iscc")]
pub use colorsnamed_nbs_iscc::NbsIscc;

#[cfg(feature = "ntc")]
pub use colorsnamed_ntc::Ntc;

#[cfg(feature = "osxcrayons")]
pub use colorsnamed_osxcrayons::Osxcrayons;

#[cfg(feature = "sanzo-wada-i")]
pub use colorsnamed_sanzo_wada_i::SanzoWadaI;

#[cfg(feature = "thesaurus")]
pub use colorsnamed_thesaurus::Thesaurus;

#[cfg(feature = "complete")]
pub use colorsnamed_complete::Complete;

#[cfg(feature = "short")]
pub use colorsnamed_short::Short;

#[cfg(feature = "best-of")]
pub use colorsnamed_best_of::BestOf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameNotFound;

impl std::fmt::Display for NameNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("no color found with that name")
    }
}

impl std::error::Error for NameNotFound {}

/// A color from any of the enabled color lists.
#[derive(Debug, Clone, PartialEq)]
pub enum AnyColor {
    #[cfg(feature = "basic")]
    Basic(Basic),
    #[cfg(feature = "html")]
    Html(Html),
    #[cfg(feature = "xkcd")]
    Xkcd(Xkcd),
    #[cfg(feature = "x11")]
    X11(X11),
    #[cfg(feature = "ral")]
    Ral(Ral),
    #[cfg(feature = "ridgway")]
    Ridgway(Ridgway),
    #[cfg(feature = "werner")]
    Werner(Werner),
    #[cfg(feature = "windows")]
    Windows(Windows),
    #[cfg(feature = "wikipedia")]
    Wikipedia(Wikipedia),
    #[cfg(feature = "french")]
    French(French),
    #[cfg(feature = "spanish")]
    Spanish(Spanish),
    #[cfg(feature = "german")]
    German(German),
    #[cfg(feature = "hindi")]
    Hindi(Hindi),
    #[cfg(feature = "risograph")]
    Risograph(Risograph),
    #[cfg(feature = "chinese-traditional")]
    ChineseTraditional(ChineseTraditional),
    #[cfg(feature = "japanese-traditional")]
    JapaneseTraditional(JapaneseTraditional),
    #[cfg(feature = "le-corbusier")]
    LeCorbusier(LeCorbusier),
    #[cfg(feature = "nbs-iscc")]
    NbsIscc(NbsIscc),
    #[cfg(feature = "ntc")]
    Ntc(Ntc),
    #[cfg(feature = "osxcrayons")]
    Osxcrayons(Osxcrayons),
    #[cfg(feature = "sanzo-wada-i")]
    SanzoWadaI(SanzoWadaI),
    #[cfg(feature = "thesaurus")]
    Thesaurus(Thesaurus),
    #[cfg(feature = "complete")]
    Complete(Complete),
    #[cfg(feature = "short")]
    Short(Short),
    #[cfg(feature = "best-of")]
    BestOf(BestOf),
}

impl AnyColor {
    pub fn color(&self) -> color::OpaqueColor<color::Srgb> {
        match self {
            #[cfg(feature = "basic")]
            Self::Basic(c) => c.color(),
            #[cfg(feature = "html")]
            Self::Html(c) => c.color(),
            #[cfg(feature = "xkcd")]
            Self::Xkcd(c) => c.color(),
            #[cfg(feature = "x11")]
            Self::X11(c) => c.color(),
            #[cfg(feature = "ral")]
            Self::Ral(c) => c.color(),
            #[cfg(feature = "ridgway")]
            Self::Ridgway(c) => c.color(),
            #[cfg(feature = "werner")]
            Self::Werner(c) => c.color(),
            #[cfg(feature = "windows")]
            Self::Windows(c) => c.color(),
            #[cfg(feature = "wikipedia")]
            Self::Wikipedia(c) => c.color(),
            #[cfg(feature = "french")]
            Self::French(c) => c.color(),
            #[cfg(feature = "spanish")]
            Self::Spanish(c) => c.color(),
            #[cfg(feature = "german")]
            Self::German(c) => c.color(),
            #[cfg(feature = "hindi")]
            Self::Hindi(c) => c.color(),
            #[cfg(feature = "risograph")]
            Self::Risograph(c) => c.color(),
            #[cfg(feature = "chinese-traditional")]
            Self::ChineseTraditional(c) => c.color(),
            #[cfg(feature = "japanese-traditional")]
            Self::JapaneseTraditional(c) => c.color(),
            #[cfg(feature = "le-corbusier")]
            Self::LeCorbusier(c) => c.color(),
            #[cfg(feature = "nbs-iscc")]
            Self::NbsIscc(c) => c.color(),
            #[cfg(feature = "ntc")]
            Self::Ntc(c) => c.color(),
            #[cfg(feature = "osxcrayons")]
            Self::Osxcrayons(c) => c.color(),
            #[cfg(feature = "sanzo-wada-i")]
            Self::SanzoWadaI(c) => c.color(),
            #[cfg(feature = "thesaurus")]
            Self::Thesaurus(c) => c.color(),
            #[cfg(feature = "complete")]
            Self::Complete(c) => c.color(),
            #[cfg(feature = "short")]
            Self::Short(c) => c.color(),
            #[cfg(feature = "best-of")]
            Self::BestOf(c) => c.color(),
        }
    }

    pub fn to_rgba8(&self) -> Rgba8 {
        match self {
            #[cfg(feature = "basic")]
            Self::Basic(c) => c.to_rgba8(),
            #[cfg(feature = "html")]
            Self::Html(c) => c.to_rgba8(),
            #[cfg(feature = "xkcd")]
            Self::Xkcd(c) => c.to_rgba8(),
            #[cfg(feature = "x11")]
            Self::X11(c) => c.to_rgba8(),
            #[cfg(feature = "ral")]
            Self::Ral(c) => c.to_rgba8(),
            #[cfg(feature = "ridgway")]
            Self::Ridgway(c) => c.to_rgba8(),
            #[cfg(feature = "werner")]
            Self::Werner(c) => c.to_rgba8(),
            #[cfg(feature = "windows")]
            Self::Windows(c) => c.to_rgba8(),
            #[cfg(feature = "wikipedia")]
            Self::Wikipedia(c) => c.to_rgba8(),
            #[cfg(feature = "french")]
            Self::French(c) => c.to_rgba8(),
            #[cfg(feature = "spanish")]
            Self::Spanish(c) => c.to_rgba8(),
            #[cfg(feature = "german")]
            Self::German(c) => c.to_rgba8(),
            #[cfg(feature = "hindi")]
            Self::Hindi(c) => c.to_rgba8(),
            #[cfg(feature = "risograph")]
            Self::Risograph(c) => c.to_rgba8(),
            #[cfg(feature = "chinese-traditional")]
            Self::ChineseTraditional(c) => c.to_rgba8(),
            #[cfg(feature = "japanese-traditional")]
            Self::JapaneseTraditional(c) => c.to_rgba8(),
            #[cfg(feature = "le-corbusier")]
            Self::LeCorbusier(c) => c.to_rgba8(),
            #[cfg(feature = "nbs-iscc")]
            Self::NbsIscc(c) => c.to_rgba8(),
            #[cfg(feature = "ntc")]
            Self::Ntc(c) => c.to_rgba8(),
            #[cfg(feature = "osxcrayons")]
            Self::Osxcrayons(c) => c.to_rgba8(),
            #[cfg(feature = "sanzo-wada-i")]
            Self::SanzoWadaI(c) => c.to_rgba8(),
            #[cfg(feature = "thesaurus")]
            Self::Thesaurus(c) => c.to_rgba8(),
            #[cfg(feature = "complete")]
            Self::Complete(c) => c.to_rgba8(),
            #[cfg(feature = "short")]
            Self::Short(c) => c.to_rgba8(),
            #[cfg(feature = "best-of")]
            Self::BestOf(c) => c.to_rgba8(),
        }
    }
}

impl FromStr for AnyColor {
    type Err = NameNotFound;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[cfg(feature = "basic")]
        if let Ok(c) = s.parse::<Basic>() {
            return Ok(Self::Basic(c));
        }
        #[cfg(feature = "html")]
        if let Ok(c) = s.parse::<Html>() {
            return Ok(Self::Html(c));
        }
        #[cfg(feature = "xkcd")]
        if let Ok(c) = s.parse::<Xkcd>() {
            return Ok(Self::Xkcd(c));
        }
        #[cfg(feature = "x11")]
        if let Ok(c) = s.parse::<X11>() {
            return Ok(Self::X11(c));
        }
        #[cfg(feature = "ral")]
        if let Ok(c) = s.parse::<Ral>() {
            return Ok(Self::Ral(c));
        }
        #[cfg(feature = "ridgway")]
        if let Ok(c) = s.parse::<Ridgway>() {
            return Ok(Self::Ridgway(c));
        }
        #[cfg(feature = "werner")]
        if let Ok(c) = s.parse::<Werner>() {
            return Ok(Self::Werner(c));
        }
        #[cfg(feature = "windows")]
        if let Ok(c) = s.parse::<Windows>() {
            return Ok(Self::Windows(c));
        }
        #[cfg(feature = "wikipedia")]
        if let Ok(c) = s.parse::<Wikipedia>() {
            return Ok(Self::Wikipedia(c));
        }
        #[cfg(feature = "french")]
        if let Ok(c) = s.parse::<French>() {
            return Ok(Self::French(c));
        }
        #[cfg(feature = "spanish")]
        if let Ok(c) = s.parse::<Spanish>() {
            return Ok(Self::Spanish(c));
        }
        #[cfg(feature = "german")]
        if let Ok(c) = s.parse::<German>() {
            return Ok(Self::German(c));
        }
        #[cfg(feature = "hindi")]
        if let Ok(c) = s.parse::<Hindi>() {
            return Ok(Self::Hindi(c));
        }
        #[cfg(feature = "risograph")]
        if let Ok(c) = s.parse::<Risograph>() {
            return Ok(Self::Risograph(c));
        }
        #[cfg(feature = "chinese-traditional")]
        if let Ok(c) = s.parse::<ChineseTraditional>() {
            return Ok(Self::ChineseTraditional(c));
        }
        #[cfg(feature = "japanese-traditional")]
        if let Ok(c) = s.parse::<JapaneseTraditional>() {
            return Ok(Self::JapaneseTraditional(c));
        }
        #[cfg(feature = "le-corbusier")]
        if let Ok(c) = s.parse::<LeCorbusier>() {
            return Ok(Self::LeCorbusier(c));
        }
        #[cfg(feature = "nbs-iscc")]
        if let Ok(c) = s.parse::<NbsIscc>() {
            return Ok(Self::NbsIscc(c));
        }
        #[cfg(feature = "ntc")]
        if let Ok(c) = s.parse::<Ntc>() {
            return Ok(Self::Ntc(c));
        }
        #[cfg(feature = "osxcrayons")]
        if let Ok(c) = s.parse::<Osxcrayons>() {
            return Ok(Self::Osxcrayons(c));
        }
        #[cfg(feature = "sanzo-wada-i")]
        if let Ok(c) = s.parse::<SanzoWadaI>() {
            return Ok(Self::SanzoWadaI(c));
        }
        #[cfg(feature = "thesaurus")]
        if let Ok(c) = s.parse::<Thesaurus>() {
            return Ok(Self::Thesaurus(c));
        }
        #[cfg(feature = "complete")]
        if let Ok(c) = s.parse::<Complete>() {
            return Ok(Self::Complete(c));
        }
        #[cfg(feature = "short")]
        if let Ok(c) = s.parse::<Short>() {
            return Ok(Self::Short(c));
        }
        #[cfg(feature = "best-of")]
        if let Ok(c) = s.parse::<BestOf>() {
            return Ok(Self::BestOf(c));
        }
        Err(NameNotFound)
    }
}

impl TryFrom<&str> for AnyColor {
    type Error = NameNotFound;
    fn try_from(s: &str) -> Result<Self, Self::Error> { s.parse() }
}

impl TryFrom<String> for AnyColor {
    type Error = NameNotFound;
    fn try_from(s: String) -> Result<Self, Self::Error> { s.parse() }
}
