//! Mutation fuzzer: `cargo run --release --example fuzz -- <seed> <cases>`.
//! Mutates the fixtures, reads each result, writes back whatever reads. Stops at the first panic
//! and saves the input to `target/fuzz-crash.psd`.

use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::sync::Mutex;

static PANIC: Mutex<String> = Mutex::new(String::new());

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

fn interesting(rng: &mut Rng, len: usize) -> u64 {
    match rng.below(8) {
        0 => 0,
        1 => 1,
        2 => u64::MAX,
        3 => i64::MAX as u64,
        4 => 1 << 63,
        5 => len as u64,
        6 => rng.below(256) as u64,
        _ => rng.next(),
    }
}

fn mutate(bytes: &mut Vec<u8>, other: &[u8], rng: &mut Rng) {
    for _ in 0..=rng.below(4) {
        if bytes.is_empty() {
            return;
        }
        let at = rng.below(bytes.len());
        match rng.below(6) {
            0 => bytes[at] ^= 1 << rng.below(8),
            1 => {
                let width = [1, 2, 4, 8][rng.below(4)];
                let value = interesting(rng, bytes.len()).to_be_bytes();
                for (slot, byte) in bytes[at..].iter_mut().zip(&value[8 - width..]) {
                    *slot = *byte;
                }
            }
            2 => bytes.truncate(at),
            3 => {
                let from = rng.below(other.len());
                let len = rng.below(64).min(other.len() - from);
                for (slot, byte) in bytes[at..].iter_mut().zip(&other[from..from + len]) {
                    *slot = *byte;
                }
            }
            4 => {
                let len = rng.below(64).min(bytes.len() - at);
                bytes.drain(at..at + len);
            }
            _ => {
                let len = rng.below(64).min(bytes.len() - at);
                let copy = bytes[at..at + len].to_vec();
                bytes.splice(at..at, copy);
            }
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let seed: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(1);
    let cases: u64 = args.next().and_then(|a| a.parse().ok()).unwrap_or(10_000);

    let root = if cfg!(target_os = "wasi") {
        Path::new(".")
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
    };
    let mut inputs: Vec<(String, Vec<u8>)> = std::fs::read_dir(root.join("tests/fixtures"))
        .expect("tests/fixtures")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let ext = path.extension()?.to_str()?;
            matches!(ext, "psd" | "psb").then(|| {
                let name = path.file_name()?.to_str()?.to_owned();
                Some((name, std::fs::read(&path).ok()?))
            })?
        })
        .collect();
    inputs.sort();
    assert!(!inputs.is_empty(), "no fixtures");

    panic::set_hook(Box::new(|info| {
        *PANIC.lock().unwrap_or_else(|e| e.into_inner()) = info.to_string();
    }));

    let mut rng = Rng(seed.max(1));
    let (mut read, mut refused) = (0u64, 0u64);
    for case in 0..cases {
        let (name, base) = &inputs[rng.below(inputs.len())];
        let other = &inputs[rng.below(inputs.len())].1;
        let mut bytes = base.clone();
        mutate(&mut bytes, other, &mut rng);
        let limit = if case % 4 == 0 {
            rng.next() >> rng.below(64)
        } else {
            u64::MAX
        };
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            let doc = softpsd::read_with_limit(&bytes, limit)?;
            let format = softpsd::format_for(doc.width, doc.height);
            softpsd::write(&doc, format, &mut std::io::Cursor::new(Vec::new()))
        }));
        match outcome {
            Ok(Ok(())) => read += 1,
            Ok(Err(_)) => refused += 1,
            Err(_) => {
                let crash = root.join("target/fuzz-crash.psd");
                std::fs::write(&crash, &bytes).ok();
                let message = PANIC.lock().unwrap_or_else(|e| e.into_inner()).clone();
                eprintln!("fuzz: panic at case {case}, seed {seed}, from {name}");
                eprintln!("{message}");
                eprintln!("input saved to {}", crash.display());
                std::process::exit(1);
            }
        }
    }
    println!(
        "fuzz: seed {seed}, {cases} cases, {read} read and written, {refused} refused, no panic"
    );
}
