# Project status and milestone plan

Updated: 2026-11-02 (Asia/Calcutta)

**M2.4 process candidates and M2.5 integrated reports are implemented. M2.6 validation tooling is implemented; full desktop acceptance and full M2 completion remain INCOMPLETE.**

[README](README.md) | [Decisions](docs/decisions.md) | [M2 plan](docs/m2-plan.md) | [M2 closure checklist](docs/m2-completion.md) | [M3 plan](docs/m3-plan.md) | [Validation evidence](docs/validation/README.md) | [Results](docs/validation/results.md)

## Implemented and checked

- M0: public repository, MIT license, contribution workflow and architecture entry points.
- M1: architecture, coverage matrix and 33-case acceptance plan prepared; lab/privilege/variant/strong-identity reviews remain open where relevant.
- M2.1: pinned Rust CLI foundation and reproducible checks.
- M2.2: separate installation/process observations, evidence classes, coverage and diagnostic Policy A.
- M2.3: explicit consent and bounded bin/npm-prefix/PATH inventory, no-reparse/sharing guards, minimized output, candidate-only identity.
- M2.4: current-session known native process-name candidates, limited metadata queries, PID/image races and weak same-location installation linkage. Missing boot identity stays a gap.
- M2.5: explicitly scoped combined scans, one bounded minimized text/JSON projection, cooperative cancellation and synthetic public examples. Eligibility is always NotIssued.
- M2.6 tooling: Windows fixture harness, sanitized artifacts and a template containing both desktop OS configurations and all 33 NOT RUN scenarios. Both hosted Windows Server jobs passed 64 tests each; see [permanent sanitized evidence](docs/validation/fixtures-2026-10-30.json).

Four reviewed native artifact pins and guarded device-local boot adapter now exist. Other variants remain candidates; no continuous runtime monitor, backend, installer or dashboard exists. Installed inventory never proves execution. Native names/writable metadata remain candidates. There is no automatic termination, deletion, uninstall, disqualification or competition verification.

## Milestones

| Milestone | State | Remaining exit condition |
| --- | --- | --- |
| M0: public foundation | Complete | Maintain documentation |
| M1: coverage/design | Documents prepared; relevant reviews open | Review exact lab/privilege/variant and strong-identity requirements |
| M2: read-only preflight | Bounded preflight and four native pins available; NOT complete | Trusted identity/boot requirements and required Windows 10/11 real-tool/privacy/failure lab outcomes |
| M3: runtime observation | Eight chunks planned; implementation not started | Supported event source, reconciliation, continuity and health evidence |
| M4: sessions/audit | Not started | Durable policy transitions, outbox and recovery |
| M5: organizer workflow | Proposed; not started | Enrollment, leases/heartbeats, review and interfaces |
| M6: controlled validation | Not started | Broader release/platform/performance evidence |

Linux remains a later v1 target. Windows 10/11 are prototype targets; hosted Windows Server jobs do not establish desktop support.

## Available evidence

| Chunk | Commit / check | Actual result |
| --- | --- | --- |
| M2.1 | [72d62a4](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/72d62a4416f0b860c5c016c7d7dc8cb958a19fdc), [CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40752729560) | Seven CLI tests and hosted checks passed |
| M2.2 | [c3b9c1a](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/c3b9c1ad5aa1c73ec33322dd1596072a00c1be40), [CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40757983515) | 26 tests and example passed |
| M2.3 | [faa7ba2](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/faa7ba252b47c5a36771efbed6fbe8da28dcc668), [CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40926226920) | 46 hosted tests passed; seven filesystem cases blocked locally |
| M2.4 | [9888b70](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/9888b70f4122794b0ee94c86b3ef2746d943494e), [CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40929011706) | 55 hosted tests passed, one controlled-child helper ignored |
| M2.5 | [87772a3](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/87772a3a176e77e08400175d10bb0a8a33b95de9), [CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40931370505) | 64 hosted tests passed, one helper ignored |
| M2.6 fixture automation | [5b07c93](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/5b07c934f24d9f81855fa7acdba5b4868972bf07), fixes through [f4270a2](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/f4270a2f28a64c278622f6c1638dc2cfa0ffa43a), [matrix CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40933806067) | Both server jobs PASS: 64 tests each plus harness checks; full M2 desktop gate INCOMPLETE |
| Local available checks | Windows 11 build 26200, X64 compiled target, StandardUser, Rust 1.99.1 GNU | Format/lint/build and 57 selected tests passed; one helper ignored |
| Historical local restrictions (2026-10-30) | [Validation review](docs/validation/README.md) | Restricted filesystem checks and direct harness invocation blocked then; see the 2026-11-01 fixture rerun below |

