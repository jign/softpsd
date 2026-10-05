use softpsd::{Blend, Channels, Document, Group, Image, Layer, Mask, Node, Rect};
use std::path::Path;

pub struct Fixture {
    pub ours: Document,
    pub photoshop: Document,
}

pub fn names() -> &'static [&'static str] {
    &["smoke"]
}

pub fn fixture(name: &str) -> Fixture {
    match name {
        "smoke" => smoke(),
        _ => panic!("unknown fixture: {name}"),
    }
}

fn smoke() -> Fixture {
    let layer = Layer {
        name: String::from("Painted"),
        visible: true,
        opacity: 153,
        blend: Blend::Multiply,
        clip_to_below: false,
        pixels: Image {
            rect: Rect {
                top: 8,
                left: 8,
                bottom: 40,
                right: 40,
            },
            data: [200, 30, 30, 255].repeat(32 * 32),
        },
        mask: Some(Mask {
            rect: Rect {
                top: 16,
                left: 16,
                bottom: 48,
                right: 48,
            },
            data: vec![255; 32 * 32],
            default: 0,
            disabled: false,
            inverted: false,
        }),
    };
    let mut merged = vec![0; 64 * 64 * 4];
    for y in 16..40 {
        for x in 16..40 {
            let offset = (y * 64 + x) * 4;
            merged[offset..offset + 4].copy_from_slice(&[200, 30, 30, 153]);
        }
    }
    let ours = Document {
        width: 64,
        height: 64,
        channels: Channels::Rgba,
        icc_profile: None,
        resolution_dpi: None,
        layers: vec![Node::Group(Group {
            name: String::from("Group A"),
            visible: true,
            opacity: 255,
            blend: Blend::Normal,
            expanded: true,
            mask: None,
            children: vec![Node::Layer(layer)],
        })],
        merged: Image {
            rect: Rect {
                top: 0,
                left: 0,
                bottom: 64,
                right: 64,
            },
            data: merged,
        },
    };
    let mut photoshop = ours.clone();
    let Node::Group(group) = &mut photoshop.layers[0] else {
        unreachable!();
    };
    group.blend = Blend::PassThrough;
    photoshop.layers.insert(
        0,
        Node::Layer(Layer {
            name: String::from("Layer 1"),
            visible: true,
            opacity: 255,
            blend: Blend::Normal,
            clip_to_below: false,
            pixels: Image {
                rect: Rect {
                    top: 0,
                    left: 0,
                    bottom: 0,
                    right: 0,
                },
                data: Vec::new(),
            },
            mask: None,
        }),
    );
    photoshop.icc_profile = Some(photoshop_profile("smoke"));
    photoshop.resolution_dpi = Some(72.0);
    Fixture { ours, photoshop }
}

pub struct Walker<'a>(pub &'a [u8]);

impl<'a> Walker<'a> {
    pub fn take(&mut self, len: usize) -> &'a [u8] {
        let (data, remaining) = self.0.split_at(len);
        self.0 = remaining;
        data
    }

    pub fn u16(&mut self) -> u16 {
        u16::from_be_bytes(self.take(2).try_into().unwrap())
    }

    pub fn u32(&mut self) -> u32 {
        u32::from_be_bytes(self.take(4).try_into().unwrap())
    }
}

fn photoshop_profile(name: &str) -> Vec<u8> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures/ps27-{name}.psd"));
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut file = Walker(&bytes);
    file.take(26);
    let colour_len = file.u32() as usize;
    file.take(colour_len);
    let resources_len = file.u32() as usize;
    let mut resources = Walker(file.take(resources_len));
    while !resources.0.is_empty() {
        resources.take(4);
        let id = resources.u16();
        let name_len = usize::from(resources.take(1)[0]);
        resources.take(name_len + (name_len + 1) % 2);
        let length = resources.u32() as usize;
        let data = resources.take(length);
        resources.take(length % 2);
        if id == 1039 {
            return data.to_vec();
        }
    }
    panic!("Photoshop fixture has no ICC profile");
}
