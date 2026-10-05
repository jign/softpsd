# Write

```rust
pub fn write(doc: &Document, format: Format, out: &mut impl Write) -> Result<()>;
pub fn format_for(width: u32, height: u32) -> Format;   // Psb above 30,000 on either side
```

## Rules

- Refuse before writing a byte. Validation runs first over the whole document: side limits
  for the format, image data length equals rect area × 4, mask data length equals mask rect
  area, PassThrough only on groups, rects inside i32. Any failure is `Error::Unsupported` or
  `Error::Malformed` and nothing is written.
- The merged image is always written. The caller supplies it; the crate never composites.
- Every layer gets `luni`, `lyid`, and for groups `lsct`. Nothing else. `lyid` counts up from
  1 in file order.
- Channel data is RLE. A row that would grow under PackBits is still written as RLE; a whole
  file is never raw. A layer with an empty rect writes each channel as 2 bytes (compression 0,
  no data).
- Layer pixels are written at the layer's rect, not padded to the document. Nothing is
  trimmed either; the caller decides the rect.
- Masks: default colour is written as given; the flags carry disabled and inverted; "position
  relative to layer" is never set, mask rects are in document space.
- Names: the Pascal name is the UTF-16 name transcoded to MacRoman with `?` for anything
  outside, cut to 31 bytes. `luni` has the full name.
- Blending ranges are the no-op ranges. Flags set bit 3 always, bit 1 for hidden, bit 4 on
  groups. Clipping byte from `clip_to_below`.
- Resolution: 1005 written only when given. ICC: 1039 written only when given, bytes
  untouched. 1057 is always written with writer name `softpsd`.
- Output is streamed in one pass except for section lengths, which are known before each
  section is emitted because every part's size is computable from the model. No seeking,
  so `out` can be a socket or a compressor.

## Not written

16 and 32-bit, CMYK, greyscale, indexed, duotone, vector masks, real user masks, layer
effects, adjustment and fill layers, text, smart objects, blending ranges with content,
layer comps, guides, slices, thumbnails, XMP. Asking for any of them is a type error, not a
runtime one: the model has no field for them.

## Lies the spec tells, and what we do

- Mask data block: the spec's 20 and 36 byte layouts are in the wrong order relative to
  Photoshop's files. We write 20 bytes only.
- Padding after the layer info section: Photoshop and GIMP disagree. We pad to 4 and length
  the section accordingly; readers use the length.
- Tagged block padding: the spec says data is padded to 2, Photoshop writes blocks back to
  back and pads `luni` to 4 inside its own length. We do the same. `lyid` and `lsct` are
  multiples of 4 already.
