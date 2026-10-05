# Phase 4 gate checks

## Chunk 1: third reader

`photoshopapi-dump.py` reads through `LayeredFile_8bit.read` and prints name, nesting,
blend, byte opacity, visibility, clipping, pixel rect, mask rect/default and disabled state.
It converts center/dimensions to rects and visits bottom first, matching the existing
dumps. UTF-8 output preserves the Unicode name fixture. Installed packages remain under
gitignored `tools/`; no crate dependency or reader/writer change was needed.

The writer gate runs PhotoshopAPI after ag-psd and before Photoshop. `gates.md` now
states the actual split: round trips run through the Cargo fixture loops; library and
Photoshop gates run through `gate-fixtures.ps1`. The tools README and third-party table
include the new dump.

The full fixture gate passes all twelve pairs with three library dumps per writer file.
Field-by-field checks against psd-tools and ag-psd agree on all twelve writer trees:
order, nesting, names, blends, opacity, visibility, clipping, pixel rects and mask
rects/defaults/disabled state. Group pixel bounds are omitted from tree comparisons.
No new third-party disagreement was found; `doc/lab/readers.md` needs no finding added.
Merged and layer pixel comparisons report zero differences throughout.

A temporary PhotoshopAPI exit 7 stops the writer gate before Photoshop and propagates
exit 7. A missing-file invocation exits nonzero. Formatting, seven Rust tests and
Clippy with warnings denied pass. Gate output is preserved in
`target/phase4-chunk1-gate.txt`.
