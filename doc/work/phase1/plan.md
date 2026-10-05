# Phase 1: writer vertical slice

Goal: `softpsd::write` produces a file Photoshop opens with the same layer tree as the
model, and Photoshop's PNG export of that file equals the merged image we wrote.

Read first, in this order: `doc/spec/model.md`, `doc/spec/write.md`,
`doc/lab/photoshop-output.md`, `doc/spec/gates.md`. The byte layouts are in model.md and are
not repeated here. Do not read the Adobe spec for layouts; model.md already resolved its
mistakes.

## Rules for this phase

- Only the writer. `src/read.rs` stays a stub.
- No feature beyond what the smoke document needs: 8-bit, RGBA, groups, one raster mask, RLE.
  Rgb documents, PSB, resolution, ICC are typed in the model but may return
  `Error::Unsupported("phase 1")` from the writer.
- No dependencies. Standard library only.
- Comments are short and technical. No comment references a doc.
- Tests: exactly the ones listed in step 9. No test asserts a byte offset, a key string or
  a constant from the spec.
- If Photoshop disagrees with model.md, stop and report the bytes. Do not change the spec
  and do not work around it.
- Every step is a commit. Commit messages say what changed, one line.

## Crate layout

```
src/lib.rs        Format, Error, Result, re-exports
src/model.rs      Document, Node, Group, Layer, Image, Mask, Rect, Channels, Blend
src/blend.rs      Blend <-> 4-byte key
src/rle.rs        PackBits row encode and decode
src/validate.rs   the checks in write.md, run before any byte is written
src/write.rs      the writer
src/read.rs       stub
examples/smoke.rs builds the smoke document and writes it to the path in argv[1]
tests/unit.rs     the tests in step 9
```

## Steps

### 1. model.rs

The types in model.md verbatim. Add to `Rect`: `width()`, `height()`, `area()`,
`is_empty()`. Fields are `i32` top, left, bottom, right. `Image::data.len()` must equal
`rect.area() * 4`; `Mask::data.len()` must equal `rect.area()`. Those are checked in
validate, not asserted in constructors. Derive `Debug, Clone, PartialEq` on everything.

### 2. blend.rs

One `const TABLE: [(Blend, [u8; 4]); 28]`. `Blend::key(self) -> [u8; 4]` and
`Blend::from_key(&[u8; 4]) -> Option<Blend>` both read the table. Keys are in model.md.

### 3. rle.rs

`pub fn encode_row(row: &[u8], out: &mut Vec<u8>)` and
`pub fn decode_row(src: &[u8], width: usize, out: &mut Vec<u8>) -> Result<()>`.

PackBits: a header byte `n` as i8. `0..=127`: copy the next `n + 1` bytes literally.
`-127..=-1`: repeat the next byte `1 - n` times. `-128`: no-op, skip. Encode runs of three
or more equal bytes as a repeat, everything else as literals, literal runs capped at 128
bytes, repeats capped at 128. An empty row encodes to nothing. Decode stops when `width`
bytes are out; more input than needed is `Error::Malformed`, less is `Error::Malformed`.

### 4. validate.rs

`pub fn validate(doc: &Document, format: Format) -> Result<()>`. Checks, in this order,
first failure returns:

1. width and height in `1..=30_000` for Psd, `1..=300_000` for Psb, else
   `Unsupported("side limit")`.
2. `merged.rect` is `0, 0, height, width` and `merged.data.len() == width * height * 4`,
   else `Malformed("merged image")`.
3. Walk every node, depth first. Nesting deeper than 64 is `Malformed("nesting")`.
   For a Layer: `pixels.data.len() == pixels.rect.area() * 4`, else `Malformed("layer
   pixels")`; `blend != PassThrough`, else `Malformed("pass through on a layer")`.
   For a Group: nothing beyond the walk.
   For any mask: `data.len() == rect.area()`, `default` is 0 or 255, else `Malformed("mask")`.
4. Every rect has `bottom >= top` and `right >= left`, else `Malformed("rect")`.

Phase 1 additionally returns `Unsupported("phase 1")` for `Channels::Rgb`, `Format::Psb`,
`icc_profile.is_some()`, `resolution_dpi.is_some()`.

