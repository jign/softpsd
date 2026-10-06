# Phase 6 gate checks

Tag `v0.1.0` at `90c0246`, gate green there. A scratch project depending on the tag through
git reads `ps27-smoke.psd`, writes it and reads back an equal document.

## Chunks

1. Housekeeping. README notice down; package holds `src/`, README, changelog, licences.
2. API. Modules private, the crate root is the API. `#[non_exhaustive]` on `Error` and `Blend`.
3. Never panics. Clippy denies every panicking construct in the library. `read_with_limit` and
   `Error::OverLimit` added. The fuzzer ran 1,000,000 cases with no panic and finds planted
   panics within about 60 cases. The corpus reads or refuses exactly as before.
4. wasm32. Library build, tests and a 100,000-case fuzz under wasmtime 49.0.2 in the gate. A
   layer whose pixel size passes `u32::MAX` refuses on both targets.
5. Measure. `examples/bench/`, `tools/compare-agpsd/`, results in `doc/lab/bench.md`.
6. Writer memory. `write` takes `Write + Seek`, streams one row at a time and seeks back for
   lengths. Write heap at 8K: 919 MB to 95 KB. Writes 15 to 20% faster. Output byte-identical
   on every fixture and a dense 2K file.
7. Docs. Rustdoc on every public item, `missing_docs` denied, README examples run as doc tests,
   `cargo doc` with warnings denied in the gate.

## Deviations from the plan

- The plan's first ag-psd-rs numbers timed its model conversion with its write. Measured
  fairly, ag-psd-rs takes 45 to 50% longer to write, not 2.7 times.
- Chunk 6 went past its target: the plan asked for the largest layer plus the merged image;
  the writer holds one row.
- The write spec said the writer never seeks. It never held: the old writer buffered every
  channel instead. `write` now requires `Seek`; a writer for plain `Write` needs a sizing pass.
- History: `b209e76` and `87822a0` each fail `cargo fmt --check` alone; the next commit fixes
  both. Both build and pass their tests.

## Soft Edge follow-up

Move the git `rev` to `tag = "v0.1.0"`; wrap the export's `Vec` in `std::io::Cursor`; read
through `read_with_limit`.
