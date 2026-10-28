# M1: Windows preflight acceptance plan

Version: 0.1 | Updated: 2026-10-28 | Status: test design only; all execution results are pending

[Coverage contract](coverage-matrix.md) | [Architecture](architecture-v1.md) | [README](../README.md)

## 1. Purpose and test boundaries

Define reproducible checks for the M2 read-only preflight prototype on Windows 10 and Windows 11. The maintainer selected both OS families. Proposed initial lab rows are Windows 10 22H2 x64 and Windows 11 24H2 x64; add other Windows 11 releases as separate rows with exact build/patch information. No row is validated yet.

Run required cases separately on each lab OS as a standard user. Repeat permission-sensitive cases in a participant-approved elevated diagnostic session when available. Lack of an administrator test must be recorded as not run, not silently counted as a pass. ARM64, 32-bit agents, Linux, WSL guests and container coverage are deferred.

M2 observes a point-in-time snapshot. This plan does not claim launch-event recall, a five-second runtime guarantee, backend verification, automatic blocking or competition suspension.

## 2. Test record and result vocabulary

For each run retain a synthetic/redacted record with: test ID, date, OS edition/release/build/architecture, account/elevation mode, scanner build/commit, catalog version, tool family/version, installation method/prefix, trusted artifact identifier, steps, expected result, actual result, coverage limitations and report reference.

Result values: NOT RUN, PASS, FAIL, BLOCKED or NOT APPLICABLE with a reason. Capture expected incomplete coverage as part of the result; it is not evidence that the hidden cases were detected. Any required row that is NOT RUN, FAIL or BLOCKED prevents a validated-support claim for that configuration.

All cases in this document start as NOT RUN. Historical documentation checks are not scanner test results. Test reports and real machine inventories must stay in ignored reports/ or another approved private location.

## 3. Safe lab setup

1. Use consenting lab accounts/devices or disposable test machines. Do not inspect other participants' machines.
2. Record the two OS configurations independently. A build succeeding on one does not validate the other.
3. Pin representative tool artifacts from the official sources linked in the coverage contract. Record versions and trusted hashes before tests; never substitute an arbitrary filename as ground truth.
4. Install one generation/variant at a time where commands conflict, particularly OpenCode 1 versus 2. Record which version is active.
5. Keep fixture package files, fake secrets and test paths synthetic. Do not use production credentials or private source code.
6. Start controlled assistants manually only in the relevant execution tests; no paid model request or real coding task is required. The scanner itself must not launch them.
7. Capture process presence immediately around a snapshot test. A process that was not actually alive during collection is not a valid active-process recall trial.
8. Record before/after application and file state. Tests must establish that scanning does not terminate, uninstall, modify or delete observed software.

## 4. Acceptance cases

### Installation and inspection scope

| ID | Setup / action | Expected result |
| --- | --- | --- |
| INST-01 | Place each pinned native Codex, Claude and OpenCode generation in a documented readable installation root, inactive; run preflight | Inventory with correct tool/rule or candidate evidence; no execution finding or installation penalty |
| INST-02 | Install each documented npm package in a supported readable prefix, inactive; run preflight | Inventory from package metadata and executable correlation where available; disclose missing corroboration; do not execute package tooling/scripts |
| INST-03 | Declare a custom npm prefix containing a pinned fixture; repeat with a denied root | Read the declared allowed scope; denied root is explicit incomplete coverage; no fallback whole-disk scan |
| INST-04 | Place a portable known executable in an approved root and another outside scope | Inspect only approved scope; do not imply the unscanned location was clean |
| SCOPE-01 | Start without participant consent, then explicitly approve the published scope | No inspection before acceptance; scope is disclosed and included in report |
| SCOPE-02 | Encounter a symlink/reparse point to a location outside approved roots, excessive depth or file-count limit | Do not traverse beyond scope/budgets; report skipped or incomplete inspection |

INST-01/02 expand into separate subcases per tool and artifact; do not count the table row as passed after testing just one assistant.

### Active process identity

| ID | Setup / action | Expected result |
| --- | --- | --- |
| RUN-01 | Keep each pinned agent active while taking a snapshot | Observe its process instance; classify only to the level justified by the identity rule; suggest closure if supported prohibited identity is established |
| RUN-02 | Launch an assistant through a supported package runner; separately run ordinary npm/npx | Match supported artifact/entry-point evidence where observable; otherwise candidate/gap; no broad package-runner violation |
| RUN-03 | Keep a known noninteractive/background variant active at snapshot; hide/close the terminal UI if the variant permits | Same process identity logic; no dependence on the terminal text or window title |
| ID-01 | Use a matching name with insufficient identity corroboration | Candidate or unknown, never fabricated confirmed identity |
| ID-02 | Rename a pinned executable and scan while active | Match only if the implemented strong artifact rule can read and verify it; otherwise document this exploratory gap |
| ID-03 | Alter a fixture after its identity was cached, or replace the executable at a known path | Invalidate stale evidence; do not reuse a prior path hash as proof of the new artifact |
| ID-04 | Forge writable package metadata or use a benign executable named after an agent | Do not classify metadata/name alone as confirmed prohibited execution |
| ID-05 | Test OpenCode 1 and 2 separately with their corresponding package records | Use distinct generation rules; no assumption that opencode uniquely identifies one generation |
| ID-06 | Reuse a PID in fixture observations, or omit start identity | Do not merge different process instances; disclose ambiguous correlation |

Corroborated and confirmed outcomes need documented predicates and trusted fixtures. A candidate finding passes a weak-identity negative test, but does not satisfy a required known-agent strong-identity detection test. If strong predicates are not implemented yet, that positive test remains incomplete.

### Benign activity and Policy A

