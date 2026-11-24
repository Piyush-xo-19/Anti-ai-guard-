# M3.4 synthetic baseline reconciliation run

- Date: 2026-11-24 (Asia/Calcutta)
- Clean source: `2ef688704b18ccd4cae61d4d53e6528b6cc2558d`
- Host: Windows 11 registry release 26H2, build 26300.9974, x64
- Toolchain: Rust 1.99.1, `x86_64-pc-windows-gnu`
- Procedure: [M3.4 cases](../m3-4.md)
- Inputs: synthetic instances, source signals and elapsed times

| Check | Outcome | Evidence |
| --- | --- | --- |
| M3.4 reconciliation | PASS | Eleven cases passed for readiness, launch/exit timing, stop-first delivery, duplicate receipts, PID reuse, incomplete identity, queue/record/metadata bounds, stream isolation, stop and uncertainty. |
| Full default Rust suite | PASS | 131 tests and doctests passed, five intentionally ignored, on clean source above. |
| Format, strict Clippy, locked build, strict rustdoc | PASS | Checks completed on the reconciliation sources. |
| Public documentation links | PASS | 136 relative targets resolved in the seven changed Markdown guides before this record was added. |
| Windows event-source delivery and live baseline integration | NOT RUN | VM and desktop source trials remain deferred. |
| M2 desktop acceptance | NOT RUN in this change | Outstanding cases stay in the [M2 ledger](../m2.md). |

The buffer and reconciler run from injected data. The records retain observed
baseline/start/stop evidence. The tests establish neither current process
liveness nor live source completeness. M3.3 adapter selection and integration
remain pending.
