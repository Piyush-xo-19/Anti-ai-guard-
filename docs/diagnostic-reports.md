# M2.5: integrated diagnostic scans and reports

Authorized: 2026-10-29 | Public JSON schema version 1 | Diagnostic only

```powershell
.\target\debug\anti-ai-guard.exe inventory --consent --npm-prefix "C:\approved\npm-prefix" --json
.\target\debug\anti-ai-guard.exe processes --consent --json
.\target\debug\anti-ai-guard.exe scan --consent --bin-root "C:\approved\bin" --processes
```

Choose real roots you approve; the paths above are placeholders. `scan` combines only selected installation roots/PATH and/or `--processes`. It never silently enables another scope. Missing consent, empty scope, duplicate/unknown flags and invalid roots return 2 before collection. All roots are validated before platform/process metadata queries. Help/version remain non-inspecting.

`scan.rs` collects once, keeps installation/process observations separate, combines execution and per-scope coverage, and applies the pure M2.2 Policy A evaluator. A process-to-installation link refers only to an installation indicator at the same declared executable location; it never upgrades identity. No cache or trusted catalog is fabricated. Current collectors emit candidates, so no execution violation, process action, disqualification or competition verification is issued.

## One minimized projection

Text and JSON render the same `PublicReport` value. JSON stdout contains one parseable object and a final newline, with no progress messages mixed into it. Invalid invocations use generic stderr errors without argument values. Broken output pipes are handled without a panic. There is no automatic report-file writer, telemetry upload or unbounded local report store; shell redirection is a separate user choice.

The schema includes scan/time identifiers, platform build/compiled architecture/current-token mode, requested and declared scope references, collector states/gaps, execution versus coverage, findings/evidence/action reasons, weak location links, metrics and explicit limitations. Eligibility is always `NotIssued`. The reported OS version/token are metadata, not proof that the platform has passed desktop acceptance. Windows Server is distinguished from Windows 10/11; compiled architecture is not claimed as the host's native architecture under emulation.

The output projection never serializes a raw `Snapshot` or Debug dump. It replaces scope labels with generated labels, restricts tool IDs to known built-in IDs, retains numeric process identity fields, and discards full image/root paths, free-text labels, arbitrary upstream rule references, raw boot-ID strings and unrecognized identifiers. Numeric build values and generated scan-ID syntax are validated. Package versions are omitted from the unified public projection. This is deliberate output minimization, not a general-purpose secret detector.

Candidate findings carry static, versioned indicator-rule IDs (`npm-package-name-indicators-v1`, `native-file-name-indicators-v1`, `process-basename-indicators-v1`) and `windows-name-and-package-indicators-v1` catalog metadata. Those rules identify candidates only; they are not authenticated strong-identity rules. Supplied upstream rule strings are not treated as trusted report identifiers.

Output is capped at 512 findings/links, 64 scopes/collector rows, 32 gaps per row and 1024 coverage concerns. Truncation is explicit and forces incomplete coverage/exit 3. Missing coverage, failed/cancelled collection and empty findings cannot be rendered as verified/clean results. Exit 0 means completed requested selector inspection without gaps; 3 means incomplete/failed/cancelled/truncated diagnostics; 1 means output failure; 2 means invalid invocation.

## Cancellation and timing

Collectors accept a cooperative cancellation flag. The CLI registers a best-effort Windows Ctrl+C/Break handler when a console permits it; the handler only sets an atomic flag. A pre-cancelled invocation skips platform and collector queries, while reporting cancelled requested scopes. Checks occur between bounded operations; a synchronous native I/O call can still block past the elapsed budget. A console without handler support retains normal process interruption, without a guarantee of a final report. Library callers should supply their own cancellation flag for embedded/concurrent use.

`collection_elapsed_ms` uses a monotonic clock around metadata and collectors, before rendering. It is not runtime launch-detection latency, an external timestamp or evidence that performance targets were met.

## Reproducible synthetic examples

The [report fixture source](../examples/report_fixture.rs) uses hard-coded observations, PID 42 and timestamp 1234. It never inspects the machine:

```powershell
cargo run --locked --offline --example report_fixture
cargo run --locked --offline --example report_fixture -- --json
```

See [synthetic text](examples/diagnostic.synthetic.txt) and [synthetic JSON](examples/diagnostic.synthetic.json). They are generated examples, not evidence of real assistants or platform support. Eight report tests cover text/JSON agreement, free-text/secret minimization, partial/failure/cancelled states, bounded truncation, invalid combined scope before process access, pre-cancellation explicit PATH opt-in and published golden-file agreement. Native/CLI and desktop evidence remain separately recorded in the validation review.

M2.4 limitations still apply: no trusted strong-identity catalog, no trusted boot ID, generic script hosts/renamed/custom agents outside native-name selection, and no continuous observation. Full Windows 10/11/pinned-artifact acceptance remains the M2.6 exit gate.

## Wire compatibility

See [public JSON schema v1](report-schema-v1.md) for required fields, null handling,
stable names and version rules. The example and golden-file test share the same
synthetic input. CI checks both JSON and text, plus fake-secret minimization.
