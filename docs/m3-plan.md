# M3 plan: Windows runtime observation

Updated: 2026-11-01 (Asia/Calcutta)
State: planning only; implementation queued behind incomplete M2 acceptance.

[Status](../PROJECT_STATUS.md) | [M2 completion](m2-completion.md) | [Architecture](architecture-v1.md) | [Decisions](decisions.md)

The [delivery queue](delivery-plan.md) gives the ordered implementation outcomes.
The [source research note](m3-source-review.md) links primary API contracts and
defines capability experiments; no source is selected or exercised by that note.

## Outcome and boundary

Extend the consented Windows preflight with runtime observations, reconciliation
and explicit health/continuity reporting. Keep installation evidence separate
from execution evidence and keep ambiguous identity under review. M3 remains a
Ring 3 diagnostic prototype for native Windows 10/11. Linux runtime collectors,
services/installers, durable session/audit/outbox, organizer verification and user
interfaces remain later work. No automatic process termination or competition
penalty is introduced. Eligibility remains NotIssued.

There are **eight proposed implementation chunks, M3.1 through M3.8**. These are
reviewable steps, not eight product versions or eight releases. They can split
into smaller commits when needed; a scope/source change requires a documented
review rather than silently changing the plan.

## Entry gate

- [ ] Close the [M2 completion checklist](m2-completion.md), with required desktop,
  trusted identity and boot-correlation outcomes recorded honestly.
- [ ] Review the runtime source's actual privileges, system-wide versus session
  visibility, fields and privacy boundary before enabling it on a real device.
- [ ] Agree supported Windows configurations, controlled launch fixtures and
  ground truth. Do not assume that standard-user access works for a chosen source.
- [ ] Decide dependencies/native API approach and record capability evidence.

Publication of this plan is authorized; these unchecked gates are not approvals
or executed tests. No runtime source has been selected by this document. A source
that cannot establish its scope/readiness must report unavailable; a snapshot
fallback cannot masquerade as continuous monitoring.

## Implementation chunks

| Chunk | Outcome | Status |
| --- | --- | --- |
| M3.1 | Source capability experiment and selection | Not started; entry gate pending |
| M3.2 | Runtime contracts and fixture-driven health reducer | Not started |
| M3.3 | Bounded Windows event adapter | Not started |
| M3.4 | Listener-first baseline reconciliation | Not started |
| M3.5 | Consented monitor lifecycle and targeted rescans | Not started |
| M3.6 | Loss detection, bounded recovery and continuity generations | Not started |
| M3.7 | Minimized runtime reports and CLI integration | Not started |
| M3.8 | Windows desktop trials and milestone review | Not started |

### M3.1: Source capability experiment and selection

Compare candidate documented Windows process-event interfaces. Record the exact
API/provider, event fields, token requirements, subscription ownership, readiness
behavior, timestamp/ordering guarantees and loss/error signals on both desktop
OS configurations. Evaluate session filtering and any unavoidable receipt of
out-of-scope metadata before choosing a source. No automatic elevation, enabling
permanent audit policies, installing a driver or starting system services.

Acceptance: a short decision record links primary API documentation and actual
capability trials. Unsupported/denied configurations remain explicit. Lack of a
loss indicator cannot be described as proof of complete delivery. Choose one
reviewed adapter for the next chunk; extra sources are not mandatory scope.

### M3.2: Runtime contracts and health reducer

Define typed observations, opaque run/source generation IDs, observation time
versus local monotonic receive time, readiness, loss/overflow/error markers and
bounded queue/deadline policies. Source receipt IDs deduplicate delivery; they do
not establish process identity. Correlation requires justified instance evidence,
not PID/name reuse. Define Starting/Healthy/Degraded/Unavailable and Stopped as
monitoring states, distinct from competition eligibility.

Acceptance: synthetic traces cover start/stop, duplicate and reordered delivery,
missing identity, PID reuse, cancellation and health transitions. Unknown or
unobservable continuity stays uncertain. No host collection in pure reducer tests.

### M3.3: Bounded Windows event adapter

Implement the reviewed source behind a narrow interface. Consent and declared
scope precede subscription. Minimize fields immediately; callbacks enqueue bounded
metadata and never block on long enrichment. Dispose owned handles/subscriptions
on cancellation and failure. Retain no argv, environment, source code, window
contents, personal paths or raw boot identifiers in public output.

