//! Blend mode keys.

use crate::Blend;

const ALL: [Blend; 28] = [
    Blend::PassThrough,
    Blend::Normal,
    Blend::Dissolve,
    Blend::Darken,
    Blend::Multiply,
    Blend::ColorBurn,
    Blend::LinearBurn,
    Blend::DarkerColor,
    Blend::Lighten,
    Blend::Screen,
    Blend::ColorDodge,
    Blend::LinearDodge,
    Blend::LighterColor,
    Blend::Overlay,
    Blend::SoftLight,
    Blend::HardLight,
    Blend::VividLight,
    Blend::LinearLight,
    Blend::PinLight,
    Blend::HardMix,
    Blend::Difference,
    Blend::Exclusion,
    Blend::Subtract,
    Blend::Divide,
    Blend::Hue,
    Blend::Saturation,
    Blend::Color,
    Blend::Luminosity,
];

impl Blend {
    pub(crate) const fn key(self) -> [u8; 4] {
        match self {
            Blend::PassThrough => *b"pass",
            Blend::Normal => *b"norm",
            Blend::Dissolve => *b"diss",
            Blend::Darken => *b"dark",
            Blend::Multiply => *b"mul ",
            Blend::ColorBurn => *b"idiv",
            Blend::LinearBurn => *b"lbrn",
            Blend::DarkerColor => *b"dkCl",
            Blend::Lighten => *b"lite",
            Blend::Screen => *b"scrn",
            Blend::ColorDodge => *b"div ",
            Blend::LinearDodge => *b"lddg",
            Blend::LighterColor => *b"lgCl",
            Blend::Overlay => *b"over",
            Blend::SoftLight => *b"sLit",
            Blend::HardLight => *b"hLit",
            Blend::VividLight => *b"vLit",
            Blend::LinearLight => *b"lLit",
            Blend::PinLight => *b"pLit",
            Blend::HardMix => *b"hMix",
            Blend::Difference => *b"diff",
            Blend::Exclusion => *b"smud",
            Blend::Subtract => *b"fsub",
            Blend::Divide => *b"fdiv",
            Blend::Hue => *b"hue ",
            Blend::Saturation => *b"sat ",
            Blend::Color => *b"colr",
            Blend::Luminosity => *b"lum ",
        }
    }

    pub(crate) fn from_key(key: &[u8; 4]) -> Option<Blend> {
        ALL.into_iter().find(|blend| &blend.key() == key)
    }
}
