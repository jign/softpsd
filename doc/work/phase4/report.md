# Phase 4 gate checks

## Chunk 1: third reader

PhotoshopAPI's dump reports nesting, names, blend, opacity, visibility, clipping, rects
and mask rect/default/disabled state; the writer gate runs it after ag-psd. UTF-8 preserves
the name fixture. All twelve trees agree field by field with psd-tools and ag-psd, with
group pixel bounds omitted. No library finding. All fixture/pixel gates and seven tests
pass. Injected exit 7 stops before Photoshop; missing input fails. `gates.md` now assigns
round trips to Cargo and external checks to `gate-fixtures.ps1`.

## Chunk 2: corpus and triage

Shallow sparse clones: ag-psd `387049670cb8` (`test/`), psd-tools `d68bf46c7140`
(`tests/psd_files/`). Idempotent rerun pulls unchanged. Counts: ag-psd 163 PSD / 10 PSB;
psd-tools 278 PSD / 37 PSB; **488 total**, none above 64 MB.

| Reader class | Files |
| --- | ---: |
| Successful | 124 |
| Unsupported | 83 |
| Unsupported layer | 280 |
| Malformed | 1 |
| Panic / skipped size | 0 / 0 |

Comparator: **124 match, 0 differs, 364 refused, 0 psd-tools failed**; no differing lines
to triage. Initially seven malformed files all opened in Photoshop. Separate reading fixes:

- Optional global mask header: `psd-tools/tests/psd_files/1layer.psd`, `2layers.psd`,
  `transparentbg-gimp.psd`, and `ag-psd/test/read/sai/src.psd` now read.
- `ag-psd/test/read/nested/src.psd`: read `lsdk` section dividers.
- `ag-psd/test/read/rle-fail/src.psd`: allow trailing PackBits no-op padding. RGB row
  164 completes 1,600 pixels at byte 31, then `80`; stored count 32.

**Finding:** `psd-tools/tests/psd_files/blend-modes/group-divider-blend-mode.psd`
opens in Photoshop but has only 1,606 bytes for a declared raw 100 × 100 × 4 composite
(40,000 bytes). psd-tools also rejects that image data. We retain `Malformed("truncated")`;
recovering a composite exceeds the allowed parsing fixes. No model/writer change.

The dependency-free corpus feature catches each reader panic and reports status per file.
An injected panic fails its test; a probe above 64 MB is skipped. Both were removed.
Corpus test, seven fixture tests, formatting, Clippy and all twelve gates pass. Details:
`target/corpus.txt`, `corpus-check.txt`, `corpus-triage.txt`; initial Photoshop triage is
preserved in `target/phase4-corpus-triage-initial.txt`.

## Chunk 3: composites and release command

`tools/gate-all.ps1` passes end to end, including a fresh `powershell -NoProfile` run:
setup check, fmt, Clippy, tests, twelve fixture gates, corpus test, then both engines
on every writer fixture. Discovery finds the supplied E: installs automatically.
Krita **5.3.4** and GIMP **3.2.6** both run; neither is skipped. GIMP uses a fresh hidden
console process, Script-Fu v3, and flattening over white. Separate logs avoid legacy
PowerShell treating native warning output as a failure. Krita exports in a hidden process.

Values below are **differing pixels / maximum 8-bit channel difference**, tolerance 1:

| Fixture | Krita | GIMP |
| --- | ---: | ---: |
| smoke | 3520 / 255 | 4096 / 255 |
| flat | 0 / 0 | 0 / 0 |
| alpha | 960 / 255 | 960 / 255 |
| blends | 997 / 255 | 997 / 255 |
| hidden | 768 / 255 | 768 / 255 |
| clip | 768 / 255 | 768 / 255 |
| nested | 960 / 255 | 960 / 255 |
| mask-white | 256 / 255 | 0 / 0 |
| empty | 960 / 255 | 960 / 255 |
| name | 960 / 255 | 960 / 255 |
| icc | 960 / 255 | 960 / 255 |
| psb | 0 / 0 | 0 / 0 |

Krita's clip paints blue outside clipping bounds; mask-white leaves a transparent hole.
These visible disagreements are recorded in `doc/lab/readers.md`. Its other differences
are only RGB at alpha zero; visible pixels agree. GIMP's transparent-fixture differences
come from the requested flattening removing alpha, including smoke's partial alpha.
Pixel mismatches stay informational; the comparison rule is unchanged.

Missing-engine simulation skips explicitly; missing setup stops with the install command.
Injected exit 7 stops at the named step. A changed pixel returns status 1; missing PNG
returns 2 and fails. Export failures/missing output/timeouts also fail. Logs and PNGs:
`target/phase4-gate-all-final.txt`, `target/composites/`. Phase 4's gate is complete;
the truncated-composite finding remains for review.
