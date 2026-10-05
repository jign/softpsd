# Gates

Two directions, four readers, one oracle. Photoshop decides every disagreement.

## Fixtures

One per claim. Each exists in two forms: `ps27-<name>.psd` made by Photoshop through
`tools/photoshop/<name>.jsx`, and `softpsd-<name>.psd` written by the crate from the same
model. Both live in `tests/fixtures/` and are small.

| Name | Claim |
| --- | --- |
| smoke | one group, one Multiply layer at 60%, a raster mask with black default |
| flat | one layer, Rgb, no transparency, positive layer count |
| alpha | Rgba document, negative layer count, transparent merged image |
| blends | one layer per blend mode, all 27 |
| hidden | a hidden layer and a hidden group |
| clip | a layer clipped to the one below |
| nested | groups three deep, open and closed |
| mask-white | a mask with white default and an inverted one and a disabled one |
| empty | a layer with no pixels, rect 0,0,0,0 |
| name | a 200-character name with non-Latin characters |
| icc | an embedded profile other than sRGB |
| psb | 30,001 px wide, Psb |

## Writer gate

For every fixture, `softpsd-<name>.psd` must:

1. Round-trip through `softpsd::read` to a model equal to the input.
2. Parse in psd-tools, ag-psd and PhotoshopAPI with the same tree: names, order, nesting,
   blend, opacity, visibility, clipping, rects, mask rects and defaults.
3. Open in Photoshop with the same tree, reported by `tools/photoshop/dump.jsx`, and
   Photoshop's own export of it to PNG must equal our merged image, exact.

1 and 2 run on every `cargo test`; 2 skips when `tools/setup.ps1` has not run. 3 runs by
hand before a release through `tools/gate-photoshop.ps1`, which needs Photoshop open.

## Reader gate

For every fixture, `softpsd::read(ps27-<name>.psd)` must give the tree Photoshop reports for
that file, and its layer pixels must equal psd-tools' per-layer pixels.

Then the corpus: `tools/fetch-corpus.ps1` pulls the ag-psd and psd-tools test files into
`corpus/` (gitignored). Every file in the corpus must either read with the same tree as
psd-tools, or refuse with `Error::Unsupported`. A panic or a `Malformed` on a file Photoshop
wrote is a bug. This runs by hand, `cargo test --features corpus`.

## Third-party checks

| Reader | How | Gate |
| --- | --- | --- |
| Photoshop 27 | `tools/photoshop/run.ps1` | writer 3, reader tree |
| psd-tools | `tools/.venv` | writer 2, reader pixels, corpus |
| ag-psd | `tools/readers/node_modules` | writer 2 |
| PhotoshopAPI | `tools/.venv` | writer 2 |
| GIMP, Krita | headless export to PNG | writer composite, when installed; skipped otherwise |

## Testing rules

- One test per row above, not one per branch. No test asserts a byte offset or a constant
  from the spec.
- A fixture is the oracle. A test that passes on first run is mutated once before it is
  believed: flip a byte in the writer, see it fail.
- Benchmarks are separate (`cargo bench`): write and read of an 8192² document with 20
  layers. They report time per file and are not gates.
