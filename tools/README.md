# Tools

`setup.ps1` installs every external reader into gitignored folders under `tools/`. Run it once
per machine. Nothing external is committed.

- `photoshop/run.ps1 <script.jsx> [-Out <file>]` runs a script in the open Photoshop and prints its result. `-Out` reaches the script as `OUT`.
- `photoshop/smoke.jsx` builds the reference fixture and saves it to `OUT`.
- `readers/psd-tools-dump.py <file>` and `readers/ag-psd-dump.js <file>` print a file's tree
  through each reader.

Writer check:

```powershell
cargo run --example smoke -- target/softpsd-smoke.psd
tools\writer-gate.ps1 target\softpsd-smoke.psd
```

The gate runs psd-tools, ag-psd, the Photoshop tree dump and PNG export, then compares
the PNG against the stored merged image. It stops at the first tool failure. Photoshop
must be running, and the input PSD must not already be open. Inspect the tree dumps
against the expected model before accepting a fixture.
