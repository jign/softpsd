#[path = "common/fixtures.rs"]
mod fixtures;

use fixtures::Walker;
use softpsd::{Blend, Channels, Document, Format, Image, Layer, Mask, Node, Rect, rle, validate};
use std::collections::HashSet;

#[test]
fn rle_round_trip() {
    let literals: Vec<u8> = (0..=255).cycle().take(300).collect();
    let mut mixed = vec![42; 100];
    mixed.extend_from_slice(&literals);
    mixed.extend([17; 300]);
    for row in [vec![42; 100], literals, vec![17; 300], mixed, Vec::new()] {
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

#[test]
fn write_smoke_parses() {
    let mut bytes = Vec::new();
    softpsd::write(&fixtures::fixture("smoke").ours, Format::Psd, &mut bytes).unwrap();
    let mut file = Walker(&bytes);
    assert_eq!(file.take(4), b"8BPS");
    assert_eq!(file.u16(), 1);
    file.take(6);
    file.u16();
    file.u32();
    file.u32();
    file.u16();
    file.u16();
    let colour_len = file.u32() as usize;
    file.take(colour_len);
    let resources_len = file.u32() as usize;
    file.take(resources_len);
    let layer_mask_len = file.u32() as usize;
    let mut layer_mask = Walker(file.take(layer_mask_len));
    let layer_info_len = layer_mask.u32() as usize;
    let mut layer_info = Walker(layer_mask.take(layer_info_len));
    let count = i16::from_be_bytes(layer_info.take(2).try_into().unwrap());
    assert_eq!(count, -3);
    let mut names = Vec::new();
    for _ in 0..count.unsigned_abs() {
        layer_info.take(16);
        let channels = layer_info.u16();
        for _ in 0..channels {
            layer_info.u16();
            layer_info.u32();
        }
        layer_info.take(8);
        layer_info.take(4);
        let extra_len = layer_info.u32() as usize;
        let mut extra = Walker(layer_info.take(extra_len));
        let mask_len = extra.u32() as usize;
        extra.take(mask_len);
        let ranges_len = extra.u32() as usize;
        extra.take(ranges_len);
        let name_len = extra.take(1)[0] as usize;
        names.push(String::from_utf8(extra.take(name_len).to_vec()).unwrap());
    }
    assert_eq!(names.len(), 3);
    assert_eq!(names, ["</Layer group>", "Painted", "Group A"]);
}

#[test]
fn fixtures_round_trip() {
    for &name in fixtures::names() {
        let expected = fixtures::fixture(name).ours;
        let format = softpsd::format_for(expected.width, expected.height);
        let mut bytes = Vec::new();
        softpsd::write(&expected, format, &mut bytes).unwrap();
        let actual = softpsd::read(&bytes).unwrap();
        assert!(actual == expected, "fixture {name}: round trip");
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/softpsd-{name}.psd"));
        let bytes = std::fs::read(path).unwrap();
        let actual = softpsd::read(&bytes).unwrap();
        assert!(actual == expected, "fixture {name}: stored file");
    }
}

#[test]
fn fixtures_read_photoshop() {
    for &name in fixtures::names() {
        let expected = fixtures::fixture(name).photoshop;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/ps27-{name}.psd"));
        let bytes = std::fs::read(path).unwrap();
        let actual = softpsd::read(&bytes).unwrap();
        assert!(actual == expected, "fixture {name}: Photoshop");
    }
}

fn smoke_field_positions(bytes: &[u8]) -> (usize, usize) {
    let mut file = Walker(bytes);
    file.take(4);
    file.u16();
    file.take(6);
    file.u16();
    file.u32();
    file.u32();
    let depth = bytes.len() - file.0.len();
    file.u16();
    file.u16();
    let colour_len = file.u32() as usize;
    file.take(colour_len);
    let resources_len = file.u32() as usize;
    file.take(resources_len);
    let layer_mask_len = file.u32() as usize;
    let mut layer_mask = Walker(file.take(layer_mask_len));
    let layer_info_len = layer_mask.u32() as usize;
    let mut layer_info = Walker(layer_mask.take(layer_info_len));
    layer_info.u16();
    layer_info.take(16);
    layer_info.u16();
    layer_info.u16();
    let channel_length = layer_info.0.as_ptr() as usize - bytes.as_ptr() as usize;
    (depth, channel_length)
}

#[test]
fn read_refuses() {
    let mut bytes = Vec::new();
    softpsd::write(&fixtures::fixture("smoke").ours, Format::Psd, &mut bytes).unwrap();
    assert!((0..bytes.len()).step_by(7).all(|length| {
        std::panic::catch_unwind(|| softpsd::read(&bytes[..length]))
            .is_ok_and(|result| result.is_err())
    }));
    let (depth, channel_length) = smoke_field_positions(&bytes);
    let mut bad_depth = bytes.clone();
    bad_depth[depth..depth + 2].copy_from_slice(&16u16.to_be_bytes());
    assert!(matches!(
        softpsd::read(&bad_depth),
        Err(softpsd::Error::Unsupported(_)),
    ));
    bytes[channel_length..channel_length + 4].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(matches!(
        softpsd::read(&bytes),
        Err(softpsd::Error::Malformed(_)),
    ));
}
