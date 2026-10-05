# Model

The data the crate exposes, and the bytes it maps to. Adobe spec sections are named in
brackets. Where Photoshop's output differs from the spec, `doc/lab/photoshop-output.md` wins.

All integers are big-endian. PSD (version 1) and PSB (version 2) differ only where noted:
PSB widens some lengths to 8 bytes and RLE row counts to 4.

## Data model

```rust
pub struct Document {
    pub width: u32,            // 1..=30_000 PSD, 1..=300_000 PSB
    pub height: u32,
    pub channels: Channels,    // Rgb (3) or Rgba (4)
    pub icc_profile: Option<Vec<u8>>,
    pub resolution_dpi: Option<f32>,
    pub layers: Vec<Node>,     // bottom to top, as Photoshop lists them reversed
    pub merged: Image,         // the composite, always present
}

pub enum Node {
    Layer(Layer),
    Group(Group),
}

pub struct Group {
    pub name: String,
    pub visible: bool,
    pub opacity: u8,
    pub blend: Blend,          // PassThrough is valid here only
    pub expanded: bool,
    pub mask: Option<Mask>,
    pub children: Vec<Node>,
}

pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub opacity: u8,           // 0..=255
    pub blend: Blend,
    pub clip_to_below: bool,
    pub pixels: Image,         // RGBA at the layer's own rect
    pub mask: Option<Mask>,
}

pub struct Image {
    pub rect: Rect,            // top, left, bottom, right in document space; may be empty
    pub data: Vec<u8>,         // interleaved RGBA, 4 bytes per pixel, straight alpha
}

pub struct Mask {
    pub rect: Rect,
    pub data: Vec<u8>,         // one byte per pixel, 255 = shown
    pub default: u8,           // value outside rect, 0 or 255
    pub disabled: bool,
}

pub enum Blend { PassThrough, Normal, Dissolve, Darken, Multiply, ColorBurn, LinearBurn,
    DarkerColor, Lighten, Screen, ColorDodge, LinearDodge, LighterColor, Overlay, SoftLight,
    HardLight, VividLight, LinearLight, PinLight, HardMix, Difference, Exclusion, Subtract,
    Divide, Hue, Saturation, Color, Luminosity }
```

Layer pixels are RGBA straight alpha in the API. The file stores planar channels; the writer
splits, the reader interleaves. A layer with an empty rect writes 2-byte channels.

## File layout

Five sections in order [File Header, Color Mode Data, Image Resources, Layer and Mask
Information, Image Data].

### Header (26 bytes)

| Bytes | Field |
| --- | --- |
| 4 | `8BPS` |
| 2 | version: 1 PSD, 2 PSB |
| 6 | zero |
| 2 | channels: 3 or 4 |
| 4 | height |
| 4 | width |
| 2 | depth: 8 |
| 2 | colour mode: 3 (RGB) |

### Colour mode data

4-byte length, 0 for RGB.

### Image resources

4-byte section length, then blocks. Each block: `8BIM`, 2-byte id, Pascal name padded to
even (empty name is `00 00`), 4-byte data length, data padded to even.

| Id | Block | We write |
| --- | --- | --- |
| 1005 | resolution: h-res fixed 16.16, unit u16 (1 = ppi), width unit u16 (1 = inches), then the same for vertical | when `resolution_dpi` is set |
| 1039 | ICC profile bytes | when `icc_profile` is set |
| 1057 | version info: u32 version 1, u8 has-real-merged 1, unicode writer name, unicode reader name, u32 file version 1 | always |

Readers assume sRGB when 1039 is absent.

### Layer and mask information

Length (4 bytes PSD, 8 PSB), then:

1. **Layer info.** Length (4 or 8), then i16 layer count. Negative means the merged image's
   first alpha channel is transparency; we write negative for Rgba, positive for Rgb. Then
   one layer record per layer, then channel image data in the same order. Padded to 4.
2. **Global layer mask info.** 4-byte length, 0.
3. Tagged blocks at document level. We write none.

#### Layer record

| Bytes | Field |
| --- | --- |
| 16 | rect: top, left, bottom, right (i32) |
| 2 | channel count |
| 6 or 10 each | channel: i16 id, length (4 PSD, 8 PSB) of that channel's image data including its 2-byte compression field |
| 4 | `8BIM` |
| 4 | blend key, table below |
| 1 | opacity |
| 1 | clipping: 0 base, 1 clipped to layer below |
| 1 | flags: bit 0 transparency protected, bit 1 hidden, bit 3 "bit 4 is valid" (always set), bit 4 pixels irrelevant to appearance (set on groups) |
| 1 | zero |
| 4 | length of the extra data that follows |

Extra data:

