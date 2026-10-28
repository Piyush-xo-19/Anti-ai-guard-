# Project decision log

Updated: 2026-10-29 | Status: accepted choices and open proposals

[Public status](../PROJECT_STATUS.md) | [Architecture](architecture-v1.md) | [Coverage contract](coverage-matrix.md) | [M2 plan](m2-plan.md)

This log makes project direction understandable without access to private notes. Accepted decisions reflect maintainer instructions or established project requirements. Proposed decisions are not silently treated as approved or implemented. Validation results are separate from design approval.

## Accepted decisions

| Decision | Agreed direction | Consequence |
| --- | --- | --- |
| ADR-001: v1 execution environment | Ring 3/user mode; no custom kernel driver in v1 | Use supported OS observations; disclose limits rather than promise kernel-level prevention |
| ADR-002: Policy A | Inactive installed coding assistants may remain; prohibited execution is restricted during verified events | Separate inventory, execution evidence, eligibility and organizer penalties |
| Endpoint language | Rust | M2.1 now supplies the Rust project and CLI build instructions; collectors remain planned |
| First prototype platforms | Native Windows 10 and Windows 11 | Start Windows work first; Linux remains a later v1 target |
| Open-source license | MIT | Public contributions follow the repository license |
| Delivery workflow | Small, reviewable chunks with relevant checks and descriptive commits | Do not build the full product in one unreviewable change |
| Milestone ordering | Review scope before the affected implementation chunk | M2.1 foundation authorized on 2026-10-29; inspection-specific privilege/lab/identity decisions remain pending before collection |
| Participant safeguards | Explicit consent and scoped inspection; no automatic uninstall, deletion, termination or disqualification | Preserve normal device use and provide explainable findings/review |
| Privacy boundary | No keystrokes, terminal history, source-code or personal-document capture in v1 | Collect only relevant metadata and redact sensitive launch information |
| Public planning | Publish milestone status, decisions and implementation plans | Contributors must not depend on the maintainer's ignored tracker |
| Feature delivery | One feature, build, run, essential tests, then push and independent testing | Testers/contributors report reproducible issues; critical and dependency-blocking bugs are fixed before building on them |

M2.1 authorization covers only CLI/project setup. It does not approve standard-user/elevated collection coverage, x64 test configurations, strong identity predicates or any automatic competition enforcement. The milestone plan otherwise remains unchanged.

Administrator elevation is not a switch to Ring 0. Any later elevation proposal still needs a narrow purpose, explicit capability reporting and review.

## Open architecture decisions

These IDs correspond to the architecture document. A resolved part of a decision does not imply all related details are settled.

| ID | Area | Current decision / proposal | What remains |
| --- | --- | --- | --- |
| V1-D01 | Platform scope | Windows 10 and Windows 11 first: accepted | Proposed x64/lab releases need review; supported builds require actual results; Linux configurations later |
| V1-D02 | Privileges | Standard-user M2 baseline proposed; elevated diagnostics optional to investigate | Review the scope and metadata gaps before M2; investigate event-source permissions in M3 |
| V1-D03 | Organizer authority | Backend verification leases proposed; standalone scanner diagnostic only | Agree first usable event-mode scope before organizer implementation |
| V1-D04 | Offline operation | Connected admission and bounded reconnection proposed | Define lease deadlines and offline policy before session enforcement |
| V1-D05 | IDE restriction | Whole-app versus feature restriction unresolved | Do not claim assistant-feature enforcement from an IDE process name |
| V1-D06 | Runtime/storage approvals | Contextual declarations proposed | Define approval fields, reviewer authority and unapproved-device policy |
| V1-D07 | Evidence lifecycle | Minimal redacted evidence and bounded storage required | Choose retention, access, appeal preservation and queue limits |
| V1-D08 | Packaging/stack | Rust endpoint agreed; other technology choices remain proposals | Choose dependencies for each chunk; backend/UI framework and agent packaging later |
| V1-D09 | Signature governance | Versioned identity rules proposed | Review trusted artifacts/predicates before positive identity claims; signing/revocation ownership before distribution |

## M1-to-M2 review gate

Review these documents together:

1. [Coverage matrix](coverage-matrix.md): OS/mode scope, tool variants, evidence and gaps.
2. [Acceptance plan](acceptance-plan.md): positive/negative cases, privacy, failure behavior and required results.
3. [M2 plan](m2-plan.md): bounded implementation chunks and exclusions.

Approval should name the privilege baseline, lab scope, identity expectations and first implementation chunk. No test outcome, continuous monitoring guarantee or complete Windows support claim follows merely from approval.

## Recording future changes

For a material choice, record its status, date, concrete decision, rationale, consequences and evidence or review reference. For changes to a trust boundary, privileges, privacy scope or event policy, explain the impact before implementation. Keep earlier decisions discoverable when superseding them.

Update the architecture/coverage document and public status in the same focused documentation change when a decision affects them. Do not copy private handoff notes or personal machine inventories into this log.
