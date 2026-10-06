//! Write and read timings with peak heap: `cargo run --release --example bench`.
//! Records numbers; asserts nothing.

mod docs;

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

struct Counting;

static CURRENT: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            let now = CURRENT.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PEAK.fetch_max(now, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        CURRENT.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new = unsafe { System.realloc(ptr, layout, new_size) };
        if !new.is_null() {
            CURRENT.fetch_sub(layout.size(), Ordering::Relaxed);
            let now = CURRENT.fetch_add(new_size, Ordering::Relaxed) + new_size;
            PEAK.fetch_max(now, Ordering::Relaxed);
        }
        new
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

const MB: f64 = 1024.0 * 1024.0;

// Tracks position and length and keeps no bytes, so write heap excludes the output.
#[derive(Default)]
struct Counter {
    position: u64,
    len: u64,
}

impl std::io::Write for Counter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.position += buf.len() as u64;
        self.len = self.len.max(self.position);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl std::io::Seek for Counter {
    fn seek(&mut self, to: std::io::SeekFrom) -> std::io::Result<u64> {
        self.position = match to {
            std::io::SeekFrom::Start(at) => at,
            std::io::SeekFrom::End(delta) => self.len.saturating_add_signed(delta),
            std::io::SeekFrom::Current(delta) => self.position.saturating_add_signed(delta),
        };
        Ok(self.position)
    }
}

// Time and heap growth above the heap at the start of `f`.
fn measure<T>(f: impl FnOnce() -> T) -> (T, Duration, f64) {
    let base = CURRENT.load(Ordering::Relaxed);
    PEAK.store(base, Ordering::Relaxed);
    let start = Instant::now();
    let value = f();
    let time = start.elapsed();
    (
        value,
        time,
        (PEAK.load(Ordering::Relaxed) - base) as f64 / MB,
    )
}

fn main() {
    let target = if cfg!(target_os = "wasi") {
        "wasm32-wasip1"
    } else {
        "native"
    };
    let mut cases: Vec<(&str, u32)> = vec![("artwork", 2048), ("dense", 2048), ("artwork", 8000)];
    // The dense 8K model alone is 3.8 GB, past a 32-bit heap.
    if cfg!(target_pointer_width = "64") {
        cases.push(("dense", 8000));
    }
    println!(
        "| target | document | side | model MB | write s | write heap KB | file MB | read s | read heap MB |"
    );
    println!("| --- | --- | --- | --- | --- | --- | --- | --- | --- |");
    for (name, side) in cases {
        let (doc, _, model) = measure(|| match name {
            "dense" => docs::dense(side),
            _ => docs::artwork(side),
        });
        let format = softpsd::format_for(side, side);
        let (file_size, write_time, write_heap) = measure(|| {
            let mut out = Counter::default();
            softpsd::write(&doc, format, &mut out).expect("write");
            out.len
        });
        let mut bytes = std::io::Cursor::new(Vec::new());
        softpsd::write(&doc, format, &mut bytes).expect("write");
        let bytes = bytes.into_inner();
        drop(doc);
        let (back, read_time, read_heap) = measure(|| softpsd::read(&bytes).expect("read"));
        drop(back);
        println!(
            "| {target} | {name} | {side} | {model:.0} | {:.2} | {:.0} | {:.0} | {:.2} | {read_heap:.0} |",
            write_time.as_secs_f64(),
            write_heap * 1024.0,
            file_size as f64 / MB,
            read_time.as_secs_f64(),
        );
    }
}
