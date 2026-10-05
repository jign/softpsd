//! Document tree and pixel buffers.

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub width: u32,
    pub height: u32,
    pub channels: Channels,
    pub icc_profile: Option<Vec<u8>>,
    pub resolution_dpi: Option<f32>,
    pub layers: Vec<Node>, // Bottom to top.
    pub merged: Image,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Layer(Layer),
    Group(Group),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub name: String,
    pub visible: bool,
    pub opacity: u8,
    pub blend: Blend, // PassThrough is valid only on groups.
    pub expanded: bool,
    pub mask: Option<Mask>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub opacity: u8,
    pub blend: Blend,
    pub clip_to_below: bool,
    pub pixels: Image,
    pub mask: Option<Mask>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub rect: Rect,
    pub data: Vec<u8>, // Interleaved RGBA, straight alpha.
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mask {
    pub rect: Rect,
    pub data: Vec<u8>, // One byte per pixel, 255 = shown.
    pub default: u8,   // Value outside the rect, 0 or 255.
    pub disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
}

impl Rect {
    /// Returns zero for an inverted horizontal extent.
    pub fn width(&self) -> usize {
        (i64::from(self.right) - i64::from(self.left)).max(0) as usize
    }

    /// Returns zero for an inverted vertical extent.
    pub fn height(&self) -> usize {
        (i64::from(self.bottom) - i64::from(self.top)).max(0) as usize
    }

    /// Returns None if the pixel count exceeds usize.
    pub fn area(&self) -> Option<usize> {
        self.width().checked_mul(self.height())
    }

    pub fn is_empty(&self) -> bool {
        self.bottom <= self.top || self.right <= self.left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channels {
    Rgb,
    Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blend {
    PassThrough,
    Normal,
    Dissolve,
    Darken,
    Multiply,
    ColorBurn,
    LinearBurn,
    DarkerColor,
    Lighten,
    Screen,
    ColorDodge,
    LinearDodge,
    LighterColor,
    Overlay,
    SoftLight,
    HardLight,
    VividLight,
    LinearLight,
    PinLight,
    HardMix,
    Difference,
    Exclusion,
    Subtract,
    Divide,
    Hue,
    Saturation,
    Color,
    Luminosity,
}
