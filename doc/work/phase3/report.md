# Phase 3 fixture checks

## Chunk 1: smoke harness

The shared fixture catalog, builder example and seven tests pass. The example lists the
catalog for the gate runner; the Photoshop expectation keeps the empty bottom layer,
PassThrough group, file's ICC resource and 72 ppi resolution. The smoke script moved to
`tools/photoshop/fixtures/` and its current documentation links were updated.

`tools/gate-fixtures.ps1 smoke`, the no-name invocation and `-Make smoke` all pass:
the writer gate reports zero differing merged pixels, and both reader gates match the
Photoshop trees with zero differing pixels in every layer. The existing Photoshop fixture
was restored byte for byte after verifying `-Make`; no new fixture was added.

The fixture round trip test rejects a changed writer depth byte, then passes after
restoration. An injected writer-gate exit 7 stops the harness before either reader gate
or the next fixture and reports `Fixture 'smoke' failed at 'writer gate': exit 7`.
Unknown names are rejected before building. Formatting, compilation and Clippy pass.
No spec disagreement or writer change was needed.

## Chunk 2: tree fixtures

Accepted and committed: alpha, hidden, clip, nested, empty and name, each with its builder,
script and two fixtures. The six-name fixture gate passes: every merged and layer pixel
comparison reports zero differences. The smoke gate also passes after the dump changes.
Seven tests, formatting, compilation and Clippy pass. Each new accepted fixture rejects
a changed layer opacity byte; fixture assertions now print only the fixture name.
The name is 200 Unicode code points (220 UTF-16 units); the JSX uses escapes.

Two gate-tool bugs were fixed without changing the reader or writer. Selecting a hidden
layer through `doc.activeLayer` makes it visible: the old dump exported green at (20,20)
as `00 FF 00 FF` instead of transparent `FF FF FF 00`. It also reported the hidden
group's child as invisible through the DOM, although its stored flags are `08` (visible),
versus `0A` for Hidden and `1A` for Hidden group. The dump now queries descriptors by ID,
uses each descriptor's own visibility, and exports before selecting any mask channel.
Python gate tools emit UTF-8 instead of failing on the non-ASCII name through cp1252.
The Rust dump prints Photoshop's `COLORBLEND` enum name.

Photoshop accepts the nested script's `layerSectionExpanded` request but does not expose
that property or save a closed group. The permitted fallback is used: Photoshop's three
groups are open; our Middle remains closed. Middle's Photoshop `lsct` data is
`00 00 00 01 38 42 49 4D 70 61 73 73 00 00 00 00`, versus our kind 2. Gate excerpt:

```text
saved nested: layerSectionExpanded unavailable; groups left open
fixture nested: passed
```

The blends finding was resolved by the reviewer's spec amendments (9dd274d and
0daeee1, already local). Every pixel record now carries `clbl=01 00 00 00`,
`infx=00 00 00 00`, `knko=00 00 00 00` after `lyid`; groups and ends do not.
Previously Photoshop reported canvas bounds for ColorBurn, ColorDodge and Difference
although both files stored their correct one-pixel rects. With these defaults, both
blends gates pass with zero pixel differences and the seventh pair is committed.

All three gate scripts set console output to UTF-8. The name gate also passes from a
fresh `powershell -NoProfile` session. No document dumps appear on fixture mismatch.

## Chunk 3: writer features

Validation and writing now support RGB, ICC resource 1039, resolution resource 1005,
and PSB. RGB writes three merged planes, no white blend, and a positive layer count;
pixel layers retain alpha. Resolution uses rounded unsigned 16.16 values and rejects
nonfinite, nonpositive or unrepresentable values before writing. PSB uses version 2,
u64 section/info/channel lengths and u32 RLE row counts, including the merged table.
A scratch probe round-trips RGB/RGBA in both formats with an odd-length ICC payload,
96 ppi, groups and masks; invalid resolutions produce no output.

Accepted: **icc** and **psb**, each committed with its builder, script and two files.
Photoshop saves the opaque Wide composite as RGB while our PSB exercises RGBA; the
Photoshop expectation reflects its count sign. Both files' merged and layer pixels match.
PSB fixture paths use `.psb`, matching Photoshop's save extension.

The installed psd-tools pixel guard incorrectly applies the 30,000-pixel PSD limit to
version 2 files. The comparison adapter temporarily uses the 300,000-pixel PSB limit
for version 2 only, restores the previous limit afterwards, and keeps allocation guards.
No decoder or fixture data was modified to pass a comparison.

Save for Web produced the full 30,001 × 8 PNG but displayed size warnings. The dump
now uses native `PNGSaveOptions` saving; merged comparison disables ICC conversion to
compare stored channel values with PNG pixels. Smoke, ICC and PSB pass again. ICC had
also passed before this switch; its colour profile required no workaround.

**Stopped: flat.** Our RGB writer fixture passes its writer and reader gates, including
zero merged/layer pixel differences. Photoshop's WHITE document, painted directly on
Background and saved with layers enabled, has zero layer-info length. Our reader correctly
returns no layers under the existing zero-length rule, whereas Photoshop synthesizes a
Background layer when opening the file. This conflicts with the planned one-layer
Photoshop expectation and reader tree gate. No reader or model workaround was made.

Layer/mask section bytes (big endian):

| File | Offset | Section length | Layer-info length | Next bytes |
| --- | ---: | --- | --- | --- |
| Photoshop flat | 20896 | `00 00 03 3C` | `00 00 00 00` | `00 00 00 00 38 42 49 4D` |
| Our flat | 120 | `00 00 03 6C` | `00 00 03 64` | `00 01` (one layer) |

```text
Fixture 'flat' failed at 'reader gate (Photoshop)':
Tree differs at line 1: softpsd=<missing>;
Photoshop=layer 'Background' opacity=255 blend=NORMAL visible=true bounds=0,0,32,32
```

The stopped flat builder, script and both files are preserved under
`target/phase3-pending/flat/`, outside the accepted catalog, for review.
The accepted catalog now has ten pairs. Its full gate passes, all comparisons report
zero differences, and the seven Rust tests, formatting and Clippy pass. ICC and PSB
each reject a mutated stored opacity byte with a fixture-specific failure.
