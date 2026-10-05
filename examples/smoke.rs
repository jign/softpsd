use softpsd::{Blend, Channels, Document, Format, Group, Image, Layer, Mask, Node, Rect};
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() -> softpsd::Result<()> {
    let path = std::env::args_os().nth(1).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "usage: smoke <output.psd>",
        )
    })?;
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
    let doc = Document {
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
    let mut out = BufWriter::new(File::create(path)?);
    softpsd::write(&doc, Format::Psd, &mut out)?;
    out.flush()?;
    Ok(())
}
