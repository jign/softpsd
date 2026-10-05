# Tools

`setup.ps1` installs every external reader into gitignored folders under `tools/`. Run it once
per machine. Nothing external is committed.

- `photoshop/run.ps1 <script.jsx> [-Out <file>]` runs a script in the open Photoshop and prints its result. `-Out` reaches the script as `OUT`.
- `photoshop/fixtures/smoke.jsx` builds the reference fixture and saves it to `OUT`.
- `readers/psd-tools-dump.py <file>` and `readers/ag-psd-dump.js <file>` print a file's tree
  through each reader.

Writer check:

```powershell
cargo run --example fixture -- smoke target/softpsd-smoke.psd
tools\writer-gate.ps1 target\softpsd-smoke.psd
```

The gate runs psd-tools, ag-psd, the Photoshop tree dump and PNG export, then compares
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