Acceptance: controlled owned-process fixtures show source readiness, receipt,
scope filtering, permission failures and cleanup. Names remain candidates until
reviewed identity evidence supports more. Short-lived processes that disappear
before enrichment remain observations with missing evidence, not confirmed tools.

### M3.4: Listener-first baseline reconciliation

Subscribe and confirm readiness before the bounded initial snapshot. Buffer the
intervening events within explicit limits, reconcile the snapshot interval, and
preserve uncertain order or identity instead of guessing. Merge duplicate evidence
only for a justified instance; preserve distinct launches and conflicting evidence.
A snapshot cannot reconstruct a short-lived process missed during an event gap.

Acceptance: deterministic fixtures cover launches/exits before, during and after
the baseline; duplicate receipt; delayed delivery; PID reuse; missing boot/start
identity; and buffer overflow. Baseline completion without subscription readiness
cannot produce Healthy continuous coverage.

### M3.5: Monitor lifecycle and targeted rescans

Add one explicitly consented, bounded monitor invocation after reviewing its CLI
contract. Define duration, cooperative cancellation, stop cleanup and monotonic
rescan scheduling. Reuse only approved scopes and M2 identity rules. Rescans check
persistent activity and drift; they are reconciliation, not a replacement for
launch events. No persistent auto-start service or participant application yet.

Acceptance: controlled fixtures demonstrate start, baseline, observation, manual
stop, duration expiry and cleanup; help/version/errors perform no collection.
Timers have deterministic unit tests. Synchronous OS calls remain a documented
limit; no hard latency guarantee follows from cooperative deadlines.

### M3.6: Loss detection and bounded recovery

Treat source disconnect, reported loss, queue overflow, enrichment failure and
permission changes as typed health events. Bound retries and memory; use backoff
and a new generation/checkpoint after reattachment. Recovery requires fresh
readiness and reconciliation, and never erases the earlier uncertain interval.
Separate expected identity limitations from operational failures without hiding
incomplete coverage. M4 owns durable restart/session recovery.

Acceptance: fault injection covers each observed source failure, overflow, retry
exhaustion, recovery, stop during recovery and failure after baseline. No silent
reset to healthy and no retroactive completeness claim. Source failures are not
reported as evidence of prohibited execution.

### M3.7: Runtime reports and CLI integration

Render human and structured diagnostics from the same minimized typed model.
Expose scope, source capability, generation/interval, findings, gaps, health and
whether observation is currently stopped. Specify any public wire changes using
the [schema compatibility policy](report-schema-v2.md); do not silently add M3
semantics to the M2 report. Keep generated IDs and bounded output; do not persist
raw event streams by default.

Acceptance: synthetic golden reports, fake-secret tests, truncation and closed-pipe
behavior, cancellation, and health/finding agreement pass. Successful receipt of
an event never grants eligibility. An empty event stream cannot prove a clean
machine or a complete interval.

### M3.8: Desktop evidence and milestone review

Run the declared native Windows 10/11 token/build/variant matrix with controlled
pinned tools, benign development cases and failure injection. Include FUT-01
(short-lived execution) and FUT-02 (loss/permission failure) from the original
acceptance plan, plus reconciliation, memory bounds, stop and no-mutation cases.
Record ground-truth launch counts, received/classified counts and unavailable or
uncertain intervals separately. Measure latency with defined clocks and report
the corpus/denominator; do not exclude misses from recall calculations.

Acceptance: public minimized results identify exactly what passed, failed or was
not run. Snapshot fixtures/Windows Server CI do not replace desktop runtime trials.
The architecture's recall, false-positive and five-second targets remain unachieved
until suitable trials support them. Close M3 only after its declared required
cases and privacy/health gates have outcomes; M4 starts separately afterward.

## Delivery and public progress

For each chunk: implement one bounded outcome, run relevant tests, update public
status and the ignored local tracker, commit, push a focused PR, wait for all
checks including CodeRabbit, address applicable findings, then merge. A cancelled
or skipped review is not an approval; record and resolve the reason. Do not keep
M3 progress only in a private file. No tests, collector availability or milestone
completion may be inferred from this planning document.
