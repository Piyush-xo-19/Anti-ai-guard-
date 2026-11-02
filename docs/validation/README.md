# M2.6: validation evidence and remaining M2 gates

The maintainer authorized M2.6 with M2.4/M2.5, retaining separate commits. The validation tooling and available fixture checks are implemented. **Full M2.6 desktop acceptance, and therefore full M2 completion, remain incomplete.** A hosted Windows Server check is not Windows 10/11 desktop validation.

## Repeat the fixture checks

From a normal Windows terminal with the pinned Rust toolchain and fetched lockfile dependencies:

```powershell
cargo fetch --locked
powershell -NoProfile -File scripts/validate-windows.ps1 -Consent
```

The script declares PowerShell 5.1 as its minimum and uses compatible APIs; hosted runs validate it on PowerShell 7. The local 5.1 invocation is policy-blocked, so that runtime has not been validated. The script accepts no inspection scope beyond its controlled tests and the explicitly consented current-session process metadata probe. It runs format, strict Clippy, build, tests, the synthetic examples and public-example agreement. It captures compiler/test output transiently, but its JSON artifact contains only command statuses, aggregate test counts, timings, source revision/dirty state, toolchain and minimized platform metadata. Raw output, fixture paths, process findings, PIDs and arguments are not uploaded.

Artifacts go under ignored `reports/validation/`; `-OutputPath` must remain below this repository's reports directory, without report-directory reparse points. Verification commands have a three-minute timeout; cleanup targets only a harness-owned verification process tree, never a participant process observed by the collector. The scanner itself has no process-control API.

The script returns 0 only when its **fixture suite** passes, and 1 if a check/setup fails. `m2_exit_gate` deliberately remains `INCOMPLETE` and real-tool desktop cases remain `NOT RUN` even when fixtures pass. On this restricted local session, seven installation filesystem fixtures are blocked by ancestor-directory access. The PowerShell harness itself is also blocked by the machine script-execution policy; no bypass flag or policy change was applied. The separate ignored local restriction record records those blocks instead of converting them into passes.

CI runs the same harness on `windows-2022` and `windows-2025`, retaining sanitized fixture artifacts. These are server environments. Current tests include a controlled self-built process named `codex.exe`, fake argument privacy, separate installation/process policy semantics, PID/image races, cancellation, reparse/sharing boundaries and text/JSON consistency. No real coding assistant is launched in fixture tests.

## Desktop lab record

[windows-lab-template.json](windows-lab-template.json) lists all 33 original acceptance IDs with `NOT RUN`, plus separate Windows 10/11 configurations. Copy the template into ignored reports and record exact edition/release/build/architecture, actual token mode, source revision/dirty state, collector/rule/catalog versions and per-tool acquisition/version/trusted-artifact ground truth. Keep trial outputs private and minimized. A template or fixture pass is never a completed lab result.

Follow [acceptance-plan.md](../acceptance-plan.md) and [coverage-matrix.md](../coverage-matrix.md). Resolve which rows/variants are required before promoting a configuration. FUT rows belong to later milestones; exploratory variants must not silently become guaranteed coverage. Record blocked/failed/not-run trials explicitly, rather than dropping them from recall calculations.

## Why the milestone stays open

- No Windows 10 desktop lab was available in this session.
- The normal-access Windows 11 fixture suite passes; restricted sandbox access is a separate historical limitation. Controlled native-image subcases have results, but full desktop acceptance is incomplete.
- Four exact native artifact pins exist. Other versions and renamed/custom/interpreter-hosted assistants are not validated.
- Guarded device-local boot metadata exists, but reboot/Fast Startup/restore validation remains pending.
- Exact lab/privilege/variant requirements still need review before any support claim.

See [results.md](results.md) for exact available checks and commits. No 95% recall, 2% false-positive or five-second runtime target is claimed. M3 continuous observation has not started. Do not mark M2 complete until the declared baseline's real-tool/OS/privacy/failure gates have recorded outcomes and the unresolved identity/coverage requirements are settled.

## Current closure work - 2026-11-01

Follow the [M2 completion checklist](../m2-completion.md) before starting [M3 implementation](../m3-plan.md). Only Windows 11 is currently available. A normal-access local rerun passed the complete synthetic suite (80 tests and two doctests), including all seven filesystem fixtures. The same filesystem suite still fails under restricted ancestor-directory access; no guard or policy was bypassed. The direct PowerShell harness was not rerun. See the dated results entry for the distinction. Real-tool trials, trusted identity/boot requirements and Windows 10 desktop acceptance remain outstanding.

See the [partial native identity pilot](identity-pilot-2026-11-01.json) for the
first pinned Codex Windows11 subcases. It records its own source/scope and leaves
M2 incomplete. It does not replace the original unexecuted two-desktop template.

The [four-native record](native-matrix-2026-11-02.json) and
[actual npm metadata/dependency-image record](npm-inventory-2026-11-02.json)
add dated partial Windows11 subcases. Their clean source revisions, exact pins,
setup limitations and still-unexecuted scenarios are retained per record.

The fixture harness keeps its desktop gate INCOMPLETE and does not execute real
assistants. Its blocker summaries now describe incomplete artifact-variant and
boot-transition validation rather than incorrectly claiming that no catalog or
boot adapter exists. Runtime facts are read from the actual scanner report;
separate native/npm records are not silently converted into full row passes.
