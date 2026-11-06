# M2 completion checklist: finish preflight before M3

Updated: 2026-11-06 (Asia/Calcutta)

[Status](../PROJECT_STATUS.md) | [M2 plan](m2-plan.md) | [Coverage](coverage-matrix.md) | [Acceptance](acceptance-plan.md) | [Queued M3 plan](m3-plan.md)

The [delivery queue](delivery-plan.md) maps remaining work to concrete evidence,
available agent actions and machine-dependent checks. It preserves this gate.

**M2 is not complete.** Candidate-only collectors, schema v2 reports and fixture
checks exist. The [artifact pilot](artifact-identity.md) and [boot adapter](boot-identity.md) cover narrow implementations; they do not complete the required artifact matrix, reboot validation, or Windows
10/11 real-tool acceptance. The maintainer requested closing these gaps before
M3 implementation on 2026-11-01. No milestone gate is waived by this document.

## Available environment and limits

The maintainer confirmed that only the current Windows 11 machine is available.
Its local registry reports release 25H2, build 26200.9711; this identifies a lab
candidate, not a tested-support claim. Record the token mode at each actual trial.
The registry ProductName alone must not determine the OS family.

No Windows 10 desktop or VM is attached. Windows Server CI is fixture evidence,
not a substitute. Windows 10 VM work is deferred by the maintainer as of
2026-11-02. Do not create a VM, install assistants system-wide or elevate automatically.
A reviewed four-artifact native set is compiled into the matcher; see
[ground truth](validation/native-artifacts.md). A synthetic process named codex.exe remains a fixture, not a real tool.

Separate [native](validation/native-matrix-2026-11-02.json) and
[npm metadata/dependency-image](validation/npm-inventory-2026-11-02.json) records
contain passing partial Windows 11 subcases. npm setup stayed in disposable
repository-local prefixes with lifecycle scripts disabled. Ordinary package
launchers/runners and full acceptance rows are not validated by those records.

## Ordered closure work

| Step | Deliverable | Evidence required before closing | Current state |
| --- | --- | --- | --- |
| 1. Freeze the lab scope and ground truth | Reviewed required versus exploratory variants, OS/build/token matrix, pinned artifact manifest | Exact official acquisition URL, version, architecture, digest/provenance, rule predicates and expected classification per artifact | Four exact native pins frozen; package and broader lab rows remain separate; Windows 10 deferred by maintainer |
| 2. Finish strong identity in the M2.4 boundary | Reviewed trusted artifact predicates and bounded read-only matching bound to the observed process instance | Positive known-artifact, benign same-name, changed/replaced artifact, stale-cache and exit/race results; no basename/package-only promotion | Four-tool native pilot implemented with SHA-256 and guarded image binding; remaining variants and broader failure/desktop evidence pending |
| 3. Settle boot-scoped correlation | Reviewed OS-backed boot identity approach, or an explicit maintainer decision about the outstanding requirement | Same-boot stability, restart distinction, denied/unavailable behavior and privacy checks; no clock/uptime estimates represented as trusted identity | Fixed adapter and private checkpoint implemented; same-boot comparison passes; actual power-transition and Windows 10 checks pending |
| 4. Execute the Windows 11 desktop subset | Complete trials on the actual declared build/mode with pinned tools | Required acceptance subcases, before/after no-mutation checks, redacted reports and ground-truth references | Native and real npm/headless snapshot subcases pass; full variant/privacy/failure matrix remains open |
| 5. Execute the Windows 10 desktop subset | Independent Windows 10 configuration and the same declared required subset | Its own exact build/mode/artifacts and required outcomes | Deferred by maintainer on 2026-11-02; not validated |
| 6. Review M2.6 and publish the result | Reviewed evidence, working commands, honest support matrix and known limitations | All required gates satisfied; focused PRs, relevant CI and completed CodeRabbit review; then update M2 status | Not ready; M3 implementation stays queued |

