# Tools

`setup.ps1` installs every external reader into gitignored folders under `tools/`. Run it once
per machine. Nothing external is committed.

- `photoshop/run.ps1 <script.jsx> [-Out <file>]` runs a script in the open Photoshop and prints its result. `-Out` reaches the script as `OUT`.
- `photoshop/smoke.jsx` builds the reference fixture and saves it to `OUT`.
- `readers/psd-tools-dump.py <file>` and `readers/ag-psd-dump.js <file>` print a file's tree
  through each reader.
- `timed.py <unit> <lane> -- <command>` runs a test command and appends its wall time to
  `test-time.csv`. Every test run goes through it.
