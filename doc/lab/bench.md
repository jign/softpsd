# Bench

Intel i7-8700K, 48 GB, Windows 10. rustc 1.97.1, release builds. wasmtime 49.0.2. One run per
row; timings move by about 10% between runs.

Documents, from `examples/bench/docs.rs`:

- `artwork`: the phase 5 export's shape. 26 stroke layers cropped to their bounds, a group of
  five, a hidden layer, one layer with a page-sized mask.
- `dense`: 26 layers covering half to all of the page, 40% painted with noisy pixels. Worst case
  for PackBits.

## softpsd

`cargo run --release --example bench`, and with `--target wasm32-wasip1` for wasm.
Write heap is the working heap above the model; the output goes to a seekable byte counter that
keeps nothing. Read heap is the heap above the input bytes, so it includes the returned
document.

| target | document | side | model MB | write s | write heap KB | file MB | read s | read heap MB |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| native | artwork | 2048 | 42 | 0.06 | 31 | 7 | 0.10 | 58 |
| native | dense | 2048 | 244 | 0.33 | 31 | 54 | 0.37 | 260 |
| native | artwork | 8000 | 759 | 0.99 | 95 | 86 | 1.52 | 1003 |
| native | dense | 8000 | 3855 | 5.53 | 95 | 854 | 5.51 | 4099 |
| wasm32-wasip1 | artwork | 2048 | 42 | 0.08 | 30 | 7 | 0.11 | 58 |
| wasm32-wasip1 | dense | 2048 | 244 | 0.44 | 30 | 54 | 0.45 | 260 |
| wasm32-wasip1 | artwork | 8000 | 759 | 1.27 | 95 | 86 | 1.78 | 1003 |

The dense 8K model is 3.8 GB, past a 32-bit heap, so it has no wasm row.

The writer streams one row at a time and seeks back to fill in lengths and row counts. Its
working heap is the row counts and one row, so it grows with the document's height, not its
size. A caller writing into a `Vec` still holds the whole file there.

## Against ag-psd-rs

`cargo run --release` in `tools/compare-agpsd/`, native, ag-psd 0.3.0. Both libraries write
into a `Vec`, so both write heaps include the output file and the `Vec`'s growth. Each writes
from its own model, built before timing.

| library | document | side | write s | write heap MB | read s | read heap MB |
| --- | --- | --- | --- | --- | --- | --- |
| softpsd | artwork | 2048 | 0.06 | 12 | 0.09 | 58 |
| ag-psd-rs | artwork | 2048 | 0.09 | 114 | 0.10 | 54 |
| softpsd | dense | 2048 | 0.35 | 88 | 0.35 | 260 |
| ag-psd-rs | dense | 2048 | 0.51 | 602 | 0.50 | 244 |
| softpsd | artwork | 8000 | 1.03 | 96 | 1.49 | 1003 |
| ag-psd-rs | artwork | 8000 | 1.52 | 1939 | 2.74 | 942 |
| softpsd | dense | 8000 | 6.01 | 1414 | 5.54 | 4099 |
| ag-psd-rs | dense | 8000 | 8.69 | 9525 | 7.49 | 3855 |

ag-psd-rs takes 45 to 50% longer to write, with 7 to 20 times the write heap; into a file or a
counter, softpsd's write heap is under 100 KB. Read heap is about 6% larger in softpsd, which
holds one layer's planes while it interleaves them. ag-psd-rs reads slower in every row.
