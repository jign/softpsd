# What Photoshop 27.5 writes

From a 64x64 RGB document, transparent background, one group holding one painted layer at 60%
Multiply with a raster mask. Fixture: `tests/fixtures/ps27-smoke.psd`, made by
`tools/photoshop/fixtures/smoke.jsx`.

Header: `8BPS`, version 1, 4 channels, 8-bit, RGB. Merged image RLE. Layer count negative: the
merged image's first alpha channel is transparency. ICC profile (sRGB, 3144 bytes) in image
resource 1039. Merged RGB is blended over white: the layer's `200, 30, 30` at alpha 153 is
stored as `222, 120, 120, 153`. psd-tools `topil()` undoes it; a writer that stores straight
RGB gets `163, 0, 0` back from it.

A transparent-background document gets an empty `Layer 1` at bounds 0,0,0,0, below the
group at the root. A new group is PassThrough and open (`lsct` kind 1). Its record's blend
key is `norm`; the `pass` is only inside `lsct` (`00 00 00 10 | 00 00 00 01 8BIM pass 00 00 00 00`,
16 bytes with a trailing sub-type; we write 12). A group's blend lives in `lsct`, not in
the record. Resolution resource 1005 is 72 ppi. The mask block is 20 bytes, flags 0.

Channel ids: -1 alpha, 0 1 2 RGB, -2 user mask. Every layer, including groups and the closing
`</Layer group>` record, carries all four colour channels (2 bytes each when empty).

Tagged blocks on every layer: `luni` unicode name, `lyid` layer id, `lspf` protected,
`lclr` sheet colour, `shmd` metadata, `fxrp` reference point. Groups add `lsct` section
divider. Pixel layers add `clbl` (`01 00 00 00`), `infx` (`00 00 00 00`), `knko`
(`00 00 00 00`). `lnsr` name source appears on default-named layers only.

A pixel layer with none of `clbl`, `infx`, `knko` and a ColorBurn, ColorDodge or Difference
blend reports `bounds` and `boundsNoMask` as the canvas, not the stored rect. The composite
is unchanged. Adding any one of the three blocks fixes it; Photoshop's own files have all
three. Measured on 1 × 1 layers in a 32 × 32 document.

Image resources written: caption digest, XMP, print information, resolution, print scale, alpha
channel names and identifiers, global angle and altitude, print flags, layer state, layer
selection ids, grids and guides, URL list, slices, pixel aspect ratio, ids seed, version info.

What the writer must emit from this list is decided per fixture against Photoshop, not from the
spec.
