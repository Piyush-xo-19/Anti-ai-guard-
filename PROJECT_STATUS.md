# Project status and milestone plan

Updated: 2026-10-28

**Current phase: M1 design review. M2 implementation has not started.**

This is the public progress summary for contributors. No private maintainer notes or earlier chat history are needed to understand the project direction.

[README](README.md) | [Decision log](docs/decisions.md) | [Architecture](docs/architecture-v1.md) | [Coverage matrix](docs/coverage-matrix.md) | [Acceptance plan](docs/acceptance-plan.md) | [M2 plan](docs/m2-plan.md)

## What is complete

- [x] Publish the project introduction, MIT license and contribution workflow.
- [x] Record Policy A: installation alone is allowed; prohibited execution during verified competition is restricted.
- [x] Establish the Rust, Ring 3/user-mode direction and Windows 10/11 prototype targets.
- [x] Draft the v1 architecture, trust boundaries, evidence handling and failure behavior.
- [x] Create the editable architecture views and commit four Mermaid diagram sources.
- [x] Publish the M1 Windows coverage matrix and acceptance plan.
- [x] Define 33 planned acceptance scenarios covering detection, benign activity, privacy, failures and deferred work.
- [x] Publish a contributor-facing decision log and bounded M2 implementation plan.

These are documentation/design deliverables. There is no working scanner, runtime monitor, backend, installer or dashboard yet. None of the 33 scanner scenarios has been executed; there are no validated OS configurations or achieved performance results.

## Milestones

| Milestone | State | Deliverable / exit condition |
| --- | --- | --- |
| M0: public foundation | Complete | Project introduction, license, architecture entry points and contribution workflow published |
| M1: coverage and design | Documents prepared; review pending | Agree the prototype privilege/lab/identity scope using the published coverage matrix and acceptance plan |
| M2: read-only preflight | Planned; not started | Scoped Rust CLI, installation/process snapshots, diagnostic reports and documented Windows results; see the M2 plan |
| M3: runtime observation | Not started | Supported process launch events, reconciliation, continuity and health measurements |
| M4: sessions and audit | Not started | Policy transitions, bounded durable audit/outbox and recovery behavior |
| M5: organizer workflow | Proposed scope; not started | Enrollment, admission/leases, heartbeats, review and interfaces |
| M6: controlled validation | Not started | Measured platform/variant results, privacy checks and release documentation |

Linux remains a later v1 target. The first prototype is native Windows 10 and Windows 11; this does not promise every historical build, architecture or installation variant is supported.

## What M1 still needs

- [ ] Review the proposed standard-user baseline and treatment of access-denied metadata.
- [ ] Review the proposed x64 lab scope and exact Windows 10/11 test releases/builds.
- [ ] Agree the evidence predicates for candidate versus supported identity findings.
- [ ] Review required versus exploratory tool-installation variants and M2 exit gates.
- [ ] Record the review outcome and select one initial M2 chunk.

Backend lease deadlines, retention policy and IDE feature controls remain open at their relevant milestones. M2 must not silently implement or claim those capabilities.

## What M2 will deliver

An on-demand, read-only diagnostic scanner that inventories supported assistant installations and processes alive during the scan. It will report scope, evidence and coverage limitations clearly. It will not issue verified competition status or automatically stop applications.

The [M2 plan](docs/m2-plan.md) splits this into project setup, shared data/policy contracts, installation inventory, process snapshots, report integration and validation. Those chunks will be reviewed and committed separately.

## Evidence and publication history

| Commit | Published work |
| --- | --- |
| [7e58709](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/7e58709971c21f7d023d321061d8c3e1a4ef882b) | Project introduction and roadmap |
| [6b51d2b](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/6b51d2bd614c0497b70a657a1a455a551e618f15) | MIT license and contribution workflow |
| [77be000](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/77be000560c82e83836e53e16a1f6de916eb8a0f) | Expanded v1 architecture and diagram sources |
| [a58d5c1](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/a58d5c1154a2271931b067680f0c81f50a38c319) | Windows 10/11 coverage contract and acceptance plan |

Published documentation has been checked for local links, whitespace and scope consistency. Architecture views were visually reviewed during composition. These checks are not evidence of product functionality.

## How this page stays current

Update this public summary when a milestone changes, a decision is accepted or a chunk is completed. Record design decisions in the decision log and link implementation evidence through commits or pull requests. Mark tests as run only when results exist, including OS/build and known gaps.

The maintainer may keep a separate ignored local tracker for detailed working notes. It is optional for contributors and must not be the only record of project decisions, milestone plans or publicly relevant progress.
