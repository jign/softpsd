//! Blend mode keys.

use crate::Blend;

const TABLE: [(Blend, [u8; 4]); 28] = [
    (Blend::PassThrough, *b"pass"),
    (Blend::Normal, *b"norm"),
    (Blend::Dissolve, *b"diss"),
    (Blend::Darken, *b"dark"),
    (Blend::Multiply, *b"mul "),
    (Blend::ColorBurn, *b"idiv"),
    (Blend::LinearBurn, *b"lbrn"),
    (Blend::DarkerColor, *b"dkCl"),
    (Blend::Lighten, *b"lite"),
    (Blend::Screen, *b"scrn"),
    (Blend::ColorDodge, *b"div "),
    (Blend::LinearDodge, *b"lddg"),
    (Blend::LighterColor, *b"lgCl"),
    (Blend::Overlay, *b"over"),
    (Blend::SoftLight, *b"sLit"),
    (Blend::HardLight, *b"hLit"),
    (Blend::VividLight, *b"vLit"),
    (Blend::LinearLight, *b"lLit"),
    (Blend::PinLight, *b"pLit"),
    (Blend::HardMix, *b"hMix"),
    (Blend::Difference, *b"diff"),
    (Blend::Exclusion, *b"smud"),
    (Blend::Subtract, *b"fsub"),
    (Blend::Divide, *b"fdiv"),
    (Blend::Hue, *b"hue "),
    (Blend::Saturation, *b"sat "),
    (Blend::Color, *b"colr"),
    (Blend::Luminosity, *b"lum "),
];

impl Blend {
    pub fn key(self) -> [u8; 4] {
        TABLE
            .iter()
            .find(|(blend, _)| *blend == self)
            .map(|(_, key)| *key)
            .expect("every blend has a key")
    }

    pub fn from_key(key: &[u8; 4]) -> Option<Blend> {
        TABLE
            .iter()
            .find(|(_, candidate)| candidate == key)
            .map(|(blend, _)| *blend)
    }
}
