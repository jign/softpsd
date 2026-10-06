# Phase 5 gate checks

Built in Soft Edge as WU156 (`dev/0.4.0`), against softpsd `403781e` pinned as a git
`rev`. File > Export > Photoshop writes the layered file; folders become groups, stroke
and raster layers become layers with visibility, opacity and masks.

## Gate

`ps_export_heavy_doc_masked_withfolder_2.psd`: 8000 x 8000, 26 layers, a folder, a masked
raster layer, a hidden layer.

- `dump.jsx` tree matches Soft Edge's layer panel: names, nesting, visibility, opacity,
  mask on.
- Photoshop composite against the stored merged image: max delta 1 on 2,529 pixels,
  none over 1. Normal blending in 8-bit sRGB agrees with Photoshop.
- Merged image equals Soft Edge's own 8K PNG export exactly.
- Three exports of the same document are byte-identical.
- Whole `Document` in memory: the export adds 63 MB to Soft Edge's wasm heap at 8K.
  No streaming writer needed.

## API review

No library finding. Straight-alpha RGBA and `Rect` take Soft Edge's tiles directly;
`PartialEq` on `Document` makes the round-trip test one assertion. A mask default other
than 0 or 255 is a PSD rule, so Soft Edge grows the mask rect to the page instead.

## Deviations from the plan row

- Automasks setting: dropped from Soft Edge's export by its PM.
- PSB switch: wired through `format_for`, not exercised from Soft Edge; 8K is below
  30,000 px. Covered by the `psb` fixture.
