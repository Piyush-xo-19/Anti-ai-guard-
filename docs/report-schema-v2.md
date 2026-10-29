# Public diagnostic JSON schema v2

Status: accepted 2026-10-30 (issues #14 and #16), superseding schema v1. This extends
the M2.5 projection already implemented with `serde_json`; the typed serde DTOs preserve minimization.
The canonical example is [diagnostic.synthetic.json](examples/diagnostic.synthetic.json).
`report::project` is the serialization boundary. Raw shared models are not wire DTOs.

## Required structure

Every listed field is emitted, including nullable fields and empty arrays. Integers
are nonnegative JSON numbers; consumers must preserve 64-bit integer precision
when interpreting large numeric fields. FILETIME is a decimal string, so ordinary JSON parsers preserve it exactly. Object key
order and whitespace have no semantic meaning. Array order preserves report order;
finding indices and links refer to that order, not persistent identifiers.

| Object | Required fields and types |
| --- | --- |
| Root | `schema_version`: integer 2; `policy_version`, `scan_id`, `collection_execution`, `coverage`, `eligibility`, `indicator_catalog_version`: strings; `observed_at_ms`: u64; `report_truncated`, `operational_gaps`: booleans; `platform`, `metrics`: objects; `scopes`, `requested`, `collectors`, `coverage_concerns`, `findings`, `location_links`, `limitations`: arrays |
| platform | `os`, `architecture`, `architecture_basis`, `inspection_mode`: strings; `build`: string or null |
| scopes[] | `id`: u32; `kind`, `label`: strings |
| requested[] | `collector`: string; `scope_id`: u32 |
| collectors[] | `collector`, `state`: strings; `scope_id`: u32; `gaps`: string array |
| coverage_concerns[] | `scope_id`: u32 or null; `collector`: string or null; `reason`: string. Both target fields are null for global concerns. |
| findings[] | `index`: integer; `scope_id`: u32; `kind`, `source`, `tool_id`, `evidence_class`, `action`, `reason`: strings; `indicator_rule_id`: string or null; `upstream_rule_reference_omitted`: boolean; `process`: object or null |
| findings[].process | `pid`: u32; `creation_filetime`: decimal string or null; `state`: string; `boot_identity_available`: boolean |
| location_links[] | `process_finding`, `installation_finding`: indices; `basis`: string |
| metrics | `inventory_probes`, `metadata_bytes`, `process_entries`, `process_queries`, `collection_elapsed_ms`: nonnegative integers |
| limitations[] | string diagnostic limitation code |

`observed_at_ms` is Unix epoch milliseconds. Zero is allowed for synthetic data;
it does not assert freshness. Process `creation_filetime` is the Windows creation
time identity token (100 ns intervals from the Windows epoch), not Unix milliseconds.
Missing boot identity makes correlation incomplete. Installation findings have
`process: null`. Missing data uses null, never zero as an implicit missing value.

## Stable spelling and meanings

These strings are case-sensitive wire names. Refactoring Rust enum names must not
silently rename them; the published golden test guards the representative output.

| Field | Current values |
| --- | --- |
| collection_execution | `Finished`, `Partial`, `Failed`, `Cancelled` |
| coverage | `InspectedRequestedScope`, `Incomplete`, `NotRequested` |
| eligibility | `NotIssued` only |
| action | `InventoryOnly`, `NoAction`, `RequestReview`, `RequestClosureAndRescan`, `RecheckProcess` |
| finding kind | `Installation`, `Process` |
| source | `PackageMetadata`, `ExecutableArtifact`, `ProcessSnapshot` |
| evidence_class | `InventoryIndicator`, `Candidate`, `Corroborated`, `ConfirmedSupported` |
| process state | `AliveAtSnapshot`, `ExitedDuringEnrichment` |
| collector | `InstallationInventory`, `ProcessSnapshot`, `RuntimeEvents` (outside milestone) |
| collector state | `Inspected`, `Partial`, `Unavailable`, `NotRequested`, `OutsideMilestone` |
| scope kind | `InstallationRoot`, `CurrentSessionProcesses` |
| os | `Windows10`, `Windows11`, `WindowsServer`, `Linux`, `Unknown` |
| architecture | `X64`, `Arm64`, `X86`, `Unknown` |
| inspection_mode | `StandardUser`, `ElevatedDiagnostic`, `Unknown` |

Action and concern reasons, gaps and limitations use the exact case-sensitive
names in [policy.rs](../src/policy.rs), [model.rs](../src/model.rs) and the static
projection in [report.rs](../src/report.rs). They are extensible diagnostic
vocabularies. Current maintenance adds `InvalidScanId`, `CollectorOutsideMilestone`
and `UnrequestedRecord`. Identical target/reason concerns occur once; global
concerns have no target. Every concern makes coverage incomplete.

`RequestClosureAndRescan` is a suggestion, never process control or competition
enforcement. Candidate observations require review. `InspectedRequestedScope`
means only the requested selectors were inspected. No report proves a clean device
or issues eligibility. Location links use
`same-declared-location-only-not-artifact-proof` and cannot strengthen identity.

## Privacy and bounds

Only generated scope labels, allowlisted tool IDs (or `unknown`), numeric build
strings and validated generated scan IDs (or `redacted-scan-id`) are projected.
Raw paths, argv, environment, free-text labels, raw boot IDs and arbitrary upstream
rule strings are omitted. Static indicator IDs are not a trusted identity catalog.
The projection limits and exit codes are documented in the
[report guide](diagnostic-reports.md); truncation sets `report_truncated: true`,
forces incomplete coverage and exit 3. Consumers must not infer success from an
empty or truncated findings array.

## Compatibility policy

- Removing or renaming required fields, changing types/nullability, changing units,
  or changing existing action/eligibility meanings requires a new schema version.
  A new action or eligibility value also requires a version bump and review.
- New optional fields and new diagnostic reason/gap/limitation, OS or capability
  codes may be added within v2. Existing names retain their meanings. Consumers
  ignore unknown optional fields and display unknown diagnostic codes as unknown;
  they never upgrade coverage, identity or eligibility because a code is unfamiliar.
  Unknown versions/actions require review rather than automation.
- `policy_version` identifies decision rules, and `indicator_catalog_version`
  identifies candidate indicators. Neither substitutes for `schema_version`.
  Policy changes need a policy version and tests; wire changes follow this policy.
- Golden files are synthetic fixtures shared by the example and integration test.
  Intentional edits require a compatibility review, not automatic regeneration to
  make a failed test pass. The fake-secret test also protects output minimization.

Linux CI validates shared code and this contract, not Linux collector support.
Windows desktop support still requires the separate acceptance matrix.

## Migration from v1

The CLI emits v2 only. creation_filetime changed from number to a decimal string to preserve every FILETIME bit in standard JSON parsers; null still means missing. Parse it with an exact 64-bit integer or BigInt, never through a floating-point number. Consumers must check schema_version before parsing. The synthetic example now uses 133000000000000001. No legacy numeric identity field is emitted.

The new required operational_gaps boolean describes collection/validation health. False means no operational error was detected; it never proves identity, coverage, clean state or eligibility. MissingBootIdentity alone (and its derived missing-process-identity/incomplete/partial concerns) is an expected prototype limitation when PID and start identity exist. All other gaps, invalid/missing/duplicate coverage, invalid metadata, cancellation, failure and truncation set the flag true. Unknown reasons are operational by default. Candidate-only boot-limited scans still report Partial/Incomplete, require review and exit 3. Scripts must parse this flag together with findings, execution and coverage; exit codes retain their prior meanings.
