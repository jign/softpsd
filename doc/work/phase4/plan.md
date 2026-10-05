# Phase 4: gate automation

Goal: one command runs every check we have, the reader survives every file in the ag-psd
and psd-tools test corpora, and the two open-source compositors report on our fixtures.

In chunks, reviewed once per chunk. Commits inside a chunk are one per tool or one per
reader fix, one line each.

Read first: `doc/spec/gates.md`, `doc/spec/read.md`, `tools/README.md`.

## Rules for this phase

- The crate stays dependency-free. Everything external lives under `tools/` or `corpus/`,
  both installed by script, nothing committed.
- The reader may change in chunk 2 only to fix a reading rule: a length, a padding, an
  optional block, a channel it should skip. A fix that would add a model field, or change
  what the writer emits, is a finding for the reviewer, not a change.
- No comment references a doc. No test asserts a byte offset, a key or a constant.
- Every reader fix commit names the corpus file that exposed it.

## Chunk 1: the third reader, and gates.md made true

- `tools/readers/photoshopapi-dump.py <file>`: prints the tree through PhotoshopAPI
  (`import photoshopapi`, `LayeredFile_8bit.read`) in the same shape as the other two
  dumps: name, nesting, blend, opacity, visibility, clipping, rect, mask rect and default.
- `tools/writer-gate.ps1` runs it after ag-psd, before Photoshop.
- `doc/spec/gates.md`: writer gate steps 1 and 2 do not run on `cargo test`; 1 runs on
  `cargo test` through the fixture loops, 2 runs through `gate-fixtures.ps1`. Fix the
  sentence. The third-party table gets PhotoshopAPI's dump path.
- `tools/README.md`: one line for the new dump.

Gate: `tools\gate-fixtures.ps1` passes with three library dumps per fixture, and the
PhotoshopAPI tree agrees with the other two on all twelve. Where it disagrees, that is a
finding for `doc/lab/readers.md`, not a gate failure, if Photoshop sides with us.

## Chunk 2: the corpus

- `tools/fetch-corpus.ps1`: shallow sparse clones of ag-psd (`test/`) and psd-tools
  (`tests/psd_files/`) into `corpus/ag-psd/` and `corpus/psd-tools/`. Idempotent; rerun
  pulls. Prints the count of `.psd` and `.psb` files found.
- A `corpus` Cargo feature, no dependencies behind it, and one test in `tests/corpus.rs`
  compiled only with it: `corpus_never_panics`. It walks `corpus/` for `.psd` and `.psb`,
  reads each file inside `catch_unwind`, and writes `target/corpus.txt` with one line per
  file: `ok`, `unsupported: <reason>`, `unsupported layer '<name>': <reason>`,
  `malformed: <reason>`, or `panic`. It asserts that no file panicked and that `corpus/` was
  not empty. Nothing else is asserted; the rest is triage. Files over 64 MB are skipped and
  listed as `skipped: size`.
- `tools/readers/corpus-check.py`: for every file in `corpus/`, runs `examples/dump.rs` and
  psd-tools, and classifies: `match` when our tree equals psd-tools' tree in the dump line
  format (psd-tools blend names with the underscores removed, groups without bounds, masks
  as on/off), `differs` with the first differing line, `refused` with our reason, or
  `psd-tools failed`. Prints a summary count per class and writes the detail to
  `target/corpus-check.txt`. Exit 0 whatever the counts; the file is the output.
- `tools/corpus-triage.ps1`: for every `malformed` line in `target/corpus.txt`, opens the
  file in Photoshop through `photoshop/opens.jsx` (open, read the layer count, close,
  return `opens` or the error text) and writes `target/corpus-triage.txt`. A file that
  Photoshop opens and we call malformed is a reader bug.

Then the triage, in this order, each fix its own commit naming the file:

1. Every panic is fixed first, whatever the cause.
2. Every `malformed` on a file Photoshop opens: fix if it is a reading rule; otherwise a
   finding.
3. `differs` lines: read the first ten. A disagreement where psd-tools is wrong goes to
   `doc/lab/readers.md`. One where we are wrong is a fix or a finding by the same rule.

Gate: `cargo test --features corpus` passes, `corpus-check.py` runs to completion, and the
report lists the counts and every finding with the file name.

## Chunk 3: composites and the one command

- `tools/composite-gate.ps1 <psd>`: exports the file to PNG through Krita
  (`krita.exe --export <psd> --export-filename <png>`) and GIMP 3
  (`gimp-console-3.x.exe -b` with a short Script-Fu that loads the PSD, flattens, exports),
  each when the program is found in `PATH` or its default install folder, then runs
  `compare-merged.py` against each PNG. A missing program prints `skipped: <name> not
  installed` and is not a failure. A pixel difference is printed, not a failure: those
  engines recomposite from the layers with their own blending, and the tolerance of 1
  will not hold for every blend mode. The report says which fixtures differ and by how
  much.
- `tools/gate-all.ps1`: in order, `setup.ps1` check (venv and node_modules present, else
  stop with the command to run), `cargo fmt --check`, `cargo clippy --all-targets`,
  `cargo test`, `gate-fixtures.ps1`, `cargo test --features corpus` when `corpus/` exists,
  `composite-gate.ps1` on every `softpsd-*` fixture. Stops at the first failure and names
  the step. This is the release gate phase 6 will run.
- `tools/README.md`: the one command at the top, then the rest.

Gate: `tools\gate-all.ps1` passes end to end on this machine, with the composite step
either run or skipped per program, and says so.

## Handoff

`doc/work/phase4/report.md`, one section per chunk: corpus counts per class, every
finding with its file, the composite results per fixture and engine. Under two pages.