- **Mask data.** 4-byte length. 0 when no mask. Otherwise 20: rect (4 × i32), default
  colour u8, flags u8 (bit 0 position relative to layer, bit 1 disabled, bit 2 "invert",
  which Photoshop ignores, bit 4 parameters follow), 2 bytes zero. Photoshop may write 36 or more (a second "real
  user mask" rect, or parameters); the reader accepts those by length and ignores them.
- **Blending ranges.** 4-byte length, then 8 bytes grey range and 8 per channel. We write
  length 40 with every range `00 00 FF FF 00 00 FF FF` for 4 channels. The reader skips.
- **Name.** Pascal string, MacRoman, padded to 4. Truncated to 31 bytes by Photoshop; the
  unicode block carries the full name.
- **Tagged blocks**, each `8BIM` (or `8B64` in PSB for the keys listed under PSB), 4-byte
  key, length (4, or 8 for the `8B64` keys), data. Blocks are not padded between each other;
  a few keys (`luni` among them) pad their own data to 4 and the length covers that padding.
  The reader advances by the length and never assumes padding.

Channel ids: 0 red, 1 green, 2 blue, -1 transparency, -2 user mask, -3 real user mask
(vector mask raster; never written, skipped on read).

Channel image data, per layer, per channel in record order: u16 compression (0 raw, 1 RLE),
then data. RLE is PackBits per row, preceded by every row's byte count (u16 PSD, u32 PSB).

Blend keys:

| Blend | Key | Blend | Key |
| --- | --- | --- | --- |
| PassThrough | `pass` | Overlay | `over` |
| Normal | `norm` | SoftLight | `sLit` |
| Dissolve | `diss` | HardLight | `hLit` |
| Darken | `dark` | VividLight | `vLit` |
| Multiply | `mul ` | LinearLight | `lLit` |
| ColorBurn | `idiv` | PinLight | `pLit` |
| LinearBurn | `lbrn` | HardMix | `hMix` |
| DarkerColor | `dkCl` | Difference | `diff` |
| Lighten | `lite` | Exclusion | `smud` |
| Screen | `scrn` | Subtract | `fsub` |
| ColorDodge | `div ` | Divide | `fdiv` |
| LinearDodge | `lddg` | Hue | `hue ` |
| LighterColor | `lgCl` | Saturation | `sat ` |
| | | Color | `colr` |
| | | Luminosity | `lum ` |

#### Tagged blocks we use

| Key | Data | On |
| --- | --- | --- |
| `luni` | u32 char count, UTF-16BE name, padded to 4 | every layer |
| `lyid` | u32 unique id | every layer |
| `lsct` | u32 kind: 1 open group, 2 closed group, 3 group end; for kind 1 or 2 also `8BIM` + blend key, which is the group's blend (Photoshop leaves `norm` in the record) | groups and group-end records |
| `clbl` `infx` `knko` | u8 then 3 zero bytes each: `clbl` 1 (blend clipped layers as group), `infx` 0 (blend interior effects), `knko` 0 (knockout none) | pixel layers |

The three blending blocks carry Photoshop's defaults. Without them Photoshop reports a
ColorBurn, ColorDodge or Difference layer's bounds as the whole canvas, measured on a 1 × 1
layer; any one of the three present restores the stored rect. We write all three, as
Photoshop does.

A group is two records: the end marker (kind 3, name `</Layer group>`) comes first in file
order, then the children, then the group itself (kind 1 or 2). That is because the file lists
layers bottom to top. Both records carry four empty channels (length 2 each).

Blocks Photoshop writes that we do not, and why:

| Key | Holds | Why not |
| --- | --- | --- |
| `lnsr` | whether the name was typed or generated | nothing reads it |
| `shmd` | layer metadata, layer-comps flag | no layer comps |
| `fxrp` | Free Transform reference point | Photoshop resets it |
| `lclr` | the colour label on the layer row | wanted: add with a `label` field when a caller has one |
| `lspf` | locks: transparency, pixels, position | wanted: add with a `locks` field when a caller has one |

A reader that does not find these uses the defaults. The reader keeps all of them as opaque
bytes and drops them.

### Image data

u16 compression, then every channel's data in id order (R, G, B, then A for Rgba). For RLE,
all rows' counts for all channels come first, then all rows' data. The merged image is always
written; files without it do not open in most third-party readers.

When the first alpha channel is transparency, Photoshop stores the merged RGB blended over
white: `stored = (c * a + 255 * (255 - a) + 127) / 255`, alpha unchanged. Measured on
`tests/fixtures/ps27-smoke.psd`: `200, 30, 30, 153` is stored as `222, 120, 120, 153`.
Readers undo it (psd-tools `topil()` does). The undo is exact at alpha 128 and above,
lossy below, and colour under alpha 0 is gone.

## PSB

| Where | PSD | PSB |
| --- | --- | --- |
| Header version | 1 | 2 |
| Side limit | 30,000 | 300,000 |
| Layer and mask section length, layer info length | 4 | 8 |
| Channel length in layer record | 4 | 8 |
| RLE row count | u16 | u32 |
| Tagged block length for `LMsk Lr16 Lr32 Layr Mt16 Mt32 Mtrn Alph FMsk lnk2 FEid FXid PxSD cinf` | 4, `8BIM` | 8, `8B64` |

We write none of those keys, so every tagged block we emit stays `8BIM` with a 4-byte length
in both formats.

## Colour

8-bit, gamma-encoded RGB. The file carries the profile in 1039 and nothing else. Photoshop
blends in the document's colour space, gamma-encoded, unless the user has set "Blend RGB
colors using gamma 1.0". A writer whose source blends linearly will see Photoshop recomposite
its layers differently from its merged image. That is the caller's problem; the crate writes
what it is given.
