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

**Stopped: blends.** The writer gate's merged comparison passes with zero differences.
Both files decode to their complete expected documents, and the Photoshop-made file's
reader gate passes with zero differences for all 27 layers. Our file's reader tree gate
fails: Photoshop reports full-canvas bounds for ColorBurn, ColorDodge and Difference,
even before PNG export. All three descriptor keys (`bounds`, `boundsNoMask`,
`boundsNoEffects`) report `0,0,32,32` for our file, but the correct 1-pixel rects for its
own file. This contradicts the lab claim that `boundsNoMask` is the stored pixel rect.

Raw layer rect bytes (top, left, bottom, right), identical in both files:

| Layer | Key | Rect bytes |
| --- | --- | --- |
| ColorBurn | `idiv` | `00000000 00000004 00000001 00000005` |
| ColorDodge | `div ` | `00000000 00000009 00000001 0000000A` |
| Difference | `diff` | `00000000 00000013 00000001 00000014` |

First failing tree lines, softpsd then Photoshop:

```text
layer 'Difference' opacity=255 blend=DIFFERENCE visible=true bounds=19,0,20,1
layer 'Difference' opacity=255 blend=DIFFERENCE visible=true bounds=0,0,32,32
```

Photoshop's records also carry `clbl=01 00 00 00`, `infx=00 00 00 00` and
`knko=00 00 00 00`; our file omits them. No spec or writer change, bounds workaround,
or fixture commit was made. The stopped builder, script and both files are preserved
locally under `target/phase3-pending/blends/`, outside the accepted catalog, for review.
