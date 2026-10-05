# softpsd

> **Not ready.** Nothing here reads or writes a file yet. Specs and tooling are being written; the API will change without notice. Do not depend on this crate.

Read and write Adobe Photoshop PSD and PSB files in Rust.

Built for [Soft Edge](https://softedge.pages.dev/). Write path first.

## Scope

- Write: 8-bit RGB and RGBA, layers, groups, raster layer masks, blend modes, opacity,
  visibility, names, merged image. PSD, and PSB above 30,000 px per side.
- Read: the subset the writer emits. Anything else is refused with `Error::Unsupported`,
  never guessed.

Not in v1: 16 and 32-bit, CMYK, duotone, vector masks, layer effects, smart objects, text layers. The format allows all of them; the model has no fields yet.

## Docs

- `doc/spec/model.md`: the data model and the byte layout it maps to.
- `doc/spec/write.md`, `doc/spec/read.md`: what each direction does and refuses.
- `doc/spec/gates.md`: fixtures, the Photoshop oracle, third-party cross-checks.
- `doc/work/`: the plan and one folder per phase. Internal: this is the order the maintainers
  are building it in, not a list of open tasks. Do not pick a phase up on your own.
- `doc/lab/`: findings. `doc/ref/`: external specs and reference clones.

## Contributing

Not yet. While the notice at the top is up, pull requests are not taken; the design is still
moving and a PR would collide with it. Once it comes down: open an issue first.

A PR that changes the writer comes with a fixture a real Photoshop opened.
A reader bug report comes with the file that broke it.

## Licence

MIT or Apache-2.0, at your option.
