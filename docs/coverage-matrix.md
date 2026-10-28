# M1: platform, privileges and detection coverage

Version: 0.1 | Updated: 2026-10-28 | Status: implementation contract draft; no detection results yet

[Architecture](architecture-v1.md) | [Acceptance plan](acceptance-plan.md) | [README](../README.md)

## 1. Decisions and boundaries

**Maintainer decision:** the first prototype targets native Windows 10 and Windows 11. Linux remains a later v1 target. M2.1 project/CLI setup was authorized on 2026-10-29 with no device inspection; the remaining M1 scope review is required before the affected collection/classification chunks.

**Proposed implementation baseline:** standard-user inspection of the consenting participant's session and readable installation metadata. Elevation is optional diagnostic coverage to investigate, not a requirement to run the first CLI and not a claim of complete visibility. Both modes run in Ring 3.

M2 is an on-demand, read-only preflight snapshot and report. It does not implement continuous event monitoring, organizer verification, suspension, a backend, storage-device enforcement or a Windows service. Those belong to subsequent milestones.

## 2. Platform validation matrix

| ID | Target | Initial lab configuration | State |
| --- | --- | --- | --- |
| OS-W10 | Native Windows 10 | Proposed baseline: 22H2 x64; record edition, exact OS build and patch level for every run | Selected OS family; not validated |
| OS-W11 | Native Windows 11 | Proposed baseline: 24H2 x64; test other available releases, including 25H2, as separate configurations | Selected OS family; not validated |
| OS-ARM | Windows on ARM64 / emulated processes | Separate compatibility investigation | Deferred from initial prototype |
| OS-X86 | 32-bit Windows/agent | Separate compatibility investigation | Deferred from initial prototype |
| OS-LNX | Linux distributions | Choose configurations after Windows prototype | Later v1 work |
| OS-ISO | WSL, VM guests and containers | Host findings do not establish guest coverage | Outside initial coverage |

The x64 baseline and initial lab releases are engineering proposals, not additional maintainer-approved limits. Selecting Windows 10/11 does not establish support for every historical build, edition, architecture or tool release. Promote a configuration to validated only after its required acceptance rows pass. No operating-system lifecycle or security-maintenance guarantee is made by this project.

## 3. What standard-user and elevated mean

| Mode | How it runs | Intended scope | What cannot be assumed |
| --- | --- | --- | --- |
| Standard user | Normal terminal; no administrator prompt | Readable current-user installation roots, PATH metadata and observable process identities | Access to every other-user/protected process, directory, service or event source |
| Elevated diagnostic | Participant explicitly starts an administrator terminal | Compare additional readable metadata and capability differences using the same read-only rules | Universal access, tamper resistance, execution prevention or Ring 0 operation |

An administrator account using a normal unelevated terminal is still the standard-user test mode. Record the process token/elevation mode rather than assuming it from account membership.

The scanner must not request terminate, suspend, injection or process-memory write rights. Read only the minimum metadata needed. An access-denied result is a coverage gap, not evidence that the process is benign or prohibited. A process that exits during enrichment is distinct from access denied. Expected gaps can be reported without crashing the scan, but must remain visible.

### Proposed collector approach

| Collector | M2 candidate | Access/error behavior |
| --- | --- | --- |
| Process enumeration | Supported Windows snapshot API, directly or through a reviewed Rust library | Distinguish snapshot failure from individual enrichment failures |
| Executable identity | Limited process queries where allowed; scoped readable file metadata | Do not equate a basename with confirmed identity |
| Installation metadata | PATH and explicit supported user/package installation roots | Bounded inspection; custom prefixes only if declared or discovered from approved metadata |
| Process start time | Supported metadata when accessible | Missing start identity downgrades PID correlation |
| Launch metadata | Optional scoped enrichment when justified | Transient local use; redact before persistence; no complete command-line dump |
| Process launch events | Capability research for M3, not an M2 feature | Probe source readiness and privileges separately; no polling-only continuity claim |