| ID | Setup / action | Expected result |
| --- | --- | --- |
| NEG-01 | Leave recognized assistants installed and inactive | Inventory only; no cheating, disqualification, execution or automatic removal claim |
| NEG-02 | Run npm install, a benign Node script, pip, Python training, Git and a compiler | No prohibited-assistant finding from interpreter/package-manager presence alone |
| NEG-03 | Run echo with an agent name and open a text file mentioning codex/claude/opencode | No execution finding from a substring, terminal text or document contents |
| NEG-04 | Use declared synthetic model artifacts or legitimate inference with Ollama while no coding assistant is active | No automatic coding-assistance violation from model/runtime presence |
| NEG-05 | Repeat a scan with no tool fixtures inside approved roots | Report scoped findings and limitations, not universal absence or a verified competition session |

### Failure reporting, privacy and read-only behavior

| ID | Setup / action | Expected result |
| --- | --- | --- |
| FAIL-01 | Deny executable metadata access for a controlled process/root; compare standard and elevated runs where available | Explicit per-source access gap; elevation does not promise all gaps vanish |
| FAIL-02 | End a controlled process between enumeration and enrichment | Process-exited/unavailable result, not a crash, false identity or clean result |
| FAIL-03 | Inject snapshot/collector/parser failure using test seams | Partial/failed report with reason; never complete coverage from an empty error result |
| FAIL-04 | Run on an untested OS/architecture fixture or unsupported mode | Unsupported/unvalidated capability is visible; no claimed support inferred from OS family alone |
| PRIV-01 | Place synthetic personal documents outside scope and audit collector accesses in a controlled test | No document/source-code/history inspection or traversal outside approved roots |
| PRIV-02 | Give a controlled process a unique fake secret in its arguments; inspect console, structured output and temporary/error logs | Fake secret and raw argument string do not persist; redaction applies to all output paths |
| PRIV-03 | Stop or cancel an approved scan | Stop promptly; incomplete/cancelled scope remains visible; no continuing hidden scan |
| SAFE-01 | Compare test software, model files and process state before/after a scan | No tool launch, termination, uninstall, file modification or deletion caused by the scanner |
| REPORT-01 | Exercise success, partial coverage, candidate findings and collector failure | Human and structured reports agree, include rule/source/scope details and never grant eligibility |

### Explicit later-milestone cases

| ID | Scenario | Milestone / expected M2 statement |
| --- | --- | --- |
| FUT-01 | Assistant starts and exits entirely between scans | M3 launch-event investigation; M2 cannot promise capture |
| FUT-02 | Event source loses permissions or drops events | M3 health/continuity test; M2 reports continuous monitoring outside scope |
| FUT-03 | Agent dies, backend disconnects or a verification lease expires | M4/M5 session tests; no M2 verified state exists |
| FUT-04 | New USB device, browser assistant, IDE feature or WSL guest agent | Outside M2 enforcement; no implied coverage from a snapshot report |

## 5. Required gates before M2 can be called complete

- Shared observation, finding, scope and coverage schemas are implemented and documented.
- Consent and bounded read-only inspection work without requiring administrator launch for the baseline.
- The declared baseline native/npm installation cases and active-at-snapshot positive cases have separate recorded outcomes on both OS lab configurations.
- Positive detection tests use a documented identity rule and trusted artifact ground truth; passing name-only candidate tests is not enough.
- Policy A, benign-development, privacy, failure and no-mutation cases pass for the implemented scope.
- Permission gaps and unsupported cases remain explicit even on otherwise successful scans.
- Formatting/lint/unit checks and real Windows integration procedures/results are recorded in the implementation chunk. CI success alone does not establish both Windows desktop configurations.
- README documents working commands only after they exist; measured gaps are published without exposing participant inventories.
- A reviewer can reproduce the report and distinguish tested functionality from deferred variants.

If only one OS has been exercised, describe that configuration's result and leave the other pending. Do not mark both Windows 10/11 validated.

## 6. Measurement method

For active-at-snapshot recall, count each required pinned tool/variant actually alive during collection as a trial. Separate supported identity detection, candidates, misses and blocked visibility; required blocked cases remain unmet and must not be hidden by dropping them from the denominator. Report per-OS and per-mode counts.

For false positives, count prohibited-execution findings across the named benign trials and report trial counts, fixtures and classifications. Inventory-only findings are not execution false positives. Do not claim a sub-2% rate from a handful of examples.

Record scan duration, peak resource use where measurable and timeout behavior. Snapshot duration is not runtime launch-detection latency. The architecture's 95% recall, 2% false-positive and five-second launch targets require a later, suitably sized runtime evaluation and are not achieved by completing M1 documentation.

## 7. M1 document review checklist

- [x] Record maintainer-selected Windows 10 and Windows 11 prototype targets.
- [x] Separate OS targets, proposed lab configurations and validated support.
- [x] Explain standard-user versus elevated inspection and propose a baseline.
- [x] Map initial tool/variant coverage to acceptance IDs.
- [x] Include negative, ambiguous-identity, privacy and failure cases.
- [x] Define the M2 report boundary and implementation exit gates.
- [ ] Maintainer review of baseline privilege scope, x64/lab versions and identity/action rules.
- [x] Authorize M2.1 non-inspection project/CLI setup on 2026-10-29; remaining review gates apply before the affected collection chunks.

M2.1 now has seven passing local CLI integration tests documented in [development instructions](development.md). They do not execute or satisfy the 33 scanner scenarios above.

M2.2 adds nineteen passing local [synthetic contract tests](shared-contracts.md) for preclassified observations, diagnostic policy, correlation and coverage. They test library semantics without device inspection or real catalog matching. All 33 scanner scenarios remain NOT RUN.

This completes preparation of the M1 coverage/test documents, not approval of all architecture proposals or execution of their tests.
