# M2.2: observations, coverage and diagnostic Policy A

Implemented: 2026-10-29 | In-memory library contract, schema version 1

[M2 plan](m2-plan.md) | [Development](development.md) | [Public status](../PROJECT_STATUS.md)

M2.2 supplies platform-independent Rust types and a pure evaluator. These M2.2 modules do not inspect a device. The CLI now separately offers [M2.3 consented installation inventory](installation-inventory.md). M2.4/M2.5 now provide current-session candidate collection and minimized text/JSON output. Real strong-identity matching remains incomplete.

## Code and responsibilities

| Source | Responsibility |
| --- | --- |
| [model.rs](../src/model.rs) | Platform/mode metadata, declared scope, collector coverage, process instances, identity assessments and snapshot inputs |
| [policy.rs](../src/policy.rs) | Diagnostic suggestions, coverage concerns and an in-memory report retaining its input evidence |
| [contracts.rs](../tests/contracts.rs) | Synthetic contract scenarios, including uncertain identity, missing coverage and PID reuse |
| [diagnostic.rs](../examples/diagnostic.rs) | Runnable hard-coded demonstration, without inspecting the host |

The M2.2 types/evaluator use only the standard library; M2.3 separately adds serde_json for bounded manifests. The types use enums for states/reasons and opaque scope references. String metadata must be minimized by the caller: a scope label is a description such as `declared package root 1`, not a personal absolute path. No raw command-line, environment, prompt, history or document-content field exists. This is not a general-purpose string redactor; callers must not put private data into labels or identifiers.

## Observation and identity boundaries

`ObservationKind::Installation` records inventory. `ObservationKind::Process` records a process instance and whether it was alive at the snapshot or exited during enrichment. Sources distinguish package metadata, executable artifacts and process snapshots. Inconsistent source/kind combinations require review and make coverage incomplete.

`IdentityAssessment` carries an optional tool/category, an evidence class and an optional versioned `RuleMatch`. Inventory indicators, candidates, corroborated identity and confirmed supported identity remain distinct. Coding assistants, local inference runtimes and other tools have separate categories.

**These are preclassified inputs, not a detector or trust boundary.** The evaluator cannot prove a supplied assessment is true. No production catalog, executable hashes, publisher checks or approved real-tool identity predicates are supplied. Future collectors/matchers must establish those facts through the outstanding identity review. A filename, path or writable package name alone must remain a candidate.

The tests use `fixture.*` identifiers and a `fixture-only` catalog version to represent hypothetical upstream match results. They demonstrate evaluator behavior, not recognition of any actual assistant. Constructing a `RuleMatch` never authenticates an artifact or authorizes competition enforcement.

## Diagnostic actions

The report carries policy version `policy-a-diagnostic-v1`. There is no verified-session input, automatic process action or session mutation.

| Input | Suggestion |
| --- | --- |
| Installation, including a supported assistant identity | `InventoryOnly`; never proof of execution or inactivity elsewhere |
| Ordinary unidentified process with inventory-level evidence | `NoAction`; not a clean-device assertion |
| Recognized local inference runtime or other non-assistant category | `NoAction` under this coding-assistant policy |
| Candidate identity, unknown candidate tool, missing/unreviewed rule or incomplete rule reference | `RequestReview` |
| Coding assistant that exited during enrichment | `RecheckProcess`; do not request closure of a stale process |
| Coding assistant missing stable boot/start identity | `RequestReview`; disclose the correlation gap |
| Alive coding assistant with stable process identity, corroborated/confirmed assessment and a complete rule reference approved for diagnostic closure | `RequestClosureAndRescan` |
| Unrequested/undeclared observation scope or inconsistent source | `RequestReview`; coverage remains incomplete |

Even the closure suggestion is a request, not execution prevention or a violation/penalty decision. Missing coverage is independent: a supported active finding can coexist with an incomplete scan. Installed inventory never upgrades a separate candidate process into supported execution.

The lower-level `suggest_action` function assumes the observation is already scoped. Use `evaluate(Snapshot)` to check declared/requested scope, aggregate coverage and retain the original observations alongside suggestions.

## Process-instance correlation

`ProcessInstance` carries a boot identifier, PID and optional process-creation value. The creation value is not the observation timestamp. Callers must use consistent boot/start identity domains.

- Matching PID, nonblank boot ID and start value establish `SameInstance`.
- A conflicting PID or known boot/start value establishes `DifferentInstance`.
- Missing/blank boot identity or missing start value leaves matching PIDs `Indeterminate`.

This helper neither enumerates processes nor validates an executable-to-process association or artifact cache. Real collection/race handling belongs to M2.4. Missing identity and a process disappearing during enrichment are distinct report concerns.

## Coverage and report semantics

