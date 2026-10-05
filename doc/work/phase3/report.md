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

## Chunk 4: mask-white finding

**Stopped for review; no fixture committed.** The three-layer writer gate passes with zero
merged differences, but Disabled covers the whole canvas and conceals Inverted's behavior.
Separate native PNG exports hide the other two layers before testing each mask:

| Layer | Shown pixels | Pixel (0,0) alpha | Pixel (16,16) alpha |
| --- | ---: | ---: | ---: |
| White | 768 | 255 | 0 |
| Disabled | 1024 | 255 | 255 |
| Inverted | 1024 | 255 | 255 |

Inverted should show only 256 pixels inside the hole under the model. Photoshop instead
shows it at full. The plan's finding is **strike `inverted` from the model**. No model or
writer change was made. Our mask blocks (excluding length/padding), top/left/bottom/right,
default and flags, are:

```text
White:    00000008 00000008 00000018 00000018 FF 00
Disabled: 00000008 00000008 00000018 00000018 00 02
Inverted: 00000008 00000008 00000018 00000018 00 04
```

Photoshop's Hide Selection White has identical bytes. Its all-black Disabled mask is
trimmed to `00000000 00000000 00000000 00000000 00 02`; its opaque composite is RGB.
The tree gate also exposes an oracle limitation: loading the mask as selection reports
nonzero selection bounds, not its stored rect. White reports `mask=0,0,32,32` instead of
`8,8,24,24`; empty Disabled reports `mask=?`. Both reader gates fail those comparisons.
No bound workaround or expected-mask adjustment was made.

Builder, script, both PSDs, individual PNGs, probe and gate logs are preserved in
`target/phase3-pending/mask-white/`. All eleven accepted pairs pass the full fixture gate;
seven tests, formatting and Clippy pass. Phase 3 awaits the mask finding decision.
