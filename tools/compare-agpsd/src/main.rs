//! softpsd against ag-psd-rs on the bench documents: `cargo run --release` from this folder.
//! Both write into a Vec, so both heap figures include the output file.

#[path = "../../../examples/bench/docs.rs"]
mod docs;

use ag_psd::psd::{self as ag, ReadOptions, WriteOptions};
use softpsd::{Blend, Channels, Document, Mask, Node, Rect};
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

fn blend(b: Blend) -> ag::BlendMode {
    use ag::BlendMode as A;
    match b {
        Blend::PassThrough => A::PassThrough,
        Blend::Multiply => A::Multiply,
        _ => A::Normal,
    }
}

fn pixels(rect: Rect, data: Vec<u8>) -> ag::PixelData {
    ag::PixelData {
        width: rect.width() as u32,
        height: rect.height() as u32,
        data,
    }
}

fn mask(mask: &Option<Mask>) -> Option<ag::LayerMaskData> {
    mask.as_ref().map(|m| ag::LayerMaskData {
        top: Some(m.rect.top as f64),
        left: Some(m.rect.left as f64),
        bottom: Some(m.rect.bottom as f64),
        right: Some(m.rect.right as f64),
        default_color: Some(m.default as f64),
        disabled: Some(m.disabled),
        image_data: Some(pixels(
            m.rect,
            m.data.iter().flat_map(|&v| [v, v, v, 255]).collect(),
        )),
        ..Default::default()
    })
}

fn node(n: &Node) -> ag::Layer {
    let mut l = ag::Layer::default();
    match n {
        Node::Group(g) => {
            l.additional_info.name = Some(g.name.clone());
            l.additional_info.mask = mask(&g.mask);
            l.hidden = Some(!g.visible);
            l.opacity = Some(g.opacity as f64 / 255.0);
            l.blend_mode = Some(blend(g.blend));
            l.opened = Some(g.expanded);
            l.children = Some(g.children.iter().map(node).collect());
        }
        Node::Layer(x) => {
            let r = x.pixels.rect;
            l.additional_info.name = Some(x.name.clone());
            l.additional_info.mask = mask(&x.mask);
            l.hidden = Some(!x.visible);
            l.opacity = Some(x.opacity as f64 / 255.0);
            l.blend_mode = Some(blend(x.blend));
            l.clipping = Some(x.clip_to_below);
            l.top = Some(r.top as f64);
            l.left = Some(r.left as f64);
            l.bottom = Some(r.bottom as f64);
            l.right = Some(r.right as f64);
            l.image_data = Some(pixels(r, x.pixels.data.clone()));
        }
    }
    l
}

fn to_ag(d: &Document) -> ag::Psd {
    ag::Psd {
        width: d.width as f64,
        height: d.height as f64,
        channels: Some(match d.channels {
            Channels::Rgb => 3.0,
            Channels::Rgba => 4.0,
        }),
        bits_per_channel: Some(8.0),
        color_mode: Some(ag::ColorMode::Rgb),
        children: Some(d.layers.iter().map(node).collect()),
        image_data: Some(pixels(d.merged.rect, d.merged.data.clone())),
        ..Default::default()
    }
}

fn main() {
    println!("| library | document | side | write s | write heap MB | read s | read heap MB |");
    println!("| --- | --- | --- | --- | --- | --- | --- |");
    for (name, side) in [
        ("artwork", 2048),
        ("dense", 2048),
        ("artwork", 8000),
        ("dense", 8000),
    ] {
        let doc = match name {
            "dense" => docs::dense(side),
            _ => docs::artwork(side),
        };
        let format = softpsd::format_for(side, side);
        let (bytes, ours_write, ours_write_heap) = measure(|| {
            let mut out = std::io::Cursor::new(Vec::new());
            softpsd::write(&doc, format, &mut out).expect("softpsd write");
            out.into_inner()
        });
        let model = to_ag(&doc);
        drop(doc);
        let (_, ours_read, ours_read_heap) =
            measure(|| softpsd::read(&bytes).expect("softpsd read"));
        drop(bytes);

        let options = WriteOptions {
            no_background: Some(true),
            ..Default::default()
        };
        let (bytes, ag_write, ag_write_heap) = measure(|| ag_psd::write_psd(&model, &options));
        drop(model);
        let read_options = ReadOptions {
            total_memory_limit: None,
            use_image_data: Some(true),
            ..Default::default()
        };
        let (_, ag_read, ag_read_heap) =
            measure(|| ag_psd::read_psd(&bytes, &read_options).expect("ag-psd read"));
        for (library, w, wh, r, rh) in [
            (
                "softpsd",
                ours_write,
                ours_write_heap,
                ours_read,
                ours_read_heap,
            ),
            ("ag-psd-rs", ag_write, ag_write_heap, ag_read, ag_read_heap),
        ] {
            println!(
                "| {library} | {name} | {side} | {:.2} | {wh:.0} | {:.2} | {rh:.0} |",
                w.as_secs_f64(),
                r.as_secs_f64()
            );
        }
    }
}