`ScanExecution` records finished, partial, failed or cancelled independently of per-collector coverage. Each requested collector/scope pair needs exactly one valid declaration and coverage record. Coverage records preserve inspected, partial, unavailable, not-requested and outside-milestone states plus typed gap reasons.

`evaluate` returns:

- `InspectedRequestedScope` only when the scan finished and every requested pair has an inspected record without gaps or structural concerns. This covers only the declared scopes and says nothing about unrequested locations or time between snapshots.
- `Incomplete` for missing/duplicate records or requests, invalid scopes, reported gaps, unfinished scans, unscoped observations or process identity/enrichment gaps. `coverage_concerns` identifies the problem and affected target; collector-specific reasons remain in the retained snapshot.
- `NotRequested` when no inspection was requested and no contradictory observations or unfinished execution were supplied.

Unrequested runtime events may be recorded as `OutsideMilestone` without being counted as a successfully inspected capability. Requesting them cannot produce successful M2 coverage.

The report contains the supplied scan ID/time/platform/scopes, observations and collector records, schema/policy versions, ordered suggestions and coverage concerns. **Eligibility has one possible value: `NotIssued`.** No value grants, suspends or revokes competition verification. A future untrusted report-ingestion API will need separate validation/authentication; this in-memory library is not that API.

## Run the synthetic demonstration

After the documented [toolchain setup](development.md):

```powershell
cargo run --locked --offline --example diagnostic
cargo test --locked --offline --test contracts
```

Expected demonstration output:

```text
Synthetic demonstration only; no device inspection.
Observation 1: InventoryOnly (InstallationIsNotExecution)
Observation 2: RequestReview (IdentityNeedsReview)
Coverage: Incomplete
Eligibility: NotIssued
```

The demonstration uses hard-coded fixture observations, unknown platform metadata and a simulated access-denied gap. It does not report anything about the machine running it. This example is a developer aid; the [M2.5 report integration](diagnostic-reports.md) is implemented separately.

## Verification and remaining work

Nineteen synthetic contract tests and seven existing CLI integration tests passed locally with Rust 1.99.1 on the existing Windows GNU toolchain. Formatting, strict Clippy, build and the synthetic example also passed. [Hosted Windows CI for the feature commit passed](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40757983515), including formatting, lint, build, tests and the example. Check subsequent commits independently.

None of the 33 real scanner acceptance scenarios has run. Full Windows 10/11 desktop/real-tool acceptance, trusted identity artifacts, privilege coverage and performance remain unvalidated. M2.3/M2.4/M2.5 now supply consented candidate inventory/snapshots and reports; see [validation evidence](validation/README.md). M3 has not started.

## Input validation and concern identity

Boot IDs are opaque and case-sensitive after trimming surrounding Unicode whitespace; internal whitespace remains significant. Both stable-identity checks and correlation use that normalization. Blank boot IDs remain missing.

A blank/whitespace scan ID produces InvalidScanId and incomplete coverage. Internal nonblank fixture IDs remain valid; the public report projection still applies its stricter generated-ID syntax. Timestamp zero is the valid Unix epoch and is allowed for synthetic inputs; it never establishes current event freshness or eligibility.

Coverage concerns are unique by (optional collector/scope target, reason), preserving first occurrence. Duplicate requested targets are checked once and yield one DuplicateRequest plus any other distinct reasons. Requested RuntimeEvents on a current-session scope yields CollectorOutsideMilestone, not a false InvalidScope; unknown/duplicate/wrong-kind scope declarations remain invalid independently. Unrequested records claiming Inspected, Partial or Unavailable produce UnrequestedRecord. NotRequested/OutsideMilestone capability records remain passive information, even for an undeclared target, and do not claim inspection or satisfy a requested collector.

## Exhaustive decision-table checks

The observation test enumerates 8568 finite equivalence classes: 3 installation/process-state choices x 3 sources x 7 optional category/name choices x 4 evidence classes x 17 optional rule approval/completeness patterns x 2 identity strengths. Blank/nonblank strings represent their trim-equivalence classes; separate regressions cover whitespace normalization. Rule/catalog values are synthetic fixture inputs, not production trust entries.

Closure is permitted if and only if the observation is a live process from ProcessSnapshot, the category is CodingAssistant with a nonblank ID, identity is stable, evidence is Corroborated or ConfirmedSupported, and a complete rule reference has diagnostic-closure approval. Installation never becomes execution; outside-scope observations never request closure; eligibility is always NotIssued. The test checks both in-scope and outside-scope evaluation for every case.

A separate 2880-case coverage table varies collectors, request presence/duplication, scope validity, execution status, record state/duplication and gaps. Every concern forces incomplete coverage and no identical target/reason pair repeats. Both tables run offline; the 8568-case table took 47 ms in the observed local debug run, which is test evidence only, not an endpoint performance claim. Runnable rustdoc examples show snapshot construction and all three process-correlation results.
