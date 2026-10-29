# M2 plan: read-only Windows preflight prototype

Updated: 2026-10-29 | State: in progress; M2.3 bounded inventory implemented; hosted Windows fixture checks passed

[Public status](../PROJECT_STATUS.md) | [Decisions](decisions.md) | [Coverage contract](coverage-matrix.md) | [Acceptance plan](acceptance-plan.md)

## Outcome

Build an on-demand Rust CLI for native Windows 10 and Windows 11 that inventories supported installations and observes recognized processes alive during a scan. Produce a clear diagnostic report with scope, evidence classes and explicit coverage failures.

M2 does not grant organizer verification. It cannot establish that nothing ran between snapshots. Proposed standard-user inspection, initial x64/lab releases and identity predicates remain subject to M1 review.

## Entry gate

The maintainer authorized M2.1 on 2026-10-29. That approval covers project/CLI setup with no device inspection. Privilege/lab scope and tool-identity choices remain review gates before the chunks that collect or classify host observations.

The maintainer subsequently authorized M2.2 on the same date: shared contracts and synthetic Policy A evaluation. This does not approve a real identity catalog, collection privileges or automatic competition enforcement.

- [ ] Review the M1 coverage matrix and acceptance plan.
- [ ] Agree privilege and lab scope for the first implementation chunk.
- [ ] Agree which identity predicates and variants are required versus exploratory.
- [ ] Identify a trusted synthetic/pinned artifact approach and record test ground truth.
- [x] Authorize the first bounded M2 chunk: M2.1 foundation only.
- [x] Authorize M2.2 contracts and fixture evaluation without device inspection.
- [x] Authorize M2.3 bounded installation inventory on 2026-10-29: explicit roots, current token without elevation and candidate-only identity.

Help/version and argument errors now work; [development instructions](development.md) document their commands. M2.2 adds shared observation/report types and a pure evaluator; M2.3 now adds consent-gated installation collection. Process collection, serialization and full CLI report integration remain planned.

## Proposed chunks

### M2.1: Rust project and reproducible checks

State: implemented. Pinned Rust package, dependency-free parser/library, help/version CLI, seven binary integration tests and Windows CI configuration are present. Local GNU build/format/lint/tests and the [initial hosted Windows CI run](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40752729560) passed. Windows 10 desktop compatibility has not been validated. See the development document for exact evidence and the unresolved local rustup-manager crash.

Deliver the minimal CLI/library structure, build prerequisites, license metadata, formatting/lint/test commands and focused automated checks. Avoid empty placeholder services and unrelated UI/backend scaffolding.

Acceptance: a fresh contributor can follow the documented build steps; help/version and argument errors behave predictably; no scan occurs without the agreed consent/scope flow. Record the compiler/toolchain used and any untested target.

### M2.2: Shared observations, coverage and Policy A reporting

State: implemented. [Shared-contract guide](shared-contracts.md) documents platform/scope metadata, separate installation/process observations, rule/evidence assessments, process-instance correlation, coverage concerns and diagnostic suggestions. The in-memory report retains evidence and can only return `NotIssued` eligibility. No device collection, real identity matcher or trusted catalog is present.

Local format, strict Clippy, build, nineteen synthetic contract tests, seven existing CLI tests and the runnable fixture example passed using Rust 1.99.1 on the Windows GNU toolchain. [Hosted Windows CI for c3b9c1a also passed](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40757983515). Synthetic supported assessments test evaluator behavior; they do not validate actual tool detection or satisfy the 33 scanner scenarios.

Define platform/scope metadata, installation and process observations, identity evidence, coverage failures and diagnostic action suggestions. Keep these types independent of Windows collection where possible.

Acceptance: synthetic fixtures demonstrate inactive inventory versus observed execution, candidate versus supported identity, incomplete coverage, process-instance correlation and no issued competition eligibility. Do not fabricate trusted identity catalog entries to make positive tests pass.

