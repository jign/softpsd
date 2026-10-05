# References

External documents and code are not committed. `fetch.ps1` snapshots the spec and clones the
reference repositories into `doc/ref/local/`, which is gitignored. Run it once per machine;
rerun to update. adobe.com times out from some networks, so the spec comes from the Wayback
Machine.

What we rely on is written into `doc/model.md` in our own words, with the Adobe section named.
`doc/lab/photoshop-output.md` records what Photoshop actually writes. When the spec and that
file disagree, the lab file is right.

## Copy rule

Read everything. Copy nothing. The crate is written from the spec and from Photoshop's output;
the clones are there to answer "how does X handle this" and to seed the quirks list. GPL code
(GIMP, Krita) can never be copied into this repo. Permissive code could be, with attribution,
but is not: one licence file, no NOTICE churn.

## Sources

| Source | Licence | Use |
| --- | --- | --- |
| [Adobe Photoshop File Formats Specification](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/) | Adobe | The format. Last updated November 2019. Incomplete and wrong in places; Photoshop output wins. |
| [ag-psd](https://github.com/Agamnentzar/ag-psd) | MIT | Quirks learned by trial and error; the best open writer to compare bytes against. `README_PSD.md`, `src/psdWriter.ts`, `src/additionalInfo.ts`. |
| [psd-tools](https://github.com/psd-tools/psd-tools) | MIT | Clean reader of every section; `src/psd_tools/psd/` mirrors the spec's structure. |
| [PhotoshopAPI](https://github.com/EmilDohne/PhotoshopAPI) | BSD-3 | Modern C++ writer; compression and 16/32-bit. Sparse: `PhotoshopAPI/` only. |
| [psd (Rust)](https://github.com/chinedufn/psd) | MIT/Apache-2.0 | The existing Rust reader. What a Rust API for this looks like, and its gaps. |
| [psd_sdk](https://github.com/MolecularMatters/psd_sdk) | BSD-2 | C++ reader with a sample exporter; clear code for channel and mask layout. |
| [GIMP file-psd](https://gitlab.gnome.org/GNOME/gimp/-/tree/master/plug-ins/file-psd) | GPL-3 | `psd-load.c`, `psd-layer-res-load.c`, `psd-export.c`: decades of quirks in the comments. Sparse. |
| [Krita psd](https://invent.kde.org/graphics/krita) | GPL-3 | `libs/psd/` (records, sections, additional-info blocks) and `plugins/impex/psd/` (loader, saver). Sparse. |
| [ICC profile spec](https://www.color.org/specification/ICC.1-2022-05.pdf) | ICC | Only for what resource 1039 carries. |
