# Read

```rust
pub fn read(input: &[u8]) -> Result<Document>;
pub fn read_header(input: &[u8]) -> Result<Header>;   // cheap: format, size, depth, mode
```

## Rules

- Accept what `write.md` emits plus what Photoshop writes for the same content. Everything
  else is `Error::Unsupported` with the reason named. The reader never guesses.
- Refused at the header: depth other than 8, colour mode other than RGB, channel count other
  than 3 or 4.
- Refused per layer: a channel id outside {0, 1, 2, -1, -2, -3}; compression other than 0
  or 1 (ZIP is 2 and 3, Photoshop writes it for 16-bit only); a `lsct` kind outside 0..=3.
- Skipped, not refused: channel -3, every tagged block not in the model, every image
  resource other than 1005 and 1039, blending ranges, mask block bytes past 20, global
  layer mask info, document-level tagged blocks. Skipping is by stored length, never by
  assumed size.
- Layers that are neither pixel nor group (`lsct` kind 0, or a layer carrying an
  adjustment, fill, text or smart object key) are refused by name, so an artist learns
  which layer stopped the import.
- Layer pixels are interleaved to RGBA straight alpha at the layer's rect. A layer with no
  -1 channel gets alpha 255. Rgb documents give alpha 255 in the merged image.
- Group nesting is rebuilt from `lsct` kinds; an unbalanced file is `Error::Malformed`.
- Bounded. Every length is checked against the remaining input before it is used; the
  decoder allocates from the header's dimensions, never from a length field alone. RLE
  decode stops at the row's byte count and the row's width, whichever comes first, and a
  short row is `Error::Malformed`. No recursion deeper than the group nesting, which is
  capped at 64.

## Growth

The reader grows from issues with the file attached. A new file kind is added only with its
fixture under `tests/fixtures/` and a row in `gates.md` in this folder. 16-bit is the likely first.
