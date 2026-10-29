# M2.4: bounded current-session process snapshots

The maintainer authorized M2.4/M2.5/M2.6 on 2026-10-29. This collector uses the current Windows token without elevation and preserves the candidate-only, diagnostic boundary.

```powershell
.\target\debug\anti-ai-guard.exe processes --consent
```

Help requires no consent; collection does. The Windows Toolhelp process snapshot supplies PIDs and executable basenames transiently. Only known native name indicators (Codex, Claude and unknown-generation OpenCode) are considered for enrichment. Session membership filters candidates to the current Windows session. Generic Node, Python, package managers, shells, model runtimes and arbitrary substrings are not assistant findings.

Handles request only `PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE`. Creation time, full executable image location and zero-time exit-state checks use the same owned handle. There are no command-line, environment, process-memory, module, terminal-history or source-code queries. No terminate, suspend, injection or memory-write API is used. All handles are non-inherited and closed by RAII. [Toolhelp snapshot](https://learn.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-createtoolhelp32snapshot), [process entry layout](https://learn.microsoft.com/en-us/windows/win32/api/tlhelp32/ns-tlhelp32-processentry32w), [creation time](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes), [image query](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-queryfullprocessimagenamew)

Full image paths are used transiently, then discarded. An optional link to an installation indicator means only the same declared executable location in this report. It is not artifact identity, package-entry-point proof or permission to upgrade identity. No artifact-hash cache exists to reuse stale path hashes.

The pre-snapshot checkpoint uses GetSystemTimePreciseAsFileTime to avoid coarse-clock false races. Creation after that checkpoint or an image basename differing from its snapshot entry yields `ProcessIdentityChanged`; that row cannot inherit the old snapshot identity. Access denied, exiting processes, unavailable start identity and enumeration failures remain distinct gaps. These conservative checks may also reject a genuine process launched in the capture window; they do not claim perfect snapshot atomicity.

Defaults bound enumeration to 4096 rows, findings to 512 and elapsed time to two cooperative seconds. Library hard bounds are 16384 rows, 512 findings and ten seconds. Native snapshot construction and synchronous OS calls are not hard time-limited. Cancellation and limits prevent further queries and remain explicit in execution/coverage status. Linux is unsupported.

**Identity limitations:** names alone stay candidates. No production hash/publisher catalog or strong artifact matcher is supplied. Renamed/custom assistants and script-hosted agents hidden behind a generic interpreter name remain outside this native-name selector. No command-line substring inference is added to fill that gap.

**Boot/correlation limitation:** creation FILETIME is recorded when accessible, but no supported trusted boot identifier is fabricated from uptime or wall-clock estimates. `boot_id` remains absent and `MissingBootIdentity` makes candidate coverage incomplete. PID reuse with differing known start times is rejected; cross-scan same-instance correlation remains conservative. This is an open M2 validation gate, not verified monitoring.

Exit 0 means requested name selectors were inspected without gaps, never a clean device. Exit 3 means partial/failed/cancelled coverage. Missing consent/invalid arguments return 2 before snapshot creation. Report integration is M2.5; continuous launch events remain M3.

Seven provider-driven tests cover consent/cancellation/budgets before capture, current-session filtering, interpreter negatives, capture failure, denied/exited/changed metadata, PID/image races and weak installation linkage. The native integration fixture launches a copy of its own test binary named `codex.exe`, passes a fake secret argument and verifies candidate classification plus omission of arguments/paths. It never launches a real assistant. Exact test results are recorded in public status and validation evidence; fixture passes do not validate pinned real assistant artifacts or both Windows desktop labs.
