#[allow(dead_code)]
#[path = "../tests/common/fixtures.rs"]
mod fixtures;

use std::fs::File;
use std::io::{BufWriter, Write};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let name = args.next().ok_or("usage: fixture <name> <out> | --list")?;
    if name == "--list" && args.len() == 0 {
        for name in fixtures::names() {
            println!("{name}");
        }
        return Ok(());
    }
    let name = name.to_str().ok_or("fixture name must be UTF-8")?;
    if !fixtures::names().contains(&name) {
        return Err(format!("unknown fixture: {name}").into());
    }
    let path = args.next().ok_or("usage: fixture <name> <out>")?;
    if args.next().is_some() {
        return Err("usage: fixture <name> <out>".into());
    }
    let doc = fixtures::fixture(name).ours;
    let format = softpsd::format_for(doc.width, doc.height);
    let mut out = BufWriter::new(File::create(path)?);
    softpsd::write(&doc, format, &mut out)?;
    out.flush()?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
