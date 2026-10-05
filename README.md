# softpsd

> **Not ready.** Nothing here reads or writes a file yet. Specs and tooling are being written; the API will change without notice. Do not depend on this crate.

Read and write Adobe Photoshop PSD and PSB files in Rust.

Built for [Soft Edge](https://soft-edge-roadmap.pages.dev). Write path first.

## Scope

- Write: 8-bit RGB and RGBA, layers, groups, raster layer masks, blend modes, opacity,
  visibility, names, merged image. PSD, and PSB above 30,000 px per side.
- Read: the subset the writer emits. Anything else is refused with `Error::Unsupported`,
  never guessed.

Not in v1: 16 and 32-bit, CMYK, duotone, vector masks, layer effects, smart objects, text layers. The format allows all of them; the model has no fields yet.

## Docs

- `doc/model.md`: the data model and the byte layout it maps to.
- `doc/write.md`, `doc/read.md`: what each direction does and refuses.
- `doc/gates.md`: fixtures, the Photoshop oracle, third-party cross-checks. `doc/plan.md`: phases.
- `doc/lab/`: findings. `doc/ref/`: external specs and reference clones.

## Contributing

A PR that changes the writer comes with a fixture a real Photoshop opened.
A reader bug report comes with the file that broke it.

## Licence

MIT or Apache-2.0, at your option.
