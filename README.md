# softpsd

Read and write Adobe Photoshop PSD and PSB files in Rust. Built for wasm.

Version 0.1, used in production by [Soft Edge](https://softedge.pages.dev/). Not on crates.io;
depend on it through git:

```toml
softpsd = { git = "https://github.com/jign/softpsd", tag = "v0.1.0" }
```

## Use

Write. softpsd stores the merged image you give it; it does not composite. `write` streams to
any `Write + Seek`: a file, or a `Vec` through `Cursor`.

```rust
use softpsd::{Blend, Channels, Document, Image, Layer, Node, Rect};

fn main() -> softpsd::Result<()> {
    let page = Rect { top: 0, left: 0, bottom: 64, right: 64 };
    let red = Image { rect: page, data: [255, 0, 0, 255].repeat(64 * 64) };
    let layer = Layer {
        name: String::from("Red"),
        visible: true,
        opacity: 255,
        blend: Blend::Normal,
        clip_to_below: false,
        pixels: red.clone(),
        mask: None,
    };
    let doc = Document {
        width: 64,
        height: 64,
        channels: Channels::Rgba,
        icc_profile: None,
        resolution_dpi: Some(72.0),
        layers: vec![Node::Layer(layer)],
        merged: red,
    };
    let mut out = std::io::Cursor::new(Vec::new());
    softpsd::write(&doc, softpsd::format_for(64, 64), &mut out)?;
    Ok(())
}
```

Read. `read_with_limit` refuses before allocating when the pixels would pass the cap; a wasm
host sets it from its free heap.

```rust,no_run
use softpsd::Node;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = std::fs::read("art.psd")?;
    let doc = softpsd::read_with_limit(&bytes, 1 << 30)?;
    for node in &doc.layers {
        match node {
            Node::Layer(layer) => println!("{}", layer.name),
            Node::Group(group) => println!("{}/", group.name),
        }
    }
    Ok(())
}
```

## Goals

- Never panic. A hostile or broken file returns an `Error`.
- Never abort. No allocation is sized from a file field the input cannot back.
- wasm32 is a first-class target, tested in the release gate.
- Low working memory and fast writes for large documents.
- No dependencies.

Every fixture is checked against Photoshop: the file opens with the same layer tree, and
Photoshop's own render equals our merged image.

## Scope

- Write: 8-bit RGB and RGBA, layers, groups, raster layer masks, blend modes, opacity,
  visibility, names, merged image, ICC profile. PSD, and PSB above 30,000 px per side.
- Read: the subset the writer emits. Anything else is refused with `Error::Unsupported`,
  never guessed.

Not supported: 16 and 32-bit, CMYK, duotone, vector masks, layer effects, smart objects, text
layers. For those, [ag-psd-rs](https://crates.io/crates/ag-psd) covers far more of the format.

## Docs

- `doc/spec/model.md`: the data model and the byte layout it maps to.
- `doc/spec/write.md`, `doc/spec/read.md`: what each direction does and refuses.
- `doc/spec/gates.md`: fixtures, the Photoshop oracle, third-party cross-checks.
- `doc/work/`: the plan and one folder per phase. Internal: the order the maintainers are
  building it in, not a list of open tasks.
- `doc/lab/`: findings. `doc/ref/`: external specs and reference clones.

## Contributing

Issues are welcome: a file that misbehaves, a reader that disagrees with Photoshop, a quirk we
have not recorded. Attach the file.

Open an issue before a pull request. A PR that changes the writer comes with a fixture a real
Photoshop opened.

## Licence

MIT or Apache-2.0, at your option.
