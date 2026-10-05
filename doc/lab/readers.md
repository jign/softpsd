# Readers we check against

| Library | Lang | Reads | Writes | Notes |
| --- | --- | --- | --- | --- |
| Photoshop 27.5 | app | yes | yes | the oracle; scripted, see photoshop-scripting.md |
| psd-tools | Python | yes | low-level | active; renders composites; exposes raw records |
| ag-psd | TypeScript | yes | yes | MIT; the most complete open writer; real-Photoshop fixtures |
| PhotoshopAPI | C++/Python | yes | yes | BSD-3; 8/16/32-bit; writes no merged image |
| GIMP | app | yes | yes | headless batch; decades of hardening |
| Krita | app | yes | yes | headless export |
| `psd` crate | Rust | yes | no | 0.3.5, Jan 2024 |
| @webtoon/psd | TypeScript | yes | no | stale |
| psd_sdk | C++ | yes | limited | reader-focused; not a gate |

Known disagreements, Photoshop is the tiebreak:

- ag-psd does not expose the ICC profile in resource 1039. psd-tools and Photoshop do.
- PhotoshopAPI output lacks the merged image; third-party readers then fail. We always write it.

`tools/readers/` dumps a file through each reader for comparison.

psd-tools applies its 30,000 px PSD side limit to version 2 files as well, so a 30,001 px PSB
fails to decode. `tools/readers/psd_limits.py` raises the limit to 300,000 for version 2 while
our gate tools decode.
