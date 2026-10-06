# Phase 6: publish

Goal: `softpsd` 0.1.0 on crates.io, with a README and rustdoc a stranger can use, and an
API we are willing to keep for the 0.1 line.

In chunks, reviewed once per chunk. One commit per item, one line each.

## Rules for this phase

- The crate stays dependency-free, dev-dependencies included. Benches use `std::time`.
- No reader or writer behaviour changes. A finding that needs one goes to the reviewer.
- No comment references a doc. Rustdoc says what an item does and what it refuses, nothing
  else.
- `cargo publish` cannot be undone, only yanked. The PM runs it or approves it on the spot.

## Chunk 1: housekeeping

- Push `ce1c54f` (the phase 5 report) and this plan.
- `Cargo.toml`: `include = ["src/**", "README.md", "LICENSE-*", "Cargo.toml"]`. Tests and
  examples stay out: both pull in `tests/common/` and the fixtures, and `doc/`, `tools/`
  have no place in the package.
- `Cargo.toml`: `documentation = "https://docs.rs/softpsd"`, `readme = "README.md"`.
- Check `rust-version = "1.97"` is true: it is the toolchain we build on, so it stays unless
  a lower one is cheap to prove. Do not chase a lower MSRV.
- `tools/gate-all.ps1`: add `cargo package` after `cargo test`. It builds the packaged crate
  in isolation, so a missing file in `include` fails the gate.

Gate: `gate-all.ps1` passes; `cargo package --list` shows only `src/`, README, licences,
manifest.

## Chunk 2: API surface for 0.1

Everything public now is a promise. Review and cut:

- `rle` and `validate` are `pub mod`. `rle` is an implementation detail: make it private.
  `validate` runs inside `write`; keep `validate::validate` public only if Soft Edge calls
  it (check `dev/0.4.0`), otherwise private.
- `blend` exposes `Blend::key` and `Blend::from_key`, the four-byte Photoshop keys. Private
  unless a caller needs them.
- `Header` has raw `channel_count`, `depth`, `color_mode` as `u16`. Keep them: `read_header`
  exists to report what a file is before we refuse it, so raw values are the point.
- `#[non_exhaustive]` on `Error` and `Blend`. Not on the model structs: callers build them
  with struct literals, and Soft Edge does.
- `Error` variants carry `&'static str` reasons. Keep.

Gate: Soft Edge's `dev/0.4.0` still builds against the trimmed crate through a local `path`
override (not committed there); `gate-all.ps1` passes.

## Chunk 3: docs

- README: the "Not ready" notice comes down. Replace with a short status line: 0.1, 8-bit
  RGB/RGBA, write and read the subset in Scope, gated against Photoshop 27.
- README: a 15-line example that builds a two-layer `Document` and writes it, and one that
  reads a file and prints layer names. Both compiled: include the README as crate docs with
  `#![doc = include_str!("../README.md")]` so `cargo test` runs them as doctests.
- README "Contributing": issues welcome now; PRs that change the writer still need a fixture
  Photoshop opened. Drop the "while the notice is up" wording.
- README "Docs": `doc/` is not in the package; link to the GitHub tree instead of paths.
- Rustdoc on every public item. `#![warn(missing_docs)]` in `lib.rs`; clippy in the gate
  then catches a missing one.
- `CHANGELOG.md`: one section, `0.1.0`, five lines at most.

Gate: `cargo doc --no-deps` has no warnings; doctests pass; README reads correctly on the
GitHub page.

## Chunk 4: benches

- `examples/bench.rs`, release build, no harness: write and read at 2K and 8K, one layer and
  26 layers (the phase 5 shape), print ms and MB/s. Records numbers, asserts nothing.
- Results in `doc/lab/bench.md`: machine, toolchain, the table. One file, overwritten when
  rerun.
- Not in `gate-all.ps1`.

Gate: the numbers exist. No target. If 8K write is slower than Soft Edge's export budget,
that is a finding for the reviewer, not a blocker.

## Chunk 5: publish

- `publish = true` (or remove the line).
- `cargo publish --dry-run`, then the PM approves, then `cargo publish`. `cargo login` is
  the PM's step if no token is set.
- Tag `v0.1.0`, push the tag.
- Check docs.rs built the page.
- From a scratch project outside the repo: `cargo add softpsd`, read
  `tests/fixtures/softpsd-smoke.psd`, write it back, compare bytes.

Gate: on crates.io, docs.rs green, the scratch round trip matches.

## After the phase

- Soft Edge moves from the git `rev` to `softpsd = "0.1"`. Its own unit, in its repo.
- `doc/work/plan.md`: phase 6 green. Whether `doc/work/` stays in the public repo after
  0.1.0 is a PM call; it is excluded from the package either way.
