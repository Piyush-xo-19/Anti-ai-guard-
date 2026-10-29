# M2.6: validation evidence and remaining M2 gates

The maintainer authorized M2.6 with M2.4/M2.5, retaining separate commits. The validation tooling and available fixture checks are implemented. **Full M2.6 desktop acceptance, and therefore full M2 completion, remain incomplete.** A hosted Windows Server check is not Windows 10/11 desktop validation.

## Repeat the fixture checks

From a normal Windows terminal with the pinned Rust toolchain and fetched lockfile dependencies:

```powershell
cargo fetch --locked
powershell -NoProfile -File scripts/validate-windows.ps1 -Consent
```

PowerShell 5.1 or newer is supported. The script accepts no inspection scope beyond its controlled tests and the explicitly consented current-session process metadata probe. It runs format, strict Clippy, build, tests, the synthetic examples and public-example agreement. It captures compiler/test output transiently, but its JSON artifact contains only command statuses, aggregate test counts, timings, source revision/dirty state, toolchain and minimized platform metadata. Raw output, fixture paths, process findings, PIDs and arguments are not uploaded.

Artifacts go under ignored `reports/validation/`; `-OutputPath` must remain below this repository's reports directory, without report-directory reparse points. Verification commands have a three-minute timeout; cleanup targets only a harness-owned verification process tree, never a participant process observed by the collector. The scanner itself has no process-control API.

The script returns 0 only when its **fixture suite** passes, and 1 if a check/setup fails. `m2_exit_gate` deliberately remains `INCOMPLETE` and real-tool desktop cases remain `NOT RUN` even when fixtures pass. On this restricted local session, seven installation filesystem fixtures are blocked by ancestor-directory access. The PowerShell harness itself is also blocked by the machine script-execution policy; no bypass flag or policy change was applied. The separate ignored local restriction record records those blocks instead of converting them into passes.

CI runs the same harness on `windows-2022` and `windows-2025`, retaining sanitized fixture artifacts. These are server environments. Current tests include a controlled self-built process named `codex.exe`, fake argument privacy, separate installation/process policy semantics, PID/image races, cancellation, reparse/sharing boundaries and text/JSON consistency. No real coding assistant is launched in fixture tests.

## Desktop lab record

[windows-lab-template.json](windows-lab-template.json) lists all 33 original acceptance IDs with `NOT RUN`, plus separate Windows 10/11 configurations. Copy the template into ignored reports and record exact edition/release/build/architecture, actual token mode, source revision/dirty state, collector/rule/catalog versions and per-tool acquisition/version/trusted-artifact ground truth. Keep trial outputs private and minimized. A template or fixture pass is never a completed lab result.

Follow [acceptance-plan.md](../acceptance-plan.md) and [coverage-matrix.md](../coverage-matrix.md). Resolve which rows/variants are required before promoting a configuration. FUT rows belong to later milestones; exploratory variants must not silently become guaranteed coverage. Record blocked/failed/not-run trials explicitly, rather than dropping them from recall calculations.

## Why the milestone stays open

- No Windows 10 desktop lab was available in this session.
- The local Windows 11 build 26200 standard-user environment exercised process fixtures, but its restricted ancestor-directory access blocks the complete installation fixture suite; no full pinned-tool desktop acceptance was run.
- No reviewed trusted strong-identity catalog/artifact predicate is supplied. Real processes remain candidates; renamed/custom/interpreter-hosted assistants are not validated.
- Trusted boot identity is absent, so cross-scan process correlation remains incomplete.
- Exact lab/privilege/variant requirements still need review before any support claim.

See [results.md](results.md) for exact available checks and commits. No 95% recall, 2% false-positive or five-second runtime target is claimed. M3 continuous observation has not started. Do not mark M2 complete until the declared baseline's real-tool/OS/privacy/failure gates have recorded outcomes and the unresolved identity/coverage requirements are settled.
