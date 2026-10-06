# Changelog

## 0.1.0

- Read and write 8-bit RGB and RGBA PSD and PSB: layers, groups, raster masks, blend modes,
  opacity, visibility, names, merged image, ICC profile, resolution.
- Every fixture checked against Photoshop 27, psd-tools, ag-psd and PhotoshopAPI.
- Never panics: clippy denies every panicking construct, and a fuzzer runs in the release gate.
- `write` streams one row at a time to any `Write + Seek`; `read_with_limit` caps pixel bytes.
- wasm32 tested under wasmtime.
