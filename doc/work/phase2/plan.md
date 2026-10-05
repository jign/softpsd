# Phase 2: reader

Goal: `softpsd::read` turns a file written by phase 1 back into the model it came from, and
reads `tests/fixtures/ps27-smoke.psd` into the tree Photoshop shows for it, with layer
pixels equal to what psd-tools decodes.

Read first, in this order: `doc/spec/read.md`, `doc/spec/model.md` (file layout), and
`doc/lab/photoshop-output.md`. `src/write.rs` is the mirror of what you are building; read
it once. The byte layouts are in model.md and not repeated here.

## Rules for this phase

- Only the reader. The writer does not change. If a round trip fails, the reader is wrong
  until bytes prove otherwise; then stop and report the bytes.
- No dependencies. Standard library only.
- Nothing panics on any input. Every slice, every length, every allocation is checked first.
  An index out of range in the reader is a bug.
- Comments are short and technical. No comment references a doc.
- Tests: exactly the ones in step 9. No test asserts a byte offset, a key string or a
  constant from the spec.
- If Photoshop's file disagrees with model.md, stop and report the bytes. Do not change
  the spec and do not work around it.
- Every step is a commit. Commit messages say what changed, one line.

## Crate changes

```
src/lib.rs        Error::UnsupportedLayer, pub use read::{read, read_header, Header}
src/read.rs       Cursor, Header, read_header, read
examples/dump.rs  prints a file's tree in dump.jsx's line format, writes layer pixels
tests/unit.rs     the three tests in step 9, added to the phase 1 four
tools/readers/compare-layers.py
tools/reader-gate.ps1
```

## Steps

### 1. Error and Header

In `lib.rs` add `Error::UnsupportedLayer { name: String, reason: &'static str }`, shown as
`unsupported layer '<name>': <reason>`. It is for layers the model cannot hold; every other
refusal stays `Unsupported` or `Malformed`.

In `read.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header { pub format: Format, pub width: u32, pub height: u32,
    pub channel_count: u16, pub depth: u16, pub color_mode: u16 }

pub fn read_header(input: &[u8]) -> Result<Header>;
pub fn read(input: &[u8]) -> Result<Document>;
```

`read_header` parses the 26 bytes and nothing else. It checks the signature and that the
version is 1 or 2, `Malformed("header")` otherwise. It does not refuse depth or mode; that
is `read`'s job, so a caller can tell a 16-bit file from garbage. Re-export all three from
`lib.rs`.

### 2. Cursor

A private `struct Cursor<'a> { data: &'a [u8], pos: usize }` with `u8`, `u16`, `i16`,
`u32`, `i32`, `u64`, `take(n) -> &'a [u8]`, `skip(n)`, `remaining()`, and
`len(format) -> Result<u64>` that reads a u32 for Psd and a u64 for Psb. Every method
returns `Malformed("truncated")` when fewer bytes remain than asked. `take` and `skip`
take a `u64` and convert through `usize::try_from`; failure is `Malformed("truncated")`.
Sub-sections are read by taking their bytes into a child `Cursor` so a bad inner length
cannot read past the section.

### 3. Header, colour mode data, image resources

`read` starts with `read_header`. Refuse: depth other than 8 as `Unsupported("depth")`,
colour mode other than 3 as `Unsupported("colour mode")`, channel count outside 3..=4 as
`Unsupported("channel count")`, a side of 0 or above the format's limit as
`Malformed("header")`.

Colour mode data: u32 length, skip.

Image resources: u32 section length, child cursor, then blocks until it is empty. Each
block: `8BIM` (anything else is `Malformed("image resources")`), u16 id, Pascal name
padded to even, u32 length, data padded to even. Keep 1039 as `icc_profile`. For 1005 read
the horizontal resolution as fixed 16.16 and the unit u16; unit 1 is ppi, unit 2 is pixels
per cm and is multiplied by 2.54; anything else is skipped. Skip every other id by length.

### 4. Layer records

Layer and mask section: length by format, child cursor. Inside: layer info length by
format, child cursor. Either length being 0 means no layers, `Channels::Rgb`, and the
merged image follows. Then i16 layer count; `Channels::Rgba` when negative, `Rgb` when
positive, zero means no layers. An Rgba count with a header channel count under 4 is
`Malformed("layer count")`.

Parse `count.abs()` records into a private `Vec<RawLayer>`:

```rust
struct RawLayer { rect: Rect, channels: Vec<(i16, u64)>, blend: Blend, opacity: u8,
    clipped: bool, visible: bool, mask: Option<Mask /* data empty for now */>,
    name: String, section: u32 /* lsct kind, 0 when absent */ }
```

