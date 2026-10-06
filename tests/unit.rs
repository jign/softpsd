#[path = "common/fixtures.rs"]
mod fixtures;

use fixtures::Walker;
use softpsd::{Blend, Channels, Document, Format, Image, Layer, Mask, Node, Rect};

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
    assert!(softpsd::write(&doc, Format::Psd, &mut std::io::sink()).is_ok());

    layer.blend = Blend::PassThrough;
    doc.layers = vec![Node::Layer(layer.clone())];
    assert!(softpsd::write(&doc, Format::Psd, &mut std::io::sink()).is_err());

    layer.blend = Blend::Normal;
    layer.mask = Some(Mask {
        rect,
        data: Vec::new(),
        default: 0,
        disabled: false,
    });
    doc.layers = vec![Node::Layer(layer)];
    assert!(softpsd::write(&doc, Format::Psd, &mut std::io::sink()).is_err());

    doc.layers.clear();
    doc.width = 31_000;
    doc.merged.rect.right = 31_000;
    doc.merged.data.resize(31_000 * 4, 0);
    assert!(softpsd::write(&doc, Format::Psd, &mut std::io::sink()).is_err());
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
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "tests/fixtures/softpsd-{name}.{}",
            fixtures::extension(name)
        ));
        let bytes = std::fs::read(path).unwrap();
        let actual = softpsd::read(&bytes).unwrap();
        assert!(actual == expected, "fixture {name}: stored file");
    }
}

#[test]
fn fixtures_read_photoshop() {
    for &name in fixtures::names() {
        let expected = fixtures::fixture(name).photoshop;
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "tests/fixtures/ps27-{name}.{}",
            fixtures::extension(name)
        ));
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