### 5. Flatten the tree to file order

In `write.rs`, a private `enum Record<'a> { Layer(&'a Layer), GroupEnd, GroupStart(&'a Group) }`
and `fn flatten(nodes: &[Node], out: &mut Vec<Record>)`. For each node in order: a Layer
pushes `Layer`; a Group pushes `GroupEnd`, then recurses into its children, then pushes
`GroupStart(group)`. The result is bottom to top, which is the file's order. The model's
`layers` vector is already bottom to top.

### 6. Channel data

Before any record is written, every channel of every record is RLE-compressed into memory,
because each layer record stores its channels' byte lengths ahead of the data.

For a `Layer` with a non-empty rect: four channels in order `-1, 0, 1, 2` (alpha first,
as Photoshop writes). Deinterleave `pixels.data` into four planes, encode each row, and
build the channel's bytes as: all row counts (u16 each), then all encoded rows, prefixed
with `00 01` (compression 1). If a `Layer` or `GroupStart` has a mask, a channel `-2` from
`mask.data` at the mask's own rect, same encoding, after the four pixel channels.

For an empty-rect layer, and for `GroupEnd` and `GroupStart`: four channels `-1, 0, 1, 2`,
each exactly `00 00` (compression 0, no data).

### 7. The writer

`pub fn write(doc: &Document, format: Format, out: &mut impl std::io::Write) -> Result<()>`.
Call `validate` first. Then emit in order, every integer big-endian:

1. Header, 26 bytes, model.md.
2. Colour mode data: `00 00 00 00`.
3. Image resources: a u32 section length, then one block, 1057 version info: `8BIM`,
   u16 1057, name `00 00`, u32 data length, data = u32 1, u8 1, unicode `softpsd`,
   unicode `softpsd`, u32 1. Unicode string is u32 char count then UTF-16BE, no padding
   inside a resource. Pad the data to even.
4. Layer and mask section. Compute its length from the parts below before writing it.
   Then: u32 layer info length; i16 layer count, negative (`-records.len()`); one record
   per flattened entry; the channel bytes per record in the same order; pad the layer info
   to 4; u32 0 for global layer mask info. No document-level tagged blocks.
5. Image data: `00 01`, then all row counts of all four merged planes R, G, B, A (u16
   each, plane by plane), then all rows' encoded data, plane by plane. The R, G, B planes
   are blended over white first, write.md has the formula; A is written as is.

