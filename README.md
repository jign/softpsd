# softpsd

Read and write Adobe Photoshop PSD and PSB files in Rust.

Built for [Soft Edge](https://soft-edge-roadmap.pages.dev). Write path first.

## Scope

- Write: 8-bit RGB and RGBA, layers, groups, raster layer masks, blend modes, opacity,
  visibility, names, merged image. PSD, and PSB above 30,000 px per side.
- Read: the subset the writer emits. Anything else is refused with `Error::Unsupported`,
  never guessed.

Not planned: 16 and 32-bit, CMYK, duotone, vector masks, smart objects, text layers.

## Contributing

A PR that changes the writer comes with a fixture a real Photoshop opened.
A reader bug report comes with the file that broke it.

## Licence

MIT or Apache-2.0, at your option.
