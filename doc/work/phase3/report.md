# Phase 3 fixture checks

## Chunk 1: smoke harness

The fixture catalog, builder example, Photoshop scripts and seven tests share one harness.
Smoke passes writer and both reader gates, including no-name and `-Make` invocations.
All merged/layer comparisons report zero differences. A mutated depth byte is rejected;
an injected writer exit 7 stops before subsequent gates and names the fixture/stage.
Unknown names fail before building. Formatting, compilation and Clippy pass.

## Chunk 2: tree fixtures

Seven pairs accepted: alpha, hidden, clip, nested, empty, name and blends. All gates pass
with zero pixel differences; opacity mutations fail with the fixture name and no document
dump. Name has 200 code points / 220 UTF-16 units, escaped in JSX. All three gate scripts
set console output to UTF-8; name passes from fresh `powershell -NoProfile`.

The dump queries descriptors by ID for each layer's own visibility and exports before
mask selection. Selecting a hidden layer had made it visible; DOM visibility had also
included the parent's state. Python tools emit UTF-8; Rust prints `COLORBLEND`.

Photoshop does not expose/save the requested closed Middle group. The permitted fallback
leaves its groups open; ours remains closed. Its Middle `lsct` bytes are
`00000001 3842494D 70617373 00000000`, versus our kind 2.

Blends originally reported canvas bounds for ColorBurn, ColorDodge and Difference despite
correct one-pixel stored rects. Reviewer commits 9dd274d / 0daeee1 resolved the spec gap:
pixel records get `clbl=01 00 00 00`, `infx=00 00 00 00`, `knko=00 00 00 00` after
`lyid`; groups/ends do not. Regated blends is committed.

## Chunk 3: writer features

Validation/writing support RGB, ICC 1039, resolution 1005 and PSB. RGB merged data has
three planes, no white blend, and positive layer count; layers retain alpha. Resolution
is rounded unsigned 16.16; nonfinite, nonpositive/unrepresentable values fail before
output. PSB writes version 2, u64 section/info/channel lengths, u32 RLE row counts.
A scratch probe covers both formats/channels, odd ICC bytes, 96 ppi, groups and masks.

ICC and PSB pass all gates. Photoshop saves opaque Wide's composite as RGB; its expected
model reflects that. PSB uses `.psb` paths. The psd-tools adapter temporarily uses the
300,000-pixel side limit for version 2, retains allocation guards and restores the limit.
Save for Web emitted size warnings despite a correctly sized PNG; native PNG saving
replaces it. Comparisons use stored pixels without ICC conversion. Smoke regates cleanly.

Flat originally stopped: Photoshop saved layer-info length zero yet displayed Background.
Its section length/info bytes were `0000033C 00000000`; ours were `0000036C 00000364`
with count `0001`. Reviewer commit c50292e (already local) specifies synthesizing Background.
The reader now builds it from the opaque merged image, Normal/255/visible, no mask/clip.
Both zero section and zero info lengths are verified. The gate compares synthesized pixels
against psd-tools' merged image. Flat passes with profile and 72 ppi added to its Photoshop
expectation and is committed as pair eleven. All seven tests and Clippy pass; mutated
flat/ICC/PSB opacities are rejected.

## Chunk 4: mask-white

The initial three-layer probe showed White at 768 visible pixels, Disabled at 1024,
and Inverted at 1024 rather than the modeled 256. Other layers were hidden individually
for these exports because Disabled's full-canvas red would conceal the inversion result.
Stored mask bytes (top/left/bottom/right, default, flags) were:

```text
White:    00000008 00000008 00000018 00000018 FF 00
Disabled: 00000008 00000008 00000018 00000018 00 02
Inverted: 00000008 00000008 00000018 00000018 00 04
```

Reviewer commit cd40217 (already local) ruled the findings. `inverted` is removed from
Mask, reader, writer and builders; mask flags now carry only disabled. The accepted fixture
has White and Disabled. Photoshop trims its all-default Disabled mask to
`00000000 00000000 00000000 00000000 00 02`, with no data; its expected model reflects
that and the opaque RGB composite, profile, 72 ppi and empty bottom Layer 1.

Loading a mask as selection measures its selected area rather than stored rect. Both dumps
now print `mask=on/off`: Photoshop reads `userMaskEnabled` from the layer descriptor;
Rust reads `disabled`. The dump no longer selects mask channels. Stored mask rects remain
checked through library dumps and document round trips.

```text
layer 'Disabled' opacity=255 blend=NORMAL visible=true bounds=0,0,32,32 mask=off
layer 'White' opacity=255 blend=NORMAL visible=true bounds=0,0,32,32 mask=on
fixture mask-white: passed
```

Mask-white passes both gates with zero merged/layer pixel differences and is committed
with its script and builder as pair twelve. A mutated stored opacity is rejected with
the fixture name. The original probe and findings remain under
`target/phase3-pending/mask-white/` for reference.

The final `tools/gate-fixtures.ps1` invocation passes all twelve pairs: smoke, alpha,
hidden, clip, nested, empty, name, blends, icc, psb, flat and mask-white. Every merged/layer
comparison reports zero differences. `cargo test` is seven green; formatting and Clippy
also pass. Phase 3 is complete; no unresolved findings remain.
