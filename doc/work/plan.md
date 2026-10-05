# Plan

Each phase ends on a gate, not on code. No phase starts before the previous gate is green.
A finding that changes a spec corrects the spec in the same commit.

| Phase | Work | Gate |
| --- | --- | --- |
| 1 Writer slice | model types, validation, writer end to end, `tools/photoshop/dump.jsx` | `softpsd-smoke.psd` opens in Photoshop with the same tree; its PNG export equals our merged image |
| 2 Reader | round trip on our output; then `ps27-smoke.psd` with Photoshop's extra blocks and mask block; bounds and refusals | both smoke files read to the same model |
| 3 Fixtures | the other eleven rows of `spec/gates.md`, one commit each | every fixture passes writer gate 1 and 2 and the reader gate |
| 4 Gate automation | PhotoshopAPI dump, `tools/fetch-corpus.ps1`, `--features corpus`, corpus triage, GIMP and Krita composites when installed, `tools/gate-all.ps1` | corpus reads or refuses, never panics; one command runs every gate |
| 5 Soft Edge export | in the Soft Edge repo: path dependency, tree to model mapping, Automasks setting, PSB switch, one-way notice; measure gamma blending against Photoshop | an 8K export opens in Photoshop and matches |
| 6 Publish | benches, README, notice down, `publish = true`, 0.1.0 | on crates.io |

Phases 1 to 4 are green. Phase 5 is next, in the Soft Edge repo.

Phase 5 is where the API gets its real review. Nothing is published before it.
