# M3.2 synthetic runtime contract run

- Date: 2026-11-16 (Asia/Calcutta)
- Clean fixture revision: `49c5ab652fffa6b6df83d93fb5d9b92a2fffa83a`
- Host: Windows 11 registry release 26H2, build 26300.9974, x64
- Toolchain: Rust 1.99.1, `x86_64-pc-windows-gnu`
- Setup: synthetic elapsed times, process instances, receipts and source signals
- Procedure: [M3.2 runtime cases](../m3-2.md)

| Check | Outcome | Evidence |
| --- | --- | --- |
| M3.2 runtime contracts | PASS | Nine integration tests passed on the clean revision. Readiness order, unknown continuity, delivery order, duplicate receipts, identity, stream isolation, bounds, loss, unavailability and stop were exercised. |
| Full Rust suite on the same Rust sources | PASS | 120 tests and doctests passed, five intentionally ignored. Format, strict Clippy, locked build and strict rustdoc passed. |
| Elevated ETW delivery on Windows 11 | NOT RUN | Deferred until an isolated lab is available. |
| Remaining M2 Windows desktop acceptance | NOT RUN in this change | Existing results and outstanding cases stay in the [M2 test index](../m2.md). |

The runtime fixtures use injected signals and never start a process-event source.
They verify the health rules, with live delivery and loss visibility still
pending for source selection.
