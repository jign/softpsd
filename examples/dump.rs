use softpsd::{Blend, Document, Mask, Node, Rect};
use std::path::Path;

fn bounds(rect: Rect) -> String {
    format!("{},{},{},{}", rect.left, rect.top, rect.right, rect.bottom)
}

fn name(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace(['\r', '\n'], " ")
}

fn mask_bounds(mask: Option<&Mask>) -> String {
    mask.map_or_else(String::new, |mask| format!(" mask={}", bounds(mask.rect)))
}

fn blend_name(blend: Blend) -> String {
    if blend == Blend::Color {
        String::from("COLORBLEND")
    } else {
        format!("{blend:?}").to_uppercase()
    }
}

fn print_tree(nodes: &[Node], depth: usize) {
    let indent = "  ".repeat(depth);
    for node in nodes.iter().rev() {
        match node {
            Node::Group(group) => {
                println!(
                    "{indent}group '{}' opacity={} blend={} visible={}{}",
                    name(&group.name),
                    group.opacity,
                    blend_name(group.blend),
                    group.visible,
                    mask_bounds(group.mask.as_ref()),
                );
                print_tree(&group.children, depth + 1);
            }
            Node::Layer(layer) => println!(
                "{indent}layer '{}' opacity={} blend={} visible={} bounds={}{}",
                name(&layer.name),
                layer.opacity,
                blend_name(layer.blend),
                layer.visible,
                bounds(layer.pixels.rect),
                mask_bounds(layer.mask.as_ref()),
            ),
        }
    }
}

fn export_pixels(nodes: &[Node], input: &Path, number: &mut usize) -> std::io::Result<()> {
    for node in nodes {
        match node {
            Node::Group(group) => export_pixels(&group.children, input, number)?,
            Node::Layer(layer) => {
                *number += 1;
                let mut path = input.as_os_str().to_owned();
                path.push(format!(".{number}.rgba"));
                let path = Path::new(&path);
                std::fs::write(path, &layer.pixels.data)?;
                println!(
                    "{number} {} {}x{} {}",
                    layer.name.replace(['\r', '\n'], " "),
                    layer.pixels.rect.width(),
                    layer.pixels.rect.height(),
                    path.display(),
                );
            }
        }
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let input = args.next().ok_or("usage: dump <psd>")?;
    if args.next().is_some() {
        return Err("usage: dump <psd>".into());
    }
    let input = Path::new(&input);
    let Document { layers, .. } = softpsd::read(&std::fs::read(input)?)?;
    print_tree(&layers, 0);
    export_pixels(&layers, input, &mut 0)?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
