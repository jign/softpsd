# Phase 2 reader checks

All seven tests, formatting, compilation and Clippy pass. The reader round trip test
fails when Normal and Multiply are swapped in the lookup; the Photoshop test fails when
its expected group opacity is changed. Both pass after restoration. The pixel comparator
accepts a one-level change, rejects a two-level change at (8, 8), and rejects missing or
extra layers. Both live Photoshop gates pass. No new discrepancy with model.md or read.md
was found; the recorded 16-byte group divider and white-blended merged RGB are handled.

`tools/reader-gate.ps1 tests/fixtures/ps27-smoke.psd`:

```text
=== softpsd ===
group 'Group A' opacity=255 blend=PASSTHROUGH visible=true
  layer 'Painted' opacity=153 blend=MULTIPLY visible=true bounds=8,8,40,40 mask=16,16,48,48
layer 'Layer 1' opacity=255 blend=NORMAL visible=true bounds=0,0,0,0
=== Photoshop ===
group 'Group A' opacity=255 blend=PASSTHROUGH visible=true
  layer 'Painted' opacity=153 blend=MULTIPLY visible=true bounds=8,8,40,40 mask=16,16,48,48
layer 'Layer 1' opacity=255 blend=NORMAL visible=true bounds=0,0,0,0
=== layer comparison ===
layer 'Layer 1' differing pixels: 0; first differing coordinate: None
layer 'Painted' differing pixels: 0; first differing coordinate: None
```

`tools/reader-gate.ps1 tests/fixtures/softpsd-smoke.psd`:

```text
=== softpsd ===
group 'Group A' opacity=255 blend=NORMAL visible=true
  layer 'Painted' opacity=153 blend=MULTIPLY visible=true bounds=8,8,40,40 mask=16,16,48,48
=== Photoshop ===
group 'Group A' opacity=255 blend=NORMAL visible=true
  layer 'Painted' opacity=153 blend=MULTIPLY visible=true bounds=8,8,40,40 mask=16,16,48,48
=== layer comparison ===
layer 'Painted' differing pixels: 0; first differing coordinate: None
```
