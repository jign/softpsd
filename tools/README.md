# Tools

Run every release check with `tools\gate-all.ps1` (Photoshop must be open).

`setup.ps1` installs every external reader into gitignored folders under `tools/`. Run it once
per machine. Nothing external is committed.

- `photoshop/run.ps1 <script.jsx> [-Out <file>]` runs a script in the open Photoshop and prints its result. `-Out` reaches the script as `OUT`.
- `photoshop/fixtures/smoke.jsx` builds the reference fixture and saves it to `OUT`.
- `readers/psd-tools-dump.py <file>` and `readers/ag-psd-dump.js <file>` print a file's tree
  through each reader.
- `readers/photoshopapi-dump.py <file>` prints the tree and mask metadata through PhotoshopAPI.

Writer check:

```powershell
cargo run --example fixture -- smoke target/softpsd-smoke.psd
tools\writer-gate.ps1 target\softpsd-smoke.psd
```

The gate runs psd-tools, ag-psd, PhotoshopAPI, the Photoshop tree dump and PNG export, then compares
the PNG against the stored merged image. It stops at the first tool failure. Photoshop
must be running, and the input PSD must not already be open. Inspect the tree dumps
against the expected model before accepting a fixture.

Fixture checks:

```powershell
tools\gate-fixtures.ps1 smoke
tools\gate-fixtures.ps1
tools\gate-fixtures.ps1 -Make smoke
```

The fixture gate builds each requested document, runs its writer and reader gates, and
runs the reader gate on its Photoshop fixture when present. With no names, it runs the
catalog from `tests/common/fixtures.rs`, exposed by `cargo run --example fixture -- --list`.
`-Make` first creates each Photoshop fixture with its script, replacing the saved fixture.
Failures name the fixture and gate. The individual writer and reader gates accept PSD
and PSB paths.

The wide `psb` fixture uses `.psb` paths in the catalog and gate runner. PNG exports
use native saving so widths above 30,000 pixels avoid Save for Web size warnings.
Pixel comparisons use stored values without ICC conversion. For version 2 files only,
the psd-tools adapter selects the PSB side limit while retaining allocation guards.

Corpus checks:

```powershell
tools\fetch-corpus.ps1
cargo test --features corpus
tools\.venv\Scripts\python.exe tools\readers\corpus-check.py
tools\corpus-triage.ps1
```

The fetcher keeps shallow sparse upstream clones in gitignored `corpus/`; reruns pull.
The optional test checks every PSD/PSB for reader panics and writes `target/corpus.txt`;
files above 64 MB are listed as skipped. The tree comparator builds the Rust dump once,
compares each file with psd-tools and writes `target/corpus-check.txt`. Its classification
counts are informational. The Photoshop triage tool opens each malformed file, counts
root layers, closes without saving and writes `target/corpus-triage.txt`; it skips files
already open in Photoshop.

Composite checks:

```powershell
tools\composite-gate.ps1 tests\fixtures\softpsd-smoke.psd
```

The gate finds Krita and GIMP 3 in PATH or Program Files on local fixed drives; optional
`-Krita` and `-Gimp` executable paths override discovery. Krita uses its batch exporter;
GIMP uses a fresh console process and Script-Fu to load, flatten over white, and export PNG.
PNG files and GIMP logs go under `target/composites/`. Pixel differences report count,
first coordinate and largest channel difference and are informational. Export failures,
missing PNGs and comparison errors fail the gate. Missing engines are explicitly skipped.

`gate-all.ps1` checks the installed reader directories, then runs formatting, Clippy,
tests, all fixture gates, the corpus panic test when fetched, and composites on every
saved writer fixture. It stops at the first failure and names the step. Run
`tools\setup.ps1` if the external reader directories are missing.
