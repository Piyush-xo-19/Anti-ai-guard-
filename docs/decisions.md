# Project decision log

Updated: 2026-10-29 | Status: accepted choices and open proposals

[Public status](../PROJECT_STATUS.md) | [Architecture](architecture-v1.md) | [Coverage contract](coverage-matrix.md) | [M2 plan](m2-plan.md)

This log makes project direction understandable without access to private notes. Accepted decisions reflect maintainer instructions or established project requirements. Proposed decisions are not silently treated as approved or implemented. Validation results are separate from design approval.

## Accepted decisions

| Decision | Agreed direction | Consequence |
| --- | --- | --- |
| ADR-001: v1 execution environment | Ring 3/user mode; no custom kernel driver in v1 | Use supported OS observations; disclose limits rather than promise kernel-level prevention |
| ADR-002: Policy A | Inactive installed coding assistants may remain; prohibited execution is restricted during verified events | Separate inventory, execution evidence, eligibility and organizer penalties |
| Endpoint language | Rust | Rust supplies bounded installation/process candidates and diagnostic reports; strong identity and desktop acceptance remain open |
| First prototype platforms | Native Windows 10 and Windows 11 | Start Windows work first; Linux remains a later v1 target |
| Open-source license | MIT | Public contributions follow the repository license |
| Delivery workflow | Small, reviewable chunks with relevant checks and descriptive commits | Do not build the full product in one unreviewable change |
| Milestone ordering | Review scope before the affected implementation chunk | M2.1 foundation and M2.2 synthetic shared contracts authorized on 2026-10-29; inspection-specific privilege/lab/identity decisions remain pending before collection |
| M2.3 implementation scope | Maintainer authorized bounded installation inventory on 2026-10-29 | Explicit bin/npm-prefix roots, opt-in PATH, current token without elevation, candidate-only metadata; no production strong-identity approval |
| M2.4/M2.5/M2.6 authorization | Maintainer requested these chunks in order on 2026-10-29, with separate commits | Current-session read-only native candidates, integrated diagnostic reports and available validation; no invented boot/strong identity or desktop test results |
| Participant safeguards | Explicit consent and scoped inspection; no automatic uninstall, deletion, termination or disqualification | Preserve normal device use and provide explainable findings/review |
| Privacy boundary | No keystrokes, terminal history, source-code or personal-document capture in v1 | Collect only relevant metadata and redact sensitive launch information |
| Public planning | Publish milestone status, decisions and implementation plans | Contributors must not depend on the maintainer's ignored tracker |
| Feature delivery | One feature, build, run, essential tests, then push and independent testing | Testers/contributors report reproducible issues; critical and dependency-blocking bugs are fixed before building on them |

M2.1 authorization covers CLI/project setup, and the subsequent M2.2 authorization covers shared types and diagnostic evaluation of synthetic preclassified inputs. Neither approves standard-user/elevated collection coverage, x64 test configurations, real strong-identity predicates or automatic competition enforcement. The subsequent M2.3 authorization covers the narrow installation scope documented in the [inventory guide](installation-inventory.md). It does not settle real-artifact identity, exact desktop validation or process privilege coverage. The milestone plan otherwise remains unchanged.

M2.2 implements the existing diagnostic-only boundary: separate inventory/execution/coverage, retain versioned rule references and require review for incomplete identity. Rule approval in the shared type means approval for a diagnostic closure suggestion only. It is not catalog authentication or authorization to suspend a competition session. See the [shared-contract guide](shared-contracts.md) for the implemented behavior and fixture boundary.

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

## Public report wire contract (accepted 2026-10-30)

The existing M2.5 `serde_json` dependency and minimized `PublicReport` projection
remain the serialization path. Shared snapshots and arbitrary upstream strings
are not serialized directly. [Schema v1](report-schema-v1.md) freezes field shapes,
nulls, enum spellings and compatibility rules. Breaking wire changes require a
schema-version bump; policy and indicator versions remain separate. A shared
synthetic fixture must match the committed JSON/text golden files on every CI OS.
This resolves issue #9 without expanding collection or enforcement scope.

## Typed public DTOs (accepted 2026-10-30, issue #15)

Use direct `serde` with its `derive` feature for private public-output DTOs only.
This enables the already locked serde/proc-macro packages; it adds no new package
versions. Explicit serde renames decouple wire enum names from Debug. Text renders
from the typed minimized DTO, and `value()` remains a compatibility view built
from that DTO. Shared raw snapshot types do not derive serialization. The v1
JSON/text golden files must remain byte-identical in this refactor.