### M2.3: Bounded installation inventory

State: implemented for the [documented fixed selectors](installation-inventory.md). Explicit bin/npm-prefix roots and opt-in PATH roots, mandatory consent, budgets, no-reparse guards, minimized output and candidate-only identity are present. Local format/lint/build and 39 unit/CLI/contract tests passed; seven filesystem fixtures remain locally blocked by sandbox ancestor access. [Hosted Windows CI passed all 46 tests](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40925592116), including those fixtures. No pinned real-tool/desktop support is validated.

Inspect documented readable PATH/installation/package roots for Codex CLI, Claude Code and the separately identified OpenCode generations. Handle supported declared custom prefixes, scope budgets and access errors.

Acceptance: relevant INST/SCOPE/NEG cases in the acceptance plan pass for the implemented variants. The scanner must not execute discovered tools, install packages, run lifecycle scripts, inspect personal document contents or follow paths outside approved scope.

### M2.4: Process snapshot and identity correlation

Enumerate processes and enrich only permitted metadata. Correlate process instances with available executable/package evidence. Handle inaccessible metadata, exiting processes, PID reuse and stale artifact caches explicitly.

Acceptance: applicable RUN/ID/FAIL cases have outcomes supported by actual evidence. A process basename alone stays a candidate. Package installation does not force a Node-runtime assumption. No terminate, suspend, injection or memory-write operation is added.

### M2.5: Diagnostic report integration

Connect consent/scope selection, collectors and identity classification to readable and structured reports. Distinguish scan completion, coverage, evidence quality and suggested actions. Add documented example reports built from synthetic fixtures.

Acceptance: output paths agree and redact fake secrets in normal and error cases. Empty findings do not imply a clean entire device. Cancellation and partial failure remain visible. No complete command-line dump or unbounded report storage.

### M2.6: Windows integration evidence and milestone review

Run the implemented contract on the selected Windows 10 and Windows 11 lab configurations. Record exact OS builds, mode, scanner/catalog/tool versions, ground truth and results without publishing private inventories.

Acceptance: required cases pass for each claimed configuration; blocked/not-run cases remain visible. Report unsupported variants and measurements honestly. Update README with commands that now work, public status with evidence, and the local tracker with handoff details.

Chunks may need to split further when their diffs become difficult to review. Dependency order matters: fixtures/contracts precede collection, and integration claims follow real tests. Do not mark all of M2 complete because project setup or one collector works.

## Outside M2

- Continuous process launch events and short-lived-process recall.
- A monitoring service, watchdog or anti-tamper mechanism.
- Automatic competition suspension, backend admission or verification leases.
- Organizer dashboard, participant GUI or installer.
- USB policy enforcement or general model-store scanning.
- Browser/IDE-feature inspection, WSL/container coverage or Linux collectors.
- Automatic process termination, uninstall, file deletion or disqualification.

These exclusions do not remove the relevant later-v1 goals or the agreed participant safeguards.

## Completion evidence

Use the [acceptance plan](acceptance-plan.md) as the detailed gate. The milestone review must link:

- Implemented scope and working build/run instructions.
- Shared fixture tests and relevant formatting/lint results.
- Installation and process test outcomes per OS/build and inspection mode.
- Benign, weak-identity, privacy, error and no-mutation cases.
- Known visibility and installation-variant gaps.
- Report examples using synthetic data only.
- Focused commits or pull requests for each completed chunk.

The 33 planned scanner scenarios include later-milestone cases; M2 is accountable for its declared required subset, not a claim to have implemented FUT cases. Those scanner scenarios have not run. M2.1/M2.2 have seven passing local CLI tests and nineteen synthetic contract tests; they are separate from detector validation. Runtime recall/latency targets require later event-collector evaluation.

## Progress updates

After each reviewed chunk, update the public status with the outcome, checks and next step. Move entries from planned to complete only with evidence. Keep proposals in the decision log distinct from accepted choices and working behavior.
