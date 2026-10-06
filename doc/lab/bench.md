# Bench

Intel i7-8700K, 48 GB, Windows 10. rustc 1.97.1, release builds. wasmtime 49.0.2.

Documents, from `examples/bench/docs.rs`:

- `artwork`: the phase 5 export's shape. 26 stroke layers cropped to their bounds, a group of
  five, a hidden layer, one layer with a page-sized mask.
- `dense`: 26 layers covering half to all of the page, 40% painted with noisy pixels. Worst case
  for PackBits.

## softpsd

`cargo run --release --example bench`, and with `--target wasm32-wasip1` for wasm.
Write heap is the working heap above the model; the output goes to a byte counter. Read heap
is the heap above the input bytes, so it includes the returned document.

| target | document | side | model MB | write s | write heap MB | file MB | read s | read heap MB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| native | artwork | 2048 | 42 | 0.07 | 11 | 7 | 0.10 | 58 |
| native | dense | 2048 | 244 | 0.47 | 60 | 54 | 0.42 | 260 |
| native | artwork | 8000 | 759 | 1.13 | 161 | 86 | 1.50 | 1003 |
| native | dense | 8000 | 3855 | 6.80 | 919 | 854 | 5.55 | 4099 |
| wasm32-wasip1 | artwork | 2048 | 42 | 0.08 | 11 | 7 | 0.10 | 58 |
| wasm32-wasip1 | dense | 2048 | 244 | 0.46 | 60 | 54 | 0.44 | 260 |
| wasm32-wasip1 | artwork | 8000 | 759 | 1.29 | 161 | 86 | 1.77 | 1003 |

The dense 8K model is 3.8 GB, past a 32-bit heap, so it has no wasm row.

The writer's working heap is about the size of the output file: it holds every encoded channel
until the section lengths are known.

## Against ag-psd-rs

`cargo run --release` in `tools/compare-agpsd/`, native, ag-psd 0.3.0. Both libraries write
into a `Vec`, so both write heaps include the output file. Each writes from its own model, built
before timing.

| library | document | side | write s | write heap MB | read s | read heap MB |
| --- | --- | --- | --- | --- | --- | --- |
| softpsd | artwork | 2048 | 0.07 | 17 | 0.09 | 58 |
| ag-psd-rs | artwork | 2048 | 0.09 | 114 | 0.10 | 54 |
| softpsd | dense | 2048 | 0.47 | 110 | 0.37 | 260 |
| ag-psd-rs | dense | 2048 | 0.51 | 602 | 0.50 | 244 |
| softpsd | artwork | 8000 | 1.16 | 243 | 1.44 | 1003 |
| ag-psd-rs | artwork | 8000 | 1.43 | 1939 | 1.49 | 942 |
| softpsd | dense | 8000 | 7.12 | 1711 | 5.40 | 4099 |
| ag-psd-rs | dense | 8000 | 7.79 | 9525 | 7.32 | 3855 |

Write speed is close. Write heap is 5.6 to 8 times larger in ag-psd-rs. Read heap is about
6% larger in softpsd, which holds one layer's planes while it interleaves them; ag-psd-rs reads
up to 35% slower.
