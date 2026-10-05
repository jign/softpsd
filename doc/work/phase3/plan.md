# Phase 3: fixtures

Goal: every row of `doc/spec/gates.md` has its two fixtures in `tests/fixtures/`, and each
pair passes the writer gate and the reader gate.

This phase is in chunks, not steps. A chunk is built, gated and reported as one unit; the
review happens once per chunk. Commits inside a chunk are still one per fixture or one per
code change, one line each.

Read first: `doc/spec/gates.md`, `doc/spec/write.md`, `doc/lab/photoshop-scripting.md`.
Phase 1 and 2 reports show what a passing gate looks like.

## Rules for this phase

- Every fixture is small: 32 × 32 unless the row needs otherwise, constant colours, RLE.
- The merged image in every builder must be trivially right without compositing: layers do
  not overlap, or the only layer under a blend is nothing. Blending maths is phase 5.
- No comment references a doc. No test asserts a byte offset, a key or a constant.
- If Photoshop disagrees with a spec, stop that fixture, report the bytes in the chunk
  report, and continue with the rest of the chunk. Do not change a spec and do not work
  around it. The reviewer decides.
- `tools/photoshop/fixtures/<name>.jsx` makes `ps27-<name>.psd`. Non-ASCII text in a
  `.jsx` is written with `\uXXXX` escapes; ExtendScript does not read UTF-8 source reliably.
- A fixture is committed only after both gates pass on it, in the same commit as its
  script and builder.

## Chunk 1: harness, on the smoke fixture only

No new fixture. The goal is that adding a fixture afterwards is one builder, one script, one
gate run.

- `tests/common/fixtures.rs`: `pub fn names() -> &'static [&'static str]` and
  `pub fn fixture(name: &str) -> Fixture` where
  `pub struct Fixture { pub ours: Document, pub photoshop: Document }`. `ours` is what we
  write. `photoshop` is what `read(ps27-<name>.psd)` must equal: the same document with
  Photoshop's differences applied (its default empty `Layer 1` at the root bottom, groups
  PassThrough unless the script sets a blend, `icc_profile` taken from the ps27 file's 1039
  resource, `resolution_dpi` 72). Move the walker from `tests/unit.rs` into this module so
  the profile can be pulled from the file. One function per fixture, nothing shared beyond
  small helpers for a filled rect and a solid image.
- `examples/fixture.rs <name> <out>`: builds `fixture(name).ours` and writes it with
  `format_for`. It includes the module with `#[path = "../tests/common/fixtures.rs"]`.
  Delete `examples/smoke.rs`.
- Tests: replace `read_round_trip` and `read_photoshop_smoke` with two tests that loop over
  `names()`: `fixtures_round_trip` (write `ours`, read back, equal; and read
  `tests/fixtures/softpsd-<name>.psd`, equal) and `fixtures_read_photoshop` (read
  `tests/fixtures/ps27-<name>.psd`, equal to `photoshop`). On a mismatch the assert
  message names the fixture. The test count stays at seven.
- `tools/photoshop/fixtures/`: move `smoke.jsx` there. Update the paths in
  `doc/spec/gates.md`, `tools/README.md` and the two lab files.
- `tools/gate-fixtures.ps1 [names...]`: for each name, or all of `names()` when none
  given: `cargo run --example fixture -- <name> target/softpsd-<name>.psd`, then
  `writer-gate.ps1` on it, then `reader-gate.ps1` on it, then `reader-gate.ps1` on
  `tests/fixtures/ps27-<name>.psd` when that file exists. Stops at the first failure and
  says which fixture and which gate. A `-Make` switch first runs
  `photoshop/run.ps1 photoshop/fixtures/<name>.jsx -Out tests/fixtures/ps27-<name>.psd`.
- `tools/writer-gate.ps1` and `reader-gate.ps1` are unchanged except that they must accept
  a `.psb` path.

Gate: `tools\gate-fixtures.ps1 smoke` passes and `cargo test` is seven green.

## Chunk 2: tree fixtures, no writer change expected

Finding from the first pass, now in write.md and model.md: pixel layers get `clbl`, `infx`
and `knko` with Photoshop's defaults, or Photoshop reports canvas bounds for three blend
modes. The writer change is authorised; blends is gated again after it.

Seven fixtures. Any writer change needed here is a finding; report it.

