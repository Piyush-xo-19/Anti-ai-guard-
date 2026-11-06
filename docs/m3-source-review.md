# M3.1 source research: decision and experiment checklist

Reviewed: 2026-11-06. Planning only; no source selected or subscription run.
M2's [entry gate](m3-plan.md#entry-gate) remains in force.

## Candidates and documented constraints

| Candidate | What the primary contract establishes | What still requires a desktop experiment |
| --- | --- | --- |
| WMI `Win32_ProcessStartTrace` / `Win32_ProcessStopTrace` | Start schema includes process/parent IDs, name, session, SID and event UTC time; it supplies no executable digest or process creation FILETIME. The event's security descriptor controls recipient access. [Start schema](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/krnlprov/win32-processstarttrace), [stop schema](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/krnlprov/win32-processstoptrace) | Current standard token's actual subscription rights, session predicate behavior, start/stop ordering, readiness, loss notification and cleanup |
| ETW process source | Controllers have documented privilege/group requirements; process tracing has a specific provider/configuration contract. Queryable session statistics include event and real-time buffer loss. [Controller access](https://learn.microsoft.com/en-us/windows/win32/etw/controlling-event-tracing-sessions), [process source](https://learn.microsoft.com/en-us/windows/win32/etw/process), [statistics](https://learn.microsoft.com/en-us/windows/desktop/ETW/event-trace-properties) | Exact permitted provider/session mode, current-token access, minimum OS configuration, payload minimization and reproducible loss/cleanup behavior |

Neither documented API availability nor being in user mode proves that a normal
participant token can use the source. Do not add group membership, enable audit
policy, elevate, start a permanent service or install a driver to make a trial pass.

For a bounded WMI experiment, Microsoft's
[ExecNotificationQuery contract](https://learn.microsoft.com/en-us/windows/win32/api/wbemcli/nf-wbemcli-iwbemservices-execnotificationquery)
requires return-immediately and forward-only flags. Releasing its enumerator
cancels the query. Verify timeout/error/cleanup behavior rather than treating the
returned enumerator as proof that all future events will be delivered.

WMI documents dropped-event and queue-overflow event classes in its
[event taxonomy](https://learn.microsoft.com/en-us/windows/win32/wmisdk/determining-the-type-of-event-to-receive).
That is not evidence that a selected temporary process subscription receives every
upstream loss signal. Prove the applicable path or describe continuity as unknown.

## Experiment contract to implement after M2

1. Use a clean committed source and explicit lab consent. Record OS/build, compiled
   architecture and actual token mode. Use only an owned disposable process corpus.
2. Declare a finite experiment duration, bounded receive queue and local counters
   before subscribing. Use a unique owned subscription/session; never stop someone
   else's trace session or overwrite its configuration.
3. Attempt the current-session filter at the source. Investigate the unavoidable
   raw payload separately from what the adapter returns. Discard SIDs, security
   descriptors, personal paths, arguments and unrelated names immediately; retain
   no raw event dump. Server-side filtering is not assumed from query syntax alone.
4. Record subscription acceptance, known owned probe receipt and baseline timing
   separately. A successful API call is not a delivery-continuity proof. Test launch
   before, during and after baseline, rapid exit, duplicate/out-of-order delivery,
   cancellation and failure before observation.
5. Keep source UTC event time distinct from local monotonic receipt/deadline time.
   **Inference:** WMI's event timestamp and PID are insufficient to substitute for
   M2's boot/start instance identity. Enrichment that finds an already exited or
   reused PID must leave identity unresolved, not confirm a tool from its name.
6. Bound queue and retry behavior. Inject a local queue overflow separately from
   testing provider/session loss. An empty event stream or zero known loss counter
   is not proof that all launches were observed.
7. Stop within a bounded wait and release only owned resources. Confirm no active
   subscription or worker remains. Publish only minimized counters, typed outcomes,
   source revision and exact tested configuration; ground-truth misses stay in the
   denominator.

## Selection gate

Choose an adapter only after the capability record answers privilege, scope,
privacy, readiness, timestamp, loss and cleanup questions. If neither source meets
the declared standard-user contract, publish that result and propose a scope or
privilege decision; polling snapshots cannot be relabeled continuous coverage.
Review the Rust native ABI/dependency choice with that evidence. No dependency,
monitor CLI, event wire schema or performance claim is introduced by this note.
