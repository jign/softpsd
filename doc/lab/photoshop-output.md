# What Photoshop 27.5 writes

From a 64x64 RGB document, transparent background, one group holding one painted layer at 60%
Multiply with a raster mask. Fixture: `tests/fixtures/ps27-smoke.psd`, made by
`tools/photoshop/smoke.jsx`.

Header: `8BPS`, version 1, 4 channels, 8-bit, RGB. Merged image RLE. Layer count negative: the
merged image's first alpha channel is transparency. ICC profile (sRGB, 3144 bytes) in image
resource 1039.

A transparent-background document gets an empty `Layer 1` at bounds 0,0,0,0.

Channel ids: -1 alpha, 0 1 2 RGB, -2 user mask. Every layer, including groups and the closing
`</Layer group>` record, carries all four colour channels (2 bytes each when empty).

Tagged blocks on every layer: `luni` unicode name, `lyid` layer id, `lspf` protected,
`lclr` sheet colour, `shmd` metadata, `fxrp` reference point. Groups add `lsct` section
divider. Pixel layers add `clbl`, `infx`, `knko`. `lnsr` name source appears on default-named
layers only.

Image resources written: caption digest, XMP, print information, resolution, print scale, alpha
channel names and identifiers, global angle and altitude, print flags, layer state, layer
selection ids, grids and guides, URL list, slices, pixel aspect ratio, ids seed, version info.

What the writer must emit from this list is decided per fixture against Photoshop, not from the
spec.
