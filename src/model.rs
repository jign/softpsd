//! Document tree and pixel buffers.

/// A layered document: the layer tree and the flattened image Photoshop shows without
/// recompositing.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Whether the document has transparency.
    pub channels: Channels,
    /// ICC profile bytes, stored untouched.
    pub icc_profile: Option<Vec<u8>>,
    /// Resolution in pixels per inch.
    pub resolution_dpi: Option<f32>,
    /// Top-level layers and groups, bottom to top.
    pub layers: Vec<Node>,
    /// The flattened image, page-sized. The caller composites it; softpsd does not.
    pub merged: Image,
}

/// One entry in the layer tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// A pixel layer.
    Layer(Layer),
    /// A group of nodes.
    Group(Group),
}

/// A layer group.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    /// Name as Photoshop shows it.
    pub name: String,
    /// Eye toggle.
    pub visible: bool,
    /// 0 to 255.
    pub opacity: u8,
    /// `PassThrough` is valid only on groups.
    pub blend: Blend,
    /// Open in Photoshop's layers panel.
    pub expanded: bool,
    /// Group mask.
    pub mask: Option<Mask>,
    /// Children, bottom to top.
    pub children: Vec<Node>,
}

/// A pixel layer.
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    /// Name as Photoshop shows it.
    pub name: String,
    /// Eye toggle.
    pub visible: bool,
    /// 0 to 255.
    pub opacity: u8,
    /// Any mode except `PassThrough`.
    pub blend: Blend,
    /// Clipping mask onto the layer below.
    pub clip_to_below: bool,
    /// Pixels and their placement on the page.
    pub pixels: Image,
    /// Layer mask.
    pub mask: Option<Mask>,
}

/// Pixels placed on the page.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    /// Placement in page coordinates. May extend past the page.
    pub rect: Rect,
    /// Interleaved RGBA, straight alpha, row-major, `rect.area() * 4` bytes.
    pub data: Vec<u8>,
}

/// A raster mask.
#[derive(Debug, Clone, PartialEq)]
pub struct Mask {
    /// Placement in page coordinates.
    pub rect: Rect,
    /// One byte per pixel, 255 shown, row-major, `rect.area()` bytes.
    pub data: Vec<u8>,
    /// Value outside the rect: 0 or 255.
    pub default: u8,
    /// Kept in the file but not applied.
    pub disabled: bool,
}

/// A rectangle in page pixels. `bottom` and `right` are exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// First row.
    pub top: i32,
    /// First column.
    pub left: i32,
    /// One past the last row.
    pub bottom: i32,
    /// One past the last column.
    pub right: i32,
}

impl Rect {
    /// Returns zero for an inverted horizontal extent. Saturates at `usize::MAX` on 32-bit targets.
    pub fn width(&self) -> usize {
        extent(self.left, self.right)
    }

    /// Returns zero for an inverted vertical extent. Saturates at `usize::MAX` on 32-bit targets.
    pub fn height(&self) -> usize {
        extent(self.top, self.bottom)
    }

    /// Returns None if the pixel count exceeds usize.
    pub fn area(&self) -> Option<usize> {
        self.width().checked_mul(self.height())
    }

    /// True when the rect covers no pixels.
    pub fn is_empty(&self) -> bool {
        self.bottom <= self.top || self.right <= self.left
    }
}

fn extent(start: i32, end: i32) -> usize {
    let span = i64::from(end).saturating_sub(i64::from(start)).max(0);
    usize::try_from(span).unwrap_or(usize::MAX)
}

/// Document colour channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channels {
    /// Opaque: the merged image's alpha is ignored.
    Rgb,
    /// Transparent: the merged image keeps its alpha.
    Rgba,
}

/// Photoshop's blend modes, named as in its layers panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[allow(
    missing_docs,
    reason = "each variant is the Photoshop mode of the same name"
)]
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
