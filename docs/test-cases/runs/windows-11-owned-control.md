# Windows 11 owned-process validation

Development run: 2026-11-14 (Asia/Calcutta), Windows 11 25H2 build
26200.9974, standard user, Rust 1.99.1 Windows GNU x64. Base revision
`827c982a057837110a946488dd99fa742b0e1eab` with the test changes in this
delivery applied.

## NEG-02: separate session coverage from the owned runner result

Temporary test-only diagnostics identified the denied operation: the scanner
could query candidate process metadata but was denied a guarded ancestor open
at the Windows package directory while trying to corroborate an executable.
Only the operation and directory category were recorded. The diagnostics were
removed after confirming the cause. The
[Microsoft packaging documentation](https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes)
describes the protected package store. No permissions or production collection
rules were changed.

- **Full-session trial: BLOCKED.** Partial coverage with `AccessDenied`
  still prevents a verdict for the entire session.
- **Owned runner trial: PASS.** Follow the
  [launcher protocol](../../validation/launcher-lab.md) with
  `GUARD_LAUNCHER_LAB_CASE=benign-node-owned`. Expected: all job members
  are present in the snapshot, membership is unchanged, both interpreter
  processes remain alive, and complete coverage of those members yields no
  assistant finding. Actual: Finished/Inspected, no gaps, zero findings and
  clean launcher completion. This separate experiment cannot clear the
  full-session gap or establish a clean device.

## PRIV-02 / SAFE-01: candidate output and process state

Run `cargo test --locked --offline --test processes controlled_candidate_is_observed_without_retaining_paths_or_arguments -- --exact`.
A self-built candidate runs with a synthetic secret in its arguments. Both CLI
formats must suppress that secret and fixture path; the JSON must still include
the owned candidate with RequestReview/NotIssued. The child must stay alive
through both scans. **PASS** for this fixture; no system-wide log audit was run.

## PRIV-03: cooperative cancellation between candidates

Run `cargo test --locked --offline --lib cancellation_during_enrichment_stops_before_the_next_candidate`.
The deterministic provider requests cancellation during the first candidate's
enrichment. Expect Cancelled/Partial and no second candidate query. This tests
the collector cancellation boundary without relying on a timing race; a real
console Ctrl+C and an OS call that blocks are separate cases. **PASS** for the
injected cancellation case. The default suite passed 112 tests/doctests with
five intentional ignores; strict Clippy and rustdoc passed.

## Remaining physical and desktop checks

An availability check found standby available but hibernation disabled, so
Fast Startup is currently unavailable. The controlled resume trial requires
an operator power transition. Restore requires a disposable restore lab; the
active development desktop was not restored. These cases are pending, as are
the remaining desktop audit and benign-tool variants. Windows 10 is deferred.