Layer record, per model.md. Fill in: rect from `pixels.rect` (zero rect for groups and
group ends); channel count and the `(i16 id, u32 length)` pairs where length is the
channel's byte length including its 2-byte compression field; `8BIM`; blend key
(`norm` for `GroupEnd`, the group's blend for `GroupStart`); opacity (255 for `GroupEnd`);
clipping byte; flags: `0x08`, plus `0x02` if hidden, plus `0x10` for `GroupStart` and
`GroupEnd`; a zero byte; u32 extra length; then the extra data:

- Mask block: `00 00 00 00` when none. Else u32 20, mask rect as four i32, default u8,
  flags u8 (`0x02` disabled, `0x04` inverted, nothing else), `00 00`.
- Blending ranges: u32 40, then five times `00 00 FF FF 00 00 FF FF`.
- Pascal name: u8 length, MacRoman bytes (ASCII in phase 1; replace non-ASCII with `?`),
  cut to 31, padded so length byte plus bytes is a multiple of 4. `</Layer group>` for
  `GroupEnd`.
- `8BIM` `luni` u32 len, u32 char count, UTF-16BE name, padded to 4, len covers the padding.
- `8BIM` `lyid` u32 4, u32 id, counting from 1 in record order.
- For `GroupStart`: `8BIM` `lsct` u32 12, u32 1 (open) or 2 (closed, when `!expanded`),
  `8BIM`, blend key. For `GroupEnd`: `8BIM` `lsct` u32 4, u32 3.

Sizes: write each record's extra data into a `Vec<u8>` first to know its length, then
emit. Do not seek on `out`.

### 8. examples/smoke.rs

Build the smoke document and write it to `argv[1]`. 64 × 64, Rgba. One group "Group A",
expanded, Normal, opacity 255, visible, containing one layer "Painted": rect 8, 8, 40, 40,
every pixel `200, 30, 30, 255`, opacity 153, Multiply, visible, no clipping, mask rect
16, 16, 48, 48 all 255 with default 0, not disabled, not inverted. Merged image: every
pixel `0, 0, 0, 0` except rows 16..40 and columns 16..40, which are `200, 30, 30, 153`.

That merged image is the correct composite: the layer is alone over transparency, so
Multiply has nothing beneath it and the result is the source colour at the layer's opacity,
cut by the mask. The file stores its RGB over white, `222, 120, 120, 153`; psd-tools and
Photoshop undo that, so the PNG export and `compare-merged.py` see `200, 30, 30, 153`.

### 9. Tests, tests/unit.rs

Four tests, no more:

- `rle_round_trip`: a row with a long run, a literal stretch, a 300-byte run (crosses the
  128 cap), and an empty row each encode then decode to themselves.
- `blend_keys_round_trip`: every `Blend` goes to a key and back to itself; all 28 keys are
  distinct.
- `validate_refuses`: a layer with PassThrough, a mask with a wrong data length and a
  31_000-wide Psd each return `Err`, and the same document with Normal and no mask returns
  `Ok`. One test, four asserts.
- `write_smoke_parses`: run the smoke writer into a `Vec<u8>`, then check the file
  through a tiny private walker: signature, version 1, the layer count is -3, three
  records, and the names in the order `</Layer group>`, `Painted`, `Group A`. This walker
  reads lengths and skips; it is not the reader and lives in the test file.

Before trusting `write_smoke_parses`, flip the sign of the layer count in the writer once
and confirm it fails, then restore.

### 10. tools/photoshop/dump.jsx

Opens the file at `OUT`, returns a text tree, then exports a PNG to `OUT + ".png"` and
closes without saving. One line per layer, depth by two-space indent, top of the stack
first as Photoshop lists it:

```
<group|layer> '<name>' opacity=<0..255> blend=<BlendMode name> visible=<true|false> bounds=<l>,<t>,<r>,<b>[ mask=<l>,<t>,<r>,<b>]
```

Opacity in Photoshop's scripting is 0..100 as a float; multiply by 2.55 and round. Mask
bounds come from selecting the mask channel: `doc.activeLayer = layer`, then Action
Manager `slct` on the mask channel and `doc.selection.bounds`, or report `mask=?` if the
API cannot give it and say so in the handoff. PNG export: `doc.exportDocument(file,
ExportType.SAVEFORWEB, opts)` with `opts.format = SaveDocumentType.PNG`,
`opts.PNG8 = false`, `opts.transparency = true`. Follow the rules in
`doc/lab/photoshop-scripting.md`.

### 11. tools/readers/compare-merged.py

`python compare-merged.py <psd> <png>`: loads the PSD's stored merged image with psd-tools
(`PSDImage.open(p).topil()`, which is the stored composite, not a recomposite) and the PNG
with PIL, both as RGBA, and prints the count of pixels that differ by more than 1 in any
channel, and the first differing coordinate. Exit code 1 when the count is not zero.

### 12. tools/writer-gate.ps1

`writer-gate.ps1 <psd>`: runs, in order, `psd-tools-dump.py`, `ag-psd-dump.js`,
`photoshop/run.ps1 photoshop/dump.jsx -Out <psd>`, then `compare-merged.py <psd> <psd>.png`.
Prints each output under a header. Fails on the first non-zero exit.

## Gate

```
cargo run --example smoke -- target/softpsd-smoke.psd
tools\writer-gate.ps1 target\softpsd-smoke.psd
```

Passes when all three dumps show the group holding the layer with Multiply, opacity 153
(0.6 in ag-psd), bounds 8,8,40,40, mask 16,16,48,48, and compare-merged reports zero
differing pixels. Then copy the file to `tests/fixtures/softpsd-smoke.psd` and commit it.

## Handoff

Write `doc/work/phase1/report.md`: what passed, the exact dump outputs, anything Photoshop
did that model.md did not predict, with bytes. Keep it under a page.
