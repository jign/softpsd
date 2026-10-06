//! Bench documents, shared with `tools/compare-agpsd`. Deterministic.

use softpsd::{Blend, Channels, Document, Group, Image, Layer, Mask, Node, Rect};

pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self(seed.max(1))
    }

    pub fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }

    pub fn below(&mut self, n: u32) -> u32 {
        self.next() % n.max(1)
    }
}

fn layer(name: String, pixels: Image) -> Layer {
    Layer {
        name,
        visible: true,
        opacity: 255,
        blend: Blend::Normal,
        clip_to_below: false,
        pixels,
        mask: None,
    }
}

fn page(side: u32) -> Rect {
    Rect {
        top: 0,
        left: 0,
        bottom: side as i32,
        right: side as i32,
    }
}

fn merged(side: u32, rng: &mut Rng) -> Image {
    let s = side as usize;
    let mut data = vec![0u8; s * s * 4];
    for (i, pixel) in data.chunks_exact_mut(4).enumerate() {
        let (x, y) = (i % s, i / s);
        let n = (rng.next() & 3) as u8;
        pixel.copy_from_slice(&[(x * 255 / s) as u8 ^ n, (y * 255 / s) as u8, 160, 255]);
    }
    Image {
        rect: page(side),
        data,
    }
}

/// 26 layers covering half to all of the page, 40% painted with noisy pixels. Worst case for RLE.
pub fn dense(side: u32) -> Document {
    let mut rng = Rng::new(0x1234_5678);
    let mut layers = Vec::new();
    for i in 0..26usize {
        let w = side / 2 + rng.below(side / 2);
        let h = side / 2 + rng.below(side / 2);
        let left = rng.below(side - w + 1) as i32;
        let top = rng.below(side - h + 1) as i32;
        let rect = Rect {
            top,
            left,
            bottom: top + h as i32,
            right: left + w as i32,
        };
        let (w, h) = (w as usize, h as usize);
        let mut data = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                if (x / 37 + y / 53 + i) % 5 < 2 {
                    let o = (y * w + x) * 4;
                    let n = (rng.next() & 7) as u8;
                    data[o..o + 4].copy_from_slice(&[
                        (x % 256) as u8 ^ n,
                        (y % 256) as u8,
                        (i * 9) as u8,
                        200 + n,
                    ]);
                }
            }
        }
        layers.push(Node::Layer(layer(
            format!("Layer {i}"),
            Image { rect, data },
        )));
    }
    Document {
        width: side,
        height: side,
        channels: Channels::Rgba,
        icc_profile: None,
        resolution_dpi: Some(72.0),
        layers,
        merged: merged(side, &mut rng),
    }
}

// Round brushes stamped along random lines, cropped to the painted bounds.
fn strokes(side: u32, rng: &mut Rng, colour: [u8; 3]) -> Image {
    let w = side / 10 + rng.below(side * 3 / 10);
    let h = side / 10 + rng.below(side * 3 / 10);
    let left = rng.below(side - w) as i32;
    let top = rng.below(side - h) as i32;
    let (wu, hu) = (w as usize, h as usize);
    let mut data = vec![0u8; wu * hu * 4];
    let radius = (side / 400).max(2) as i32;
    for _ in 0..12 {
        let (mut x, mut y) = (rng.below(w) as i32, rng.below(h) as i32);
        let (dx, dy) = (rng.below(5) as i32 - 2, rng.below(5) as i32 - 2);
        for _ in 0..(w.max(h) / 2) {
            for oy in -radius..=radius {
                for ox in -radius..=radius {
                    let d2 = ox * ox + oy * oy;
                    let (px, py) = (x + ox, y + oy);
                    if d2 > radius * radius || px < 0 || py < 0 || px >= w as i32 || py >= h as i32
                    {
                        continue;
                    }
                    let alpha = (255 - d2 * 200 / (radius * radius)) as u8;
                    let o = (py as usize * wu + px as usize) * 4;
                    if data[o + 3] < alpha {
                        data[o..o + 4].copy_from_slice(&[colour[0], colour[1], colour[2], alpha]);
                    }
                }
            }
            x += dx;
            y += dy;
        }
    }
    let rect = Rect {
        top,
        left,
        bottom: top + h as i32,
        right: left + w as i32,
    };
    Image { rect, data }
}

/// The phase 5 export's shape: 26 stroke layers cropped to their bounds, one group of five,
/// a hidden layer, and one layer masked by a page-sized mask.
pub fn artwork(side: u32) -> Document {
    let mut rng = Rng::new(0x0bad_cafe);
    let stroke_layer = |i: usize, rng: &mut Rng| {
        let colour = [rng.next() as u8, rng.next() as u8, rng.next() as u8];
        layer(format!("Stroke {i}"), strokes(side, rng, colour))
    };
    let mut layers: Vec<Node> = (0..20)
        .map(|i| Node::Layer(stroke_layer(i, &mut rng)))
        .collect();
    let children = (20..25)
        .map(|i| Node::Layer(stroke_layer(i, &mut rng)))
        .collect();
    layers.push(Node::Group(Group {
        name: String::from("Folder"),
        visible: true,
        opacity: 255,
        blend: Blend::PassThrough,
        expanded: true,
        mask: None,
        children,
    }));
    if let Node::Layer(hidden) = &mut layers[3] {
        hidden.visible = false;
    }
    let mut masked = stroke_layer(25, &mut rng);
    let s = side as usize;
    let mask = (0..s * s).map(|i| ((i % s) * 255 / s) as u8).collect();
    masked.mask = Some(Mask {
        rect: page(side),
        data: mask,
        default: 0,
        disabled: false,
    });
    layers.push(Node::Layer(masked));
    Document {
        width: side,
        height: side,
        channels: Channels::Rgba,
        icc_profile: None,
        resolution_dpi: Some(72.0),
        layers,
        merged: merged(side, &mut rng),
    }
}
