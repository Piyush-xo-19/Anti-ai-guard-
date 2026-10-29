# Project status and milestone plan

Updated: 2026-10-30 (Asia/Calcutta)

**M2.4 process candidates and M2.5 integrated reports are implemented. M2.6 validation tooling is implemented; full desktop acceptance and full M2 completion remain INCOMPLETE.**

[README](README.md) | [Decisions](docs/decisions.md) | [M2 plan](docs/m2-plan.md) | [Validation evidence](docs/validation/README.md) | [Results](docs/validation/results.md)

## Implemented and checked

- M0: public repository, MIT license, contribution workflow and architecture entry points.
- M1: architecture, coverage matrix and 33-case acceptance plan prepared; lab/privilege/variant/strong-identity reviews remain open where relevant.
- M2.1: pinned Rust CLI foundation and reproducible checks.
- M2.2: separate installation/process observations, evidence classes, coverage and diagnostic Policy A.
- M2.3: explicit consent and bounded bin/npm-prefix/PATH inventory, no-reparse/sharing guards, minimized output, candidate-only identity.
- M2.4: current-session known native process-name candidates, limited metadata queries, PID/image races and weak same-location installation linkage. Missing boot identity stays a gap.
- M2.5: explicitly scoped combined scans, one bounded minimized text/JSON projection, cooperative cancellation and synthetic public examples. Eligibility is always NotIssued.
- M2.6 tooling: Windows fixture harness, sanitized artifacts and a template containing both desktop OS configurations and all 33 NOT RUN scenarios. Hosted server matrix verification follows publication.

No trusted strong identity catalog, continuous runtime monitor, backend, installer or dashboard exists. Installed inventory never proves execution. Native names/writable metadata remain candidates. There is no automatic termination, deletion, uninstall, disqualification or competition verification.

## Milestones

| Milestone | State | Remaining exit condition |
| --- | --- | --- |
| M0: public foundation | Complete | Maintain documentation |
| M1: coverage/design | Documents prepared; relevant reviews open | Review exact lab/privilege/variant and strong-identity requirements |
| M2: read-only preflight | Candidate-only implementation available; NOT complete | Trusted identity/boot requirements and required Windows 10/11 real-tool/privacy/failure lab outcomes |
| M3: runtime observation | Not started | Supported event source, reconciliation, continuity and health evidence |
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
| Local available checks | Windows 11 build 26200, X64 compiled target, StandardUser, Rust 1.99.1 GNU | Format/lint/build and 57 selected tests passed; one helper ignored |
| Local restrictions | [Validation review](docs/validation/README.md) | Seven installation filesystem cases and direct PowerShell harness invocation blocked; no policy bypass |

The ignored local tracker is optional maintainer context. All contributor-facing decisions, plans, commands and remaining gates are public. All 33 original pinned-tool desktop scenarios remain NOT RUN; synthetic fixtures do not replace them. No recall/false-positive/runtime-latency target has been achieved.

## Next work

Run independent diagnostics only with explicit consent and selected roots/session. Resolve trusted identity and boot correlation requirements, approve exact lab/variant scope, then execute the required desktop matrix before promoting support or completing M2. Preserve milestone order; M3 has not started.
