use softpsd::{Blend, rle};
use std::collections::HashSet;

// TODO: validate_refuses, write_smoke_parses.

#[test]
fn rle_round_trip() {
    let mut row = vec![42; 100];
    row.extend((0..=255).cycle().take(300));
    row.extend([17; 300]);
    for row in [row, Vec::new()] {
        let mut encoded = Vec::new();
        rle::encode_row(&row, &mut encoded);
        let mut decoded = Vec::new();
        rle::decode_row(&encoded, row.len(), &mut decoded).unwrap();
        assert_eq!(decoded, row);
    }
}

#[test]
fn blend_keys_round_trip() {
    let blends = [
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
    let mut keys = HashSet::new();
    for blend in blends {
        let key = blend.key();
        assert_eq!(Blend::from_key(&key), Some(blend));
        assert!(keys.insert(key));
    }
}
