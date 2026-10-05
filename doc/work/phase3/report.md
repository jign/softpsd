# Phase 3 fixture checks

## Chunk 1: smoke harness

The shared fixture catalog, builder example and seven tests pass. The example lists the
catalog for the gate runner; the Photoshop expectation keeps the empty bottom layer,
PassThrough group, file's ICC resource and 72 ppi resolution. The smoke script moved to
`tools/photoshop/fixtures/` and its current documentation links were updated.

`tools/gate-fixtures.ps1 smoke`, the no-name invocation and `-Make smoke` all pass:
the writer gate reports zero differing merged pixels, and both reader gates match the
Photoshop trees with zero differing pixels in every layer. The existing Photoshop fixture
was restored byte for byte after verifying `-Make`; no new fixture was added.

The fixture round trip test rejects a changed writer depth byte, then passes after
restoration. An injected writer-gate exit 7 stops the harness before either reader gate
or the next fixture and reports `Fixture 'smoke' failed at 'writer gate': exit 7`.
Unknown names are rejected before building. Formatting, compilation and Clippy pass.
No spec disagreement or writer change was needed.
