# References

External documents are not committed; Adobe's spec is copyrighted and the library sources have
their own licences. `fetch.ps1` snapshots them into `doc/ref/local/`, which is gitignored. adobe.com times out from some networks, so the spec comes from the Wayback Machine.
What we rely on is written into `doc/model.md` in our own words, with the section named.

| Source | Use |
| --- | --- |
| [Adobe Photoshop File Formats Specification](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/) | The format. Last updated November 2019. Incomplete and wrong in places; Photoshop output wins. |
| [ag-psd README_PSD.md](https://github.com/Agamnentzar/ag-psd/blob/master/README_PSD.md) and `src/` | Quirks learned by trial and error; the best writer to compare bytes against. |
| [psd-tools source](https://github.com/psd-tools/psd-tools) | Clean reader of every section; `psd_tools/psd/` mirrors the spec's structure. |
| [PhotoshopAPI docs](https://photoshopapi.readthedocs.io/) | Modern writer; its notes on compression and 16/32-bit. |
| [GIMP file-psd](https://gitlab.gnome.org/GNOME/gimp/-/tree/master/plug-ins/file-psd) | `psd-load.c`, `psd-layer-res-load.c` and `psd-export.c` comments hold decades of quirks. |
| [ICC profile spec](https://www.color.org/specification/ICC.1-2022-05.pdf) | Only for what resource 1039 carries. |

`doc/lab/photoshop-output.md` records what Photoshop actually writes. When the spec and that
file disagree, the lab file is right.