| Name | `ours` | Photoshop script |
| --- | --- | --- |
| alpha | one layer `Dot`, rect 4,4,12,12, `0,120,255,255`, Normal, 255 | transparent document, fill the selection |
| hidden | `Shown` rect 0,0,16,16 red; `Hidden` rect 16,16,32,32 green, `visible: false`; group `Hidden group`, `visible: false`, holding `Inside` rect 0,16,16,32 blue. Merged shows `Shown` only | `layer.visible = false` on the layer and the set |
| clip | `Base` rect 8,8,24,24 red; above it `Clipped` rect 0,0,32,32 blue, `clip_to_below: true`. Merged is blue inside 8,8,24,24 | `layer.grouped = true` on the upper layer |
| nested | `Outer` open > `Middle` closed > `Inner` open > `Deep` rect 4,4,12,12 red | nested `layerSets.add()`. Photoshop's DOM has no expanded flag; try the `layerSectionExpanded` key through `setd`, else leave all open and say so |
| empty | `Empty` rect 0,0,0,0 no data, below `Dot` rect 4,4,12,12 red | `artLayers.add()` with nothing painted, plus the painted layer |
| name | one layer named 200 characters mixing Japanese, Cyrillic and one emoji, rect 4,4,12,12 red | the same name through `\u` escapes |
| blends | 27 layers, one per `Blend` except PassThrough, named after the variant, each a 1 × 1 pixel at `i,0,i+1,1` for `i` in 0..27, colour `255,0,0,255`. Merged is those 27 pixels. Over transparency every mode yields the source | `layer.blendMode = BlendMode.<NAME>` per layer |

`examples/dump.rs` prints `COLORBLEND` for `Blend::Color`, because that is Photoshop's
enum name. Every other variant upper-cased already matches.

Gate: `tools\gate-fixtures.ps1 alpha hidden clip nested empty name blends` passes, and
`cargo test` is green with the seven fixtures in the loops.

## Chunk 3: writer features

Lift the phase 1 refusals in `validate.rs` and `write.rs`, one commit each, then three
fixtures.

- Rgb: header channel count 3, positive layer count, merged written as three planes with no
  white blend. Layers still carry their -1 channel; an Rgb document only means the merged
  image has no transparency.
- Resolution: resource 1005, model.md has the layout. Written when `resolution_dpi` is set.
- ICC: resource 1039, the bytes as given.
- PSB: version 2, 8-byte lengths where model.md says so, u32 row counts in layer channels
  and the merged image. `format_for` already picks it.

| Name | `ours` | Photoshop script |
| --- | --- | --- |
| flat | `Channels::Rgb`, one layer `Background` rect 0,0,32,32 white with a red rect 8,8,24,24 painted in, alpha 255 everywhere; merged the same | `documents.add(..., DocumentFill.WHITE)`, paint the rect on the Background layer. Photoshop's Background layer has no -1 channel |
| icc | the alpha fixture plus `icc_profile: Some(bytes of Adobe RGB (1998))`, pulled from `ps27-icc.psd`'s 1039 by the walker | `doc.convertProfile("Adobe RGB (1998)", Intent.RELATIVECOLORIMETRIC, false, false)` before saving |
| psb | 30,001 × 8, Rgba, one layer `Wide` rect 0,0,8,30001 red, merged the same | `documents.add(30001, 8, ...)`, save through Action Manager `save` with `As` = `largeDocumentFormat`; `PhotoshopSaveOptions` refuses the size |

Known risk, icc: Save For Web may convert the PNG to sRGB, which would fail
`compare-merged.py` for a non-sRGB file. If it does, switch `dump.jsx` to
`doc.saveAs(file, new PNGSaveOptions(), true)`, which writes the pixels as stored, and
rerun the smoke writer gate to show nothing else changed. Record which path was taken.

Gate: `tools\gate-fixtures.ps1 flat icc psb` passes; `cargo test` green with ten fixtures.

## Chunk 4: mask-white, and a question for Photoshop

One fixture with three layers, each rect 0,0,32,32 red, each with a mask rect 8,8,24,24 all
zero:

- `White`: default 255, so the layer shows everywhere except the hole.
- `Disabled`: default 0, `disabled: true`, so the whole layer shows.
- `Inverted`: default 0, `inverted: true`. The model says the hole is inverted and the layer
  shows only inside 8,8,24,24.

Photoshop script: `White` through Hide Selection (`Mk` with `Usng` = `HdSl`), `Disabled`
through `setd` of `userMaskEnabled` false on the layer. Photoshop cannot make an inverted
mask flag from the UI, so `ps27-mask-white.psd` has the first two layers only and
`photoshop` in the builder reflects that.

The question: does Photoshop honour mask flag bit 2 when it opens our file? The merged
comparison answers it. If the PNG shows the `Inverted` layer at full, Photoshop ignores the
flag, and the finding is "strike `inverted` from the model". Report, do not change the model.

Gate: `tools\gate-fixtures.ps1 mask-white`, with the inverted result stated either way.

## Phase gate

```
tools\gate-fixtures.ps1
cargo test
```

All twelve pairs pass both gates; seven tests green.

## Handoff

`doc/work/phase3/report.md`, one section per chunk, appended as each chunk closes: what
passed, the gate output for any fixture that needed a decision, findings with bytes. Under
two pages at the end.
