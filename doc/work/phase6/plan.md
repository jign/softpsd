# Phase 6: industrial wasm

Goal: a PSD/PSB library that never panics, never aborts on a hostile file, runs on wasm32 as a
first-class target, and writes large documents with the least working memory we can reach.
Public on GitHub, tagged `v0.1.0`, used as a git dependency. Not published to crates.io.

Position against ag-psd-rs: both pass our Photoshop writer gate on all twelve fixtures. At
8000 x 8000, dense, both write in about 7 s; ag-psd-rs needs 9.5 GB of write heap to our
1.7 GB, panics on bad input, and cannot embed an ICC profile. Numbers in `doc/lab/bench.md`. ag-psd-rs covers far more of the format. We do not compete on features; we compete
on memory, speed and failure behaviour.

In chunks, reviewed once per chunk. One commit per item, one line each.

## Rules for this phase

- The crate stays dependency-free, dev-dependencies included. Anything else lives in its own
  Cargo project under `tools/`.
- Reader and writer output does not change. The only behaviour change allowed is a panic or
  an abort becoming an `Error`.
- Numbers are recorded, never asserted. No test asserts a timing or a byte count.
- No comment references a doc.
- Every fuzz or bench run has a fixed cap: a case count or a time budget, and a printed seed.

## Chunk 1: housekeeping

- Push `master`.
- `Cargo.toml`: `include = ["src/**", "README.md", "LICENSE-*", "Cargo.toml"]`. Keeps
  `cargo package` meaningful as a self-containment check. `publish` stays `false`.
- `tools/gate-all.ps1`: `cargo package` after `cargo test`.
- README: the "Not ready" notice comes down. Status line: 0.1, git dependency only, the
  goals of this phase in one sentence each. Install line:
  `softpsd = { git = "https://github.com/jign/softpsd", tag = "v0.1.0" }`.
- README: one line pointing to ag-psd-rs for text layers, effects and high bit depth.

Gate: `gate-all.ps1` passes.

## Chunk 2: API surface for 0.1

- `rle`, `validate`, `blend` become private modules. Soft Edge uses none of them directly.
  `Blend::key` and `Blend::from_key` go with `blend`.
- `#[non_exhaustive]` on `Error` and `Blend`. Not on the model structs: callers build them
  with struct literals.
- `Header` keeps its raw `u16` fields: it reports what a file is before we refuse it.

Gate: Soft Edge `dev/0.4.0` builds against the trimmed crate through a local `path`
override, not committed there; `gate-all.ps1` passes.

## Chunk 3: never panics

- Lints, library code only (`cfg_attr(not(test), ...)`): deny `clippy::unwrap_used`,
  `expect_used`, `panic`, `unreachable`, `todo`, `indexing_slicing`,
  `arithmetic_side_effects`, `cast_possible_truncation`. Fix every hit; each fix returns an
  `Error` or is proven by the types. An `#[allow]` carries a one-line reason.
- usize is 32 bits on wasm32. Every size computed from file fields uses checked u64
  arithmetic and converts with `try_from`. A 300,000 px PSB rect must refuse, not wrap.
- Allocation bounds: no buffer is sized from a file field before that field is checked
  against the input. Raw data cannot exceed the bytes left; PackBits output cannot exceed
  the declared rows. A file that claims more than it can hold is `Malformed`.
- `read_with_limit(input, max_decoded_bytes)`: refuses before allocating when the decoded
  document would exceed the cap. `read` is `read_with_limit` without a cap. A wasm host sets
  it from its heap headroom.
- `examples/fuzz.rs`: byte mutations (flip, truncate, splice, length-field overwrite) of every
  fixture, `catch_unwind` around `read`, fails on the first panic and prints the seed and the
  mutated file. Arguments: seed, case count. `gate-all.ps1` runs it with a fixed seed and a
  count that finishes in under 30 s.

Gate: clippy clean with the lints; the fuzzer runs 1,000,000 cases once, by hand, with no
panic; the corpus still reads or refuses as before.

## Chunk 4: wasm32 as a target

- `cargo build --target wasm32-unknown-unknown` in `gate-all.ps1`.
- `cargo test --target wasm32-wasip1` under wasmtime in `gate-all.ps1`: the fixture round
  trips and the fuzzer's fixed-seed run. `tools/setup.ps1` installs the target and checks for
  wasmtime; the gate fails when either is missing, never skips.
- One test at the wasm32 boundary: a document whose decoded size passes `u32::MAX` refuses
  on wasm32 and native alike.

Gate: both wasm steps pass inside `gate-all.ps1`.

## Chunk 5: measure

- `examples/bench.rs`: a counting global allocator in the example itself, std only. Write
  and read at 2K and 8K, 26 layers, the phase 5 shape plus a dense synthetic one. Prints time,
  peak heap above the input model, output size.
- Runs native and under wasmtime.
- `tools/compare-agpsd/`: its own Cargo project, `ag-psd` pinned, the same documents through
  ag-psd-rs. Gitignored `target/`.
- `doc/lab/bench.md`: machine, toolchains, one table. Overwritten when rerun.

Gate: the table exists for native and wasm.

## Chunk 6: writer memory

The writer buffers the whole layer section before writing it, so its peak is about twice the
output file: 1.6 GB for a 794 MB file in the dense 8K case.

- Find where the peak goes, from the bench, before changing anything.
- Target: working heap bounded by the largest layer's encoded channels plus the merged image,
  not by the document. The section lengths PSD needs up front come from a sizing pass over
  the encoded layers, or from `Write + Seek` back-patching. The reviewer picks after the
  numbers.
- Output stays byte-identical: the fixture gates and the phase 5 three-export check prove it.

Gate: the bench shows the new peak; every fixture writes the same bytes as before.

## Chunk 7: tag

- Rustdoc on every public item, `#![warn(missing_docs)]`. The README examples compiled as
  doctests through `#![doc = include_str!("../README.md")]`.
- `CHANGELOG.md`: `0.1.0`, five lines at most.
- Tag `v0.1.0`, push it.

Gate: tag on GitHub; `gate-all.ps1` green at the tag.

## After the phase

- Soft Edge moves its `rev` to `tag = "v0.1.0"` and sets `read_with_limit` where it reads.
  `write` now takes `Write + Seek`: its `Vec` goes through `std::io::Cursor`. Its own unit, in
  its repo.
