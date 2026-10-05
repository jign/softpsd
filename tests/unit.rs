use softpsd::{Blend, Channels, Document, Format, Image, Layer, Mask, Node, Rect, rle, validate};
use std::collections::HashSet;

// TODO: write_smoke_parses.

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

#[test]
fn validate_refuses() {
    let rect = Rect {
        top: 0,
        left: 0,
        bottom: 1,
        right: 1,
    };
    let image = Image {
        rect,
        data: vec![0, 0, 0, 0],
    };
    let mut layer = Layer {
        name: String::from("Painted"),
        visible: true,
        opacity: 255,
        blend: Blend::Normal,
        clip_to_below: false,
        pixels: image.clone(),
        mask: None,
    };
    let mut doc = Document {
        width: 1,
        height: 1,
        channels: Channels::Rgba,
        icc_profile: None,
        resolution_dpi: None,
        layers: vec![Node::Layer(layer.clone())],
        merged: image,
    };
    assert!(validate::validate(&doc, Format::Psd).is_ok());

    layer.blend = Blend::PassThrough;
    doc.layers = vec![Node::Layer(layer.clone())];
    assert!(validate::validate(&doc, Format::Psd).is_err());

    layer.blend = Blend::Normal;
    layer.mask = Some(Mask {
        rect,
        data: Vec::new(),
        default: 0,
        disabled: false,
        inverted: false,
    });
    doc.layers = vec![Node::Layer(layer)];
    assert!(validate::validate(&doc, Format::Psd).is_err());

    doc.layers.clear();
    doc.width = 31_000;
    doc.merged.rect.right = 31_000;
    doc.merged.data.resize(31_000 * 4, 0);
    assert!(validate::validate(&doc, Format::Psd).is_err());
}