Microsoft documents process snapshots and access-controlled executable queries. Its ProcessStartTrace event exposes a process name, IDs and other fields, but its published schema does not contain a command-line field. Consequently, an event observation is not automatically a complete tool-identity finding. This last statement is a design inference from those API contracts. [Snapshot API](https://learn.microsoft.com/en-us/windows/win32/toolhelp/taking-a-snapshot-and-viewing-processes), [process rights](https://learn.microsoft.com/en-us/windows/win32/procthread/process-security-and-access-rights), [executable query](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-queryfullprocessimagenamew), [ProcessStartTrace](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/krnlprov/win32-processstarttrace).

No universal event-source availability or permission claim is made. Choose an M3 source only after a Windows 10/11 capability experiment.

## 4. Initial tool identity catalog

| Tool family | Recognized package identifier candidates | Native-install variants to investigate | Source |
| --- | --- | --- | --- |
| Codex CLI | `@openai/codex` | Windows installer, standalone release binary and package-selected executable | [Official repository](https://github.com/openai/codex) |
| Claude Code | `@anthropic-ai/claude-code` | Native Windows installer, WinGet and npm-selected executable | [Official setup](https://code.claude.com/docs/en/setup) |
| OpenCode 1 | `opencode-ai` | Native release and supported Windows package-manager layouts | [Official v1 documentation](https://opencode.ai/docs/) |
| OpenCode 2 | `@opencode/cli` | Standalone Windows executable and package-selected executable | [Official v2 documentation](https://opencode.ai/v2/docs/) |

The two OpenCode generations need separate rules even though both use the `opencode` command. Do not import v1 installation assumptions into v2. For every lab artifact, pin a release/version, architecture, acquisition URL and expected identity before testing; package names above are candidate inventory selectors, not integrity proofs.

Current Claude npm documentation describes a native binary. OpenCode 2 also documents package installation selecting a native binary. Therefore, npm metadata and the actual child executable need correlation; Node ancestry is not mandatory evidence for every npm-installed agent. [Claude installation behavior](https://code.claude.com/docs/en/setup#install-with-npm), [OpenCode 2 installation behavior](https://opencode.ai/v2/docs/).

Discovery must never execute an unknown assistant, invoke its help/version command, run package lifecycle scripts, install software or send model requests. Controlled test setup is separate from scanner behavior.

## 5. Coverage rows and acceptance mapping

All rows below are planned and unexecuted. M2 identity findings are diagnostic and never grant competition verification.

Evidence abbreviations: **P** = readable package metadata; **X** = executable identity/path; **H** = trusted pinned executable hash or equivalent approved strong identity; **I** = process instance including start identity where available; **C** = corroborated package entry point/launch context.

| ID | Scenario | Evidence to investigate | M2 expected behavior | Acceptance IDs |
| --- | --- | --- | --- | --- |
| C01 | Codex native install, inactive | X plus H if cataloged | Inventory; installation alone is allowed | INST-01, NEG-01 |
| C02 | Codex npm install, inactive | P and selected X | Inventory with version/source and limits | INST-02 |
| C03 | Codex active at scan time | I + X + approved H/C | Supported execution finding if rule passes; otherwise candidate | RUN-01, ID-01 |
| C04 | Claude native install, inactive | X plus H if cataloged | Inventory only | INST-01, NEG-01 |
| C05 | Claude npm install, inactive | P and selected X | Inventory; do not assume Node runtime | INST-02 |
| C06 | Claude active at scan time | I + X + approved H/C | Supported execution finding or explicit candidate | RUN-01, ID-01 |
| C07 | OpenCode 1 install, inactive | P `opencode-ai` and/or X | Inventory with generation/rule identity | INST-01, INST-02, ID-05 |
| C08 | OpenCode 2 install, inactive | P `@opencode/cli` and/or X | Separate inventory rule; report unknown generation if unresolved | INST-01, INST-02, ID-05 |
| C09 | Either OpenCode generation active | I + X + approved H/C | Execution finding tied to the observed generation/artifact | RUN-01, ID-05 |
| C10 | Declared custom npm prefix | Bounded approved root + P/X | Inventory if readable; otherwise scoped incomplete result | INST-03 |
| C11 | Readable portable executable in an approved root | X and H if cataloged | Candidate or supported inventory; no whole-disk search | INST-04, PRIV-01 |
| C12 | Package runner with ephemeral cache | I/X with available P/C | Candidate if corroboration unavailable; never flag npm/npx generally | RUN-02, NEG-02 |
| C13 | Known noninteractive/background process active at snapshot | I/X and approved identity | Same identity logic; do not depend on a visible terminal window | RUN-03 |
| C14 | Renamed known-version executable | I/X and H | Recognize only where artifact access and pinned hash permit | ID-02 |
| C15 | Changed/unknown executable or forged metadata | Incomplete or conflicting P/X/H | Candidate/unknown; no fabricated confirmation | ID-03, ID-04 |
| C16 | Other-user/protected/inaccessible process | Partial I/X; explicit error | Report visibility gap; do not claim a complete clean scan | FAIL-01 |
| C17 | Assistant starts and exits between snapshots | May be entirely unobserved | Document not covered by M2; event tests belong to M3 | FUT-01 |
| C18 | Inactive Ollama/model artifacts or legitimate inference | Only explicitly scoped indicators | No automatic prohibited-assistant finding | NEG-04 |

M2's required installed-CLI baseline is C01/C02/C04/C05/C07/C08/C10 within documented readable roots. C11/C12/C14 are measured exploratory variants; do not advertise them as supported until demonstrated. C03/C06/C09/C13 are active-at-snapshot observations, not continuous execution coverage.

## 6. Finding/report contract

Proposed top-level report fields: schema version, scan ID/time, platform/build/architecture, inspection mode, inspected scope, rule/catalog version, findings, per-collector coverage, errors and omitted capabilities.

Each finding includes category, tool ID or unknown identity, installation versus observed execution, evidence class, rule ID, redacted source explanation and relevant process-instance reference. Do not persist raw command lines, credentials, prompts or unnecessary absolute personal paths.

M2.2 now implements the [in-memory shared contracts and diagnostic evaluator](shared-contracts.md) for supplied observations. Scope uses opaque IDs/minimized labels, and gap/action explanations use typed reasons. Real collection, identity matching, rendered/serialized reports and consent enforcement remain planned; these types do not validate any coverage row above.

Keep these concepts separate:

- **Scan execution:** finished, partial or failed.
- **Scope coverage:** inspected, partial, unavailable, not requested or outside milestone.
- **Identity evidence:** inventory, candidate, corroborated or confirmed supported identity.
- **Suggested preflight action:** inventory only, request closure, request review or coverage incomplete.
- **Competition eligibility:** not issued by the standalone M2 scanner.

An empty findings list is not proof that no assistants exist. Names or participant-writable metadata alone produce candidates. Strong identity predicates need a documented trusted catalog artifact; until that exists, the scanner must not pretend to confirm identity.

## 7. M2 delivery and later work

M2 should introduce a read-only Rust CLI, bounded collector interfaces, explicit consent/scope selection, human-readable and structured diagnostic output, fixtures and Windows snapshot tests. Exact flags, serialization library and Rust dependencies are M2 implementation choices.

M3 will investigate launch events and event loss. M4 adds durable session decisions/audit. Organizer leases, disqualification review, storage policy enforcement, model-store inventories beyond declared scope and dashboards are not shortcuts to M2 completion.

## 8. M1 completion boundary

The coverage contract and acceptance procedures can be completed as documentation now; no test result is implied. Before calling M1 fully approved, review the proposed x64/lab-release baseline, standard-user scope, evidence rules and M2 exit criteria. Backend retention/leases and IDE feature controls may remain open because M2 neither grants competition eligibility nor implements those features.
