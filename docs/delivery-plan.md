# M2 closure and M3 delivery queue

Updated: 2026-11-06 (Asia/Calcutta). This is an execution plan, not a support or
completion claim. The maintainer authorized continued delivery of remaining M2
and then M3. Existing acceptance, privacy and review gates still apply.

[Public status](../PROJECT_STATUS.md) | [M2 exit gates](m2-completion.md) |
[M3 plan](m3-plan.md) | [Acceptance cases](acceptance-plan.md)

## Where work stands

M2 has bounded installation inventory, current-session process snapshots,
diagnostic reports, four exact native pins and controlled npm-launcher labs.
M3 has no runtime collector yet. Windows 10 remains deferred; Windows Server CI
does not replace desktop evidence. A successful partial trial closes only that
subcase, not its parent acceptance row or every installation variant.

## M2 work in dependency order

| Work item | Available evidence | Next concrete outcome | Dependency |
| --- | --- | --- | --- |
| CLI consistency (#29/#31/#37) | PR #48 merged after all seven checks, including CodeRabbit; 110 local tests/doctests pass | Complete: help aliases, deduplication and truthful scope-limit errors | Keep regression checks in the default suite |
| Collector capability (#35) | PR #49 merged after all seven checks, including CodeRabbit; 111 local tests/doctests and five real launcher controls pass | Complete: unavailable boot correlation disclosed independently of findings | Preserve empty, benign and candidate regression cases |
| Public contract corrections (#42/#44) | Manifest handles are scoped to one probe; wire projection has `MissingSuggestion` | Documentation now explains handle lifetime and the fallback action reason | No wire or lifetime behavior change |
| Required variant ledger | Native, npm metadata/dependency and real launcher records exist separately | Map each C01-C10/C13 variant to exact source, pin, mode and remaining outcome | Review the frozen four-tool scope; retain separate OpenCode generations |
| Boot transition | Private checkpoint and same-boot checks pass on the recorded source | Compare the preserved checkpoint after a known physical operation and publish only a minimized verdict | Operator identifies actual Restart versus shutdown/start or sleep; identical source revision |
| Desktop benign/privacy/failure checks | Controlled same-name processes, Node/npm and denied-directory cases pass; synthetic tests cover other branches | Expand missing real Windows 11 cases with owned fixtures and before/after checks | Existing trusted local tools only; unavailable software stays NOT RUN |
| Windows 10 | No desktop/VM evidence | Run the same required subset with its own build/token record | Maintainer supplies the deferred desktop/VM |
| M2 closeout | Required gates remain incomplete | Review per-case outcomes, support wording and reproduction commands | All required gates satisfied, CI and completed CodeRabbit |

No unattended reboot, sleep, shutdown, restore, elevation, security-policy change
or system-wide assistant installation is scheduled. Those operations can interrupt
the maintainer's other work. Do not replace a checkpoint merely because its
comparison is pending. Actual power operations and source labels must be recorded
truthfully; a test phase name cannot establish what the operator did.

## Acceptance ledger to finish

| Group | Existing evidence to reuse without upgrading its scope | Additional closure work |
| --- | --- | --- |
| INST-01/02; NEG-01 | [Native](validation/native-matrix-2026-11-02.json) and [npm metadata/dependency](validation/npm-inventory-2026-11-02.json) subcases | Confirm required variants independently; preserve inventory-only actions and inactive-tool ground truth |
| INST-03/04; SCOPE-01/02 | Declared-root, consent, junction, sharing and budget fixtures; real denied empty directory | Separate allowed custom-prefix, out-of-scope and denied cases in the desktop ledger; no whole-disk fallback |
| RUN-01/02/03; ID-05 | Four pins and [real launcher/headless version-query snapshots](validation/launcher-boot-2026-11-03.json) | Distinguish suspended image, npm runner, headless snapshot and actual daemon/interactive behavior; do not combine them into one full-row PASS |
| ID-01/03/04/06 | Same-name/same-size benign images, spoofed metadata, retained handles and synthetic PID/start races | Record exact real versus synthetic evidence and any unexecuted replacement/exit subcase; no cached path hash |
| ID-02 | Renamed variants are exploratory | Record an explicit candidate/miss/visibility outcome; no new broad hash search merely to complete this row |
| NEG-02/03/04/05 | [Completion-checked Node/npm control](validation/node-complete-2026-11-03.json), pure policy and scope fixtures | Separate available real development tools from simulated inference/model cases; absent Ollama is NOT RUN |
| FAIL-01/02/03/04 | [Real permission denial/recovery](validation/negative-controls-2026-11-03.json), provider fault and race fixtures | Keep unavailable elevated/protected-process cases explicit; empty results from failed inspection never count as clean |
| PRIV-01/02/03; SAFE-01; REPORT-01 | Controlled process fake arguments, scoped filesystem tests, decoded JSON privacy checks and text/JSON goldens | Review public text, structured output and private test logs; verify only owned cleanup and disclose synchronous cancellation limits |

The [2026-11-06 regression record](validation/launcher-regression-2026-11-06.json)
records all four real launcher positives and the completion-checked Node/npm
control on clean source `8d77af6`. It verifies the collector correction without
changing the scope of the original trials.

The original [unexecuted template](validation/windows-lab-template.json) remains a
template. Keep dated evidence records immutable in meaning: the withdrawn Node
negative was not retrospectively upgraded by its corrected rerun. FUT-01/FUT-02
belong to M3; FUT-03/FUT-04 are not M2 completion shortcuts.

## M3 implementation sequence after the entry gate

Each row is a separate reviewable delivery, followed by relevant checks, public
status, a commit/PR, completed CI/CodeRabbit and merge before the next delivery.

| Chunk | Concrete deliverable | Minimum acceptance before proceeding |
| --- | --- | --- |
| M3.1 | Source capability record comparing WMI trace events and ETW; one justified source/API/dependency decision | Actual token/build trials, subscription scope, readiness, stop cleanup and loss visibility; denied configurations explicit |
| M3.2 | Pure typed runtime observations and health reducer | Starting, Healthy, Degraded, Unavailable and Stopped traces; readiness plus reconciled baseline required for healthy; missing identity never correlates by PID alone |
| M3.3 | One bounded Windows adapter behind a narrow source interface | Owned-process launches, immediate field minimization, bounded queue/overflow, denied subscription and cancellation cleanup |
| M3.4 | Listener-first baseline reconciliation | Launch/exit around baseline boundaries, duplicate and reordered delivery, PID reuse and overflow; no reconstruction claim for missed short-lived processes |
| M3.5 | Explicit consented monitor invocation with finite duration and monotonic rescan schedule | Help/errors perform no collection; duration expiry, stop and resource cleanup; no startup service |
| M3.6 | Typed loss detection and bounded recovery generations | Disconnect, permission loss, queue overflow, retry exhaustion, cancellation during recovery; uncertain intervals retained after recovery |
| M3.7 | Separate minimized runtime report contract and CLI integration | Human/JSON agreement, fake secrets, truncation, closed pipe and stopped state; schema changes explicit; eligibility remains NotIssued |
| M3.8 | Desktop evidence and milestone review | Ground-truth launches, received/classified/missed counts, latency clocks, loss/failure trials, memory bounds and no-mutation records per actual OS/mode |

The [source research note](m3-source-review.md) prepares M3.1 without running an
event collector or selecting a source prematurely. The [eight-chunk plan](m3-plan.md)
defines the full contracts and exclusions. Linux collectors, persistent services,
competition sessions, backend/UI and automatic process control remain outside M3.

## Completion rule

Commit counts, green fixtures and successful event receipt are not milestone
completion criteria. Public status must identify implemented behavior, measured
subcases, untested variants and external dependencies separately. In particular,
deferring the Windows 10 VM does not silently waive the existing two-desktop gate.
Changing that gate needs an explicit maintainer decision in the decision log.
