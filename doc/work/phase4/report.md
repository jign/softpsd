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

## Chunk 2: corpus and triage

The fetcher shallow/sparse clones ag-psd `test/` and psd-tools `tests/psd_files/` into
ignored `corpus/`; rerunning pulls successfully with unchanged counts. Revisions:
ag-psd `387049670cb8`, psd-tools `d68bf46c7140`. Counts: 163 PSD / 10 PSB from ag-psd,
278 PSD / 37 PSB from psd-tools, **488 files** total. No file exceeds 64 MB.

The dependency-free `corpus` feature adds one test: catch each reader panic, classify
results in `target/corpus.txt`, require files and no panics. The Python comparator builds
`examples/dump.rs` once and records normalized trees / first differences. Photoshop
triage opens malformed files, counts root layers, closes without saving and records results.

| Final class | Files |
| --- | ---: |
| Read successfully | 124 |
| Unsupported | 83 |
| Unsupported layer | 280 |
| Malformed | 1 |
| Panic / skipped size | 0 / 0 |

The comparator completes: **124 match, 0 differs, 364 refused, 0 psd-tools failed**.
There are no differing lines to inspect. The initial pass had 118 reads and seven malformed
files; Photoshop opened all seven. Three reading-rule fixes, committed separately:

- Omitted global mask header: `psd-tools/tests/psd_files/1layer.psd`, `2layers.psd`,
  `transparentbg-gimp.psd`, and `ag-psd/test/read/sai/src.psd` now read.
- `ag-psd/test/read/nested/src.psd`: treat `lsdk` as a section divider like `lsct`.
- `ag-psd/test/read/rle-fail/src.psd`: allow trailing PackBits no-op padding. RGB row
  164 produces all 1,600 pixels at byte 31, then ends with `80`; row count is 32.
  Ordinary encoded data after the completed row remains an error.

**Finding:** `psd-tools/tests/psd_files/blend-modes/group-divider-blend-mode.psd`
opens in Photoshop but declares raw merged data for 100 × 100 × 4 (40,000 bytes), with
only 1,606 bytes present. psd-tools' `topil()` also fails with decompressed length
mismatch. After accepting its omitted global mask header, we still return
`Malformed("truncated")` for the composite. Recovering pixels from layers would exceed
the permitted reading-rule changes; model/writer are unchanged. Final Photoshop triage
again confirms it opens. No new disagreement with psd-tools needs a lab entry.

A temporary injected reader panic is caught, listed and fails the corpus test; a
probe above 64 MB is skipped by size. Both probes were removed before the final run.
`cargo test --features corpus` passes the corpus test and seven fixture tests;
formatting and Clippy with warnings denied pass. All twelve fixture pairs pass their
full gates after the fixes. Detailed outputs: `target/corpus.txt`, `corpus-check.txt`,
`corpus-triage.txt`; initial triage is preserved in `target/phase4-corpus-triage-initial.txt`.

