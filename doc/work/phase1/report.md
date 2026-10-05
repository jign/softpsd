# Phase 1 writer checks

Steps 9–12 implemented. All four Rust tests, formatting, compilation and Clippy pass.
The smoke parser fails with a positive layer count, then passes after restoration.
The comparator rejects a two-level pixel change and accepts a one-level change.
The gate stops at the first failing reader. The full smoke run exits 0; output below.

The fixture gate remains pending: Photoshop's `layer.bounds` reports 16,16,40,40,
where the stored layer rect is 8,8,40,40. Its rect bytes (top, left, bottom, right) are
`00 00 00 08 00 00 00 08 00 00 00 28 00 00 00 28`. The mask rect is 16,16,48,48.
No writer or spec workaround was made, and no accepted fixture was copied.

```text
=== psd-tools ===
psd-tools 64x64 mode=RGB depth=8 channels=4
icc: 0 bytes
  group 'Group A' opacity=255 blend=NORMAL bbox=(8, 8, 40, 40)
    pixel 'Painted' opacity=153 blend=MULTIPLY bbox=(8, 8, 40, 40) mask bbox=(16, 16, 48, 48) bg=0 disabled=False
layer count field: -3
  '</Layer group>' channels=[(-1, 2), (0, 2), (1, 2), (2, 2)] blocks=['UNICODE_LAYER_NAME', 'LAYER_ID', 'SECTION_DIVIDER_SETTING']
  'Painted' channels=[(-1, 130), (0, 130), (1, 130), (2, 130), (-2, 130)] blocks=['UNICODE_LAYER_NAME', 'LAYER_ID']
  'Group A' channels=[(-1, 2), (0, 2), (1, 2), (2, 2)] blocks=['UNICODE_LAYER_NAME', 'LAYER_ID', 'SECTION_DIVIDER_SETTING']
merged compression: RLE
=== ag-psd ===
ag-psd 64x64 channels=4 bits=8 mode=3 icc=false
  'Group A' opacity=1 blend=normal bbox=0,0,0,0
    'Painted' opacity=0.6 blend=multiply bbox=8,8,40,40 mask 16,16,48,48 default=0 disabled=false
resources: versionInfo
=== Photoshop ===
group 'Group A' opacity=255 blend=NORMAL visible=true bounds=16,16,40,40
  layer 'Painted' opacity=153 blend=MULTIPLY visible=true bounds=16,16,40,40 mask=16,16,48,48
=== merged comparison ===
differing pixels: 0
first differing coordinate: None
```