Per record, per model.md: rect, u16 channel count, pairs of (i16 id, length by format),
`8BIM` else `Malformed("layer record")`, blend key through `Blend::from_key` else
`Unsupported("blend mode")`, opacity, clipping, flags (bit 1 set means hidden), one byte,
u32 extra length, child cursor for the extra data:

- Mask block: u32 length. 0 means none. 20 or more: rect, default, flags (bit 1 disabled,
  bit 2 inverted), then skip the rest of the block by its length. 1 to 19 is
  `Malformed("mask block")`. A mask with a default other than 0 or 255 is
  `Malformed("mask block")`.
- Blending ranges: u32 length, skip.
- Pascal name: u8 length, bytes, then pad so length byte plus bytes is a multiple of 4.
  Decode as ASCII, replacing bytes above 127 with `?`. Used only when `luni` is absent.
- Tagged blocks until the extra cursor is empty: `8BIM` or `8B64` else
  `Malformed("tagged block")`, 4-byte key, length (u64 only for the PSB keys listed in
  model.md under `8B64`, u32 otherwise), data by length. Handle `luni` (u32 char count,
  UTF-16BE, overrides the Pascal name), `lsct` (u32 kind; 0..=3, else
  `Unsupported("section kind")`; for kind 1 or 2 the data continues with `8BIM` and a
  blend key, and that key is the group's blend: Photoshop leaves `norm` in the record.
  Keep it in `RawLayer` as `section_blend: Option<Blend>`, unknown is
  `Unsupported("blend mode")`; Photoshop's block is 16 bytes, ours 12, read by length).
  Every other key is skipped by length, except the refusal list below.

Refused by name with `Error::UnsupportedLayer`, reason in brackets, when the record
carries any of these keys:

- `SoCo GdFl PtFl brit levl curv expA vibA hue2 blnc blwh phfl mixr clrL nvrt post thrs
  selc grdm` [adjustment or fill layer]
- `TySh tySh` [text layer]
- `SoLd SoLE PlLd` [smart object]
- `vmsk vsms vstk vogk` [vector mask or shape]

A record with `lsct` kind 0 is refused the same way with [unknown section kind].
Layer effects (`lfx2`, `lrFX`) are skipped, not refused.

Channel ids: 0, 1, 2, -1, -2 are kept; -3 is kept in the list so its bytes are skipped
later; any other id is `Unsupported("channel id")`.

### 5. Channel data

After the records, in record order, for each channel of each record: take exactly its
stored length into a child cursor. Then u16 compression: 0 raw, 1 RLE, anything else
`Unsupported("compression")`. The plane's rect is the layer rect for ids 0, 1, 2, -1 and
the mask rect for -2. For -3 the bytes are skipped by length without decoding. An empty
rect gives an empty plane and the channel must hold nothing past the compression word.

Raw: the plane is `width * height` bytes, exactly. RLE: one row count per row (u16 Psd,
u32 Psb), then each row through `rle::decode_row` with its own count of bytes; the row's
bytes must be exactly consumed. Short or long is `Malformed("channel data")`. The plane
size comes from the rect, with `Rect::area()` None as `Malformed("rect")`.

Then interleave into `Image.data` as RGBA: planes 0, 1, 2 and -1; a missing -1 gives
alpha 255; a missing colour plane is `Malformed("channel data")`. Plane -2 becomes
`Mask.data`; a -2 channel without a mask block, or a mask block without a -2 channel, is
`Malformed("mask block")`.

After the last record's channels, skip to the end of the layer info cursor; the padding is
whatever is left. Then u32 global layer mask info length, skip. Then the rest of the layer
and mask section is document-level tagged blocks, skipped as a whole.

### 6. Tree

Walk the `RawLayer`s in file order, bottom to top, with a stack of open groups:

- Kind 3 (`</Layer group>`): push a new frame. The record's own fields are discarded.
- Kind 1 or 2: pop a frame, else `Malformed("group nesting")`. Build a `Group` from this
  record: name, visible, opacity, blend (`section_blend`, or the record's key when the
  block has none), expanded is kind 1, mask, children are the frame's nodes. Push it into the parent frame, or the root.
- Otherwise a `Layer`: name, visible, opacity, blend (PassThrough here is
  `Malformed("pass through on a layer")`), clip_to_below, pixels, mask. Push it into the
  current frame or the root.

A frame left open at the end is `Malformed("group nesting")`. More than 64 open frames is
`Malformed("nesting")`.

### 7. Merged image

u16 compression, 0 or 1, else `Unsupported("compression")`. Planes in order R, G, B, then
A when the header says 4 channels. RLE: all row counts for all planes first, then all rows.
The planes are decoded with the same code as step 5. Then interleave to `merged.data` at
rect `0, 0, height, width`.

When `Channels::Rgba`, undo the white blend per read.md: for each pixel with alpha `a`,
`c = ((stored - 255 + a) * 255 + a / 2) / a` in i32 arithmetic, clamped to 0..=255, and 0
when `a` is 0. When `Channels::Rgb`, alpha is 255 and a fourth plane, if present, is
ignored.

A file that ends before the merged image is `Malformed("truncated")`. Bytes after the
merged image are ignored.

### 8. examples/dump.rs

`cargo run --example dump -- <psd>`: reads the file and prints one line per node in the
exact format of `tools/photoshop/dump.jsx`, top of the stack first:

```
<group|layer> '<name>' opacity=<0..255> blend=<NAME> visible=<true|false> bounds=<l>,<t>,<r>,<b>[ mask=<l>,<t>,<r>,<b>]
```

Blend names as Photoshop prints them: the `Blend` variant in upper case with no
separator, so `PASSTHROUGH`, `COLORBURN`, `MULTIPLY`. Groups print no `bounds=` because the file has
none for them. It also writes every pixel layer's RGBA bytes to `<psd>.<n>.rgba`, `n`
counting from 1 in file order over pixel layers only, and prints after the tree one line
per file: `<n> <name> <w>x<h> <path>`. On a read error it prints the error and exits 1.

### 9. Tests, tests/unit.rs

Three tests added to the phase 1 four, no more:

- `read_round_trip`: write the smoke document to a `Vec<u8>`, read it back, assert the
  `Document` equals the one written. Then read `tests/fixtures/softpsd-smoke.psd` and
  assert it equals the same document.
- `read_photoshop_smoke`: read `tests/fixtures/ps27-smoke.psd`. Assert the tree is, bottom
  to top: a layer `Layer 1` with an empty rect and no mask; a group `Group A`, PassThrough,
  expanded, opacity 255, visible, holding one layer `Painted`, Multiply, opacity 153,
  visible, not clipped, rect 8,8,40,40, every pixel `200, 30, 30, 255`, mask rect
  16,16,48,48 all 255 with default 0, not disabled, not inverted. Assert `channels` is
  Rgba, `icc_profile` is 3144 bytes, `resolution_dpi` is 72, and the merged pixel at
  x 20, y 20 is `200, 30, 30, 153`. Build the expected `Document` and compare whole
  values; do not assert field by field.
- `read_refuses`: three asserts in one test. The smoke bytes cut at every length from 0 to
  the end in steps of 7 return `Err`, never panic. The smoke bytes with the depth field
  set to 16 return `Err(Unsupported(_))`. The smoke bytes with the first layer's first
  channel length set to `0xFFFF_FFFF` return `Err(Malformed(_))`. Patch the bytes by
  searching for the field through the walker from `write_smoke_parses`, not by a literal
  offset.

Before trusting `read_round_trip`, swap two blend keys in the reader's lookup once and
confirm it fails, then restore. Before trusting `read_photoshop_smoke`, change the
expected opacity once and confirm it fails.

### 10. tools/readers/compare-layers.py

`python compare-layers.py <psd>`: runs `cargo run --example dump -- <psd>` from the repo
root, parses the `<n> <name> <w>x<h> <path>` lines, loads each `.rgba`, and compares it
with psd-tools' pixels for the layer of the same name: `layer.numpy()` for the RGBA
array, cut to the layer's bbox, or `layer.topil("RGBA")`; whichever gives straight alpha
at the layer's own rect. Prints per layer the count of pixels differing by more than 1 in
any channel and the first differing coordinate. Exit 1 when any count is not zero or a
layer is missing on either side.

### 11. tools/reader-gate.ps1

`reader-gate.ps1 <psd>`: runs `cargo run --example dump -- <psd>` and
`photoshop/run.ps1 photoshop/dump.jsx -Out <psd>`, strips ` bounds=...` from the group
lines of the Photoshop output, and compares the two trees line by line. Prints both under
headers. Then runs `compare-layers.py <psd>`. Fails on the first difference or non-zero
exit. The Photoshop step also exports `<psd>.png`, which this gate ignores.

## Gate

```
tools\reader-gate.ps1 tests\fixtures\ps27-smoke.psd
tools\reader-gate.ps1 tests\fixtures\softpsd-smoke.psd
```

Both pass when the trees match and every layer reports zero differing pixels. `cargo test`
passes with seven tests.

## Handoff

Write `doc/work/phase2/report.md`: what passed, the exact gate outputs, anything in
Photoshop's file that model.md or read.md did not predict, with bytes. Keep it under a page.