These steps complete existing M2 obligations; they are not a new product milestone
or permission to broaden scanning. Strong identity work must keep process/file
races explicit. Reading a file at an observed path does not alone prove that the
bytes are the image executing in that process.

## Required test scope

Use the existing [coverage matrix](coverage-matrix.md#5-coverage-rows-and-acceptance-mapping)
and [acceptance table](acceptance-plan.md#4-acceptance-cases). The installed-CLI
baseline is C01/C02/C04/C05/C07/C08/C10 in readable declared roots. Active-at-snapshot
C03/C06/C09/C13 needs separate process evidence. Confirm the required subset and
expand installation rows per tool/package/generation before claiming completion.
C11/C12/C14 remain exploratory until independently validated. FUT-01/FUT-02 are
M3 work; they must not be counted as M2 passes or block M2 for unimplemented M3 scope.
No exploratory case may silently replace a required baseline case.

The lab manifest must distinguish Codex CLI, Claude Code, OpenCode 1 and OpenCode
2 artifacts/entry points. Do not guess their current packaging or treat an npm
package record as evidence of active execution. Ordinary npm/npx, Node/Python,
Git, compilers, local inference and inactive installations remain negative cases.

## Recording evidence

1. Copy [the unexecuted lab template](validation/windows-lab-template.json) into
   ignored reports; retain the committed template as an unexecuted reference.
2. Record source revision and dirty state, OS/build/token, compiler target,
   collector/catalog versions, case ID, artifact ground truth, steps, expected
   and actual result, coverage gaps, and minimized evidence references.
3. Use PASS/FAIL/BLOCKED/NOT RUN/NOT APPLICABLE with a reason. Unsupported identity
   or inaccessible data cannot be converted into a positive detector result.
4. Keep real machine output, absolute paths and artifact details that require
   review private. Publish only reviewed minimized summaries, not raw inventories.
5. Summarize required cases independently for each OS/configuration. Do not turn
   the 33-row planning table into a completion percentage: rows expand by variant,
   some are exploratory, and FUT rows belong to later milestones.

The existing Windows harness checks fixtures and the public schema; its
`m2_exit_gate: INCOMPLETE` is deliberate. Even a complete fixture pass cannot
certify real-tool support. A final M2 review must consider the separate lab records
and identity requirements, not just change that constant to COMPLETE.

## Work that can happen now

Prepare/review the artifact manifest and identity predicates, investigate a
supported boot identity source, and run controlled Windows 11 fixtures. Before
real-tool trials, arrange pinned artifacts and a consenting disposable lab scope.
A Windows 10 machine or VM is still required to finish the agreed two-OS gate.
M3 can be planned publicly in parallel; no M3 collector implementation starts
under the current instruction until M2 has its completion evidence.

## Fixture evidence available on 2026-11-01

The full existing local suite passed with normal directory access: 80 tests and two doctests, zero failures, one intentionally ignored child helper. Format, strict Clippy and strict rustdoc passed. Source: 6568037; Rust 1.99.1 GNU; Windows 11 25H2 build 26200.9711, StandardUser fixture report. Restricted execution still denies ancestor access in the installation suite. No code/ACL/guard/script-policy change was used. This closes the local fixture-execution obstacle only; it does not close any real-tool, strong-identity, boot or Windows 10 requirement above. See [dated evidence](validation/results.md#local-fixture-rerun---2026-11-01).

The [boot adapter](boot-identity.md) is a partial closure of step 3, not completed reboot/restore validation. Known fixed-header behavior is empirically guarded and unsupported queries retain the gap. Artifact identity and real-tool desktop results remain separate requirements.

The [native artifact pilot](artifact-identity.md) partially implements step 2. It does not silently remove Claude/OpenCode/package variants from the required matrix. One suspended-image Codex trial is not completion of all Windows 11 cases, and no restart or Windows 10 result is fabricated.