The ignored local tracker is optional maintainer context. All contributor-facing decisions, plans, commands and remaining gates are public. The desktop scenarios are not complete across variants; partial real-artifact subcases are recorded separately from synthetic fixtures. No recall/false-positive/runtime-latency target has been achieved.

## Next work

Run independent diagnostics only with explicit consent and selected roots/session. Resolve trusted identity and boot correlation requirements, approve exact lab/variant scope, then execute the required desktop matrix before promoting support or completing M2. Follow the [M2 completion checklist](docs/m2-completion.md). Only the current Windows 11 machine is available; Windows 10 requires a separate lab. Preserve milestone order; the [eight-chunk M3 plan](docs/m3-plan.md) is public planning only and M3 code has not started.

## Issue maintenance - 2026-10-30

Regression fixes for #1/#3/#4/#5/#6 validate metadata and active coverage claims, distinguish unsupported collectors, normalize boot-ID boundaries and emit each concern once per target/reason. All 24 contract tests passed locally; hosted verification and remaining issue work follow in focused commits. These changes do not complete the outstanding M2 desktop/strong-identity gates.

### CI and report contract follow-up

- Added shared-code Linux CI and strict rustdoc on every CI OS (#7); no Linux collector support claimed.
- Formalized schema v1 compatibility and retained the existing minimized serde_json boundary (#9).
- Added a shared synthetic example/golden test; arbitrary free-text minimization stays covered.
- Windows local checks: 65 selected tests plus two doctests pass; strict Clippy and rustdoc pass.
- [Maintenance CI for 8bae7e6](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40938599135) passed: 74 tests/doctests on each Windows Server runner (one controlled-child helper ignored), 65 on Ubuntu. Format, strict Clippy/rustdoc, build and synthetic examples passed. These results cover issues #1-#9; see [PR #19](https://github.com/Piyush-xo-19/Anti-ai-guard-/pull/19).
- M2 desktop/strong-identity exit gates remain incomplete.

## Issue follow-up #10-#18

Scope/help errors, root normalization/location matching, broken-pipe status,
cancellation documentation and exact-budget handling are corrected. The report
uses typed serde DTOs with explicit wire names. The refactor preserved v1 goldens;
[subsequent schema v2](docs/report-schema-v2.md) changes FILETIME to a decimal
string and adds `operational_gaps` without weakening incomplete coverage or
eligibility rules. Local: 73 selected tests and two doctests pass, with strict
Clippy/rustdoc/formatting. [Hosted v2 CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40940449420) passed on source 4a68d19: 82 tests/doctests on each Windows Server runner (one controlled-child helper ignored), 72 on Ubuntu. All format/lint/build/rustdoc/example and Windows harness checks passed. See [PR #20](https://github.com/Piyush-xo-19/Anti-ai-guard-/pull/20). M2 desktop gates remain open.

## M2 closure review - 2026-11-01

The maintainer requested M2 completion before M3 implementation and confirmed only the current Windows 11 desktop is available. [M2 completion work](docs/m2-completion.md) records six ordered closure steps; [M3 planning](docs/m3-plan.md) describes eight queued chunks, not implemented features.

The complete existing fixture suite now passes locally with normal directory access: 80 tests plus two doctests (82 total), zero failures and one intentionally ignored controlled-child helper. The seven filesystem cases still fail in the restricted sandbox but pass outside that restriction; no directory guard, ACL or script policy was changed. Format, strict Clippy and strict rustdoc pass. Host: Windows 11 25H2 build 26200.9711; native test output reported StandardUser. These are controlled synthetic fixtures, not pinned real-tool desktop acceptance. Strong identity, trusted boot correlation and the required real-tool/Windows 10 lab gates remain open. M2 is INCOMPLETE and M3 code has not started.

## M2 robustness follow-up - 2026-11-01

[PR #25](https://github.com/Piyush-xo-19/Anti-ai-guard-/pull/25) fixes empty PATH
segments without adding current-directory scope, preserves original invalid-entry
indices, and makes reported boot availability agree with normalized model identity.
Review follow-ups allow an empty PATH when explicit valid roots remain and reject
unexpanded Windows percent-variable expressions in PATH entries, as documented.
The schema-heading correction is merged in PR #24 (#23). No trusted boot source
or strong identity matcher is introduced by these consistency fixes.

Local full suite: 84 tests plus two doctests (86 total), zero failures, one
intentionally ignored controlled-child helper; format and strict Clippy pass.
Hosted checks and CodeRabbit for the updated PR must complete before merge.
M2 closure still requires the remaining identity/boot and two-desktop lab work.

## M2 boot adapter - 2026-11-01

A [fixed self-header boot adapter](docs/boot-identity.md) now supplies device-local OS boot metadata when its strict capability checks succeed. It requests no variable payload or foreign process data and retains MissingBootIdentity on failure. Windows 11 same-boot controlled checks pass; actual restart/restore and Windows 10 trials remain pending. Trusted artifact/image identity is the next separate chunk. M2 is still incomplete and M3 has not started.

## M2 pinned artifact pilot - 2026-11-01

A compiled [single-artifact catalog](catalog/windows-artifacts.json) and
[guarded native matcher](docs/artifact-identity.md) now support Codex CLI0.166.0
Windows x64 under strict capability/digest/file-binding/lifetime checks. Unpinned
variants remain candidates. The boot adapter merged in PR#26 after all seven
checks including CodeRabbit passed. One controlled suspended-image Windows11
real-artifact trial passed; broader real-tool, reboot/restore and Windows10 trials
remain pending. M2 and M3 status are unchanged: M2 incomplete, M3 not started.

## Four native pins and controlled Windows 11 evidence - 2026-11-02

The [frozen native scope](docs/validation/native-artifacts.md) now covers Codex,
Claude and both OpenCode generations. [Eight native subcases](docs/validation/native-matrix-2026-11-02.json)
passed (four inactive inventory, four owned suspended images). The default suite passed 98
tests/doctests; two helper/lab tests are intentionally ignored. No full
row/interactive/performance claim follows. Windows 10 VM work is deferred by the
maintainer; remaining M2 work continues and M3 is not started. Main protection
is active, independently verified through the GitHub ruleset API.

Four pinned npm metadata subcases and four directly observed package-native
dependency images also pass; see [package evidence](docs/validation/npm-inventory-2026-11-02.json).
Lifecycle scripts were disabled, so ordinary package-launcher/runner behavior
remains unvalidated. These results do not close full INST-02/RUN-02 rows.

## M2 scope and failure regressions - 2026-11-02

Windows reserved device names and illegal path components are rejected before
I/O. Invalid scope IDs cannot link to the first root. Totally unavailable
inventory reports Failed; mixed coverage reports Partial. At this stage, the
full local suite passed 101 tests/doctests, with two intentionally ignored tests.
Formatting, strict Clippy and strict rustdoc checks passed. The inventory guide
documents consent, privacy, exit codes and unchanged literal-percent handling.

The standalone Codex download names now have exact inventory selectors, matching
the existing process selectors (#34). A filesystem regression verifies candidate
installation evidence with zero content bytes read. An ARM64 filename does not
establish ARM64 scanner support. This follow-up passed 102 tests/doctests, with
zero failures and two intentionally ignored tests; formatting and strict Clippy
and rustdoc checks passed.

CodeRabbit identified cancellation arriving during the final failed root open.
The collector now reports Cancelled while retaining Unavailable and AccessDenied
details. A deterministic regression covers that race. The full local suite now
passes 103 tests/doctests, with zero failures and two intentionally ignored tests.
Formatting, strict Clippy and strict rustdoc checks passed. Review completion
remains mandatory before merge.

## Launcher and boot-transition validation tooling - 2026-11-03

An opt-in [npm launcher lab](docs/validation/launcher-lab.md) verifies frozen
entrypoints, isolated profiles, an owned process tree and the actual scanner
report. All four launcher trials passed during development; committed-source
evidence follows separately. This is a controlled headless version-query
snapshot, not interactive coding or daemon-mode acceptance.

A [private boot checkpoint](docs/validation/boot-transition-lab.md) can compare
the existing self-header provider before and after a deliberate power transition.
It never changes the machine power state. Actual restart/restore and Windows 10
results are still pending. M2 remains incomplete and M3 has not started.

[Committed-source launcher and checkpoint evidence](docs/validation/launcher-boot-2026-11-03.json)
now records four passing npm entrypoint trials, a same-boot comparison and three
expected checkpoint rejection cases. The default suite passes 106 tests/doctests,
with zero failures and five intentionally ignored labs/helpers. Actual machine
power transitions remain pending; no private baseline was published.

The [real denied-directory control](docs/validation/negative-controls-2026-11-03.json)
passes. The earlier Node/npm negative verdict was withdrawn; the
[completion-checked rerun](docs/validation/node-complete-2026-11-03.json) passes
with Finished execution, Inspected coverage and zero gaps. A
temporary denied empty directory reports explicit AccessDenied/incomplete
coverage. Its original ACL was restored, recovery was verified and the empty
fixture was removed. Existing user directory permissions were not changed.
