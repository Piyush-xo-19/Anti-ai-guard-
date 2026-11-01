# M2.6 available evidence

Updated 2026-11-01 (Asia/Calcutta); the dated evidence below is historical unless stated otherwise. Full M2 desktop acceptance remains INCOMPLETE.

| Check | Evidence | Result |
| --- | --- | --- |
| M2.4 candidate snapshots | [9888b70](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/9888b70f4122794b0ee94c86b3ef2746d943494e), [hosted CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40929011706) | PASS: 55 tests, 1 controlled-child helper ignored; no desktop/strong-identity claim |
| M2.5 report integration | [87772a3](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/87772a3a176e77e08400175d10bb0a8a33b95de9), [hosted CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40931370505) | PASS: 64 tests, 1 controlled-child helper ignored |
| Local selected GNU checks | Windows 11 build 26200, X64 compiled target, StandardUser token; Rust/Cargo 1.99.1 | PASS: format, strict Clippy, build, 57 selected tests including controlled native process fixture; 1 helper ignored |
| Local installation filesystem fixtures | Seven positive filesystem cases | BLOCKED: ancestor attributes denied by restricted session, Windows error 5 |
| Local PowerShell validation harness | Direct script invocation | BLOCKED: machine script-execution policy; policy unchanged, no bypass |
| Two hosted server fixture jobs | [M2.6 CI run 40933806067](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40933806067), [permanent sanitized evidence](fixtures-2026-10-30.json) | PASS on each: 64 tests, 0 failed, 1 controlled-child helper ignored; format/lint/build/examples/schema checks passed |
| Windows 10 pinned-tool desktop lab | No such lab attached | NOT RUN |
| Windows 11 complete pinned-tool desktop lab | Only restricted fixture environment available | NOT RUN |
| Strong assistant identity and renamed/script-hosted variants | No reviewed trusted artifact catalog | BLOCKED / NOT RUN; names remain candidates |
| Boot-scoped same-instance correlation | No trusted boot identifier | INCOMPLETE; explicit coverage gap |

The ignored local record is a maintainer artifact, not required contributor context. Public CI artifacts contain only minimized platform/toolchain/source/check summaries. No real participant inventory, PID list, absolute paths, command lines or genuine assistant trial data is published.

A passing fixture suite does not satisfy the 33 original desktop lab scenarios. The [lab template](windows-lab-template.json) leaves both OS configurations and every scenario NOT RUN. Complete the required variant/scope review, trusted identity work and real lab trials before promoting any desktop support or marking M2 complete.

## Hosted matrix details

Source revision: `f4270a2f28a64c278622f6c1638dc2cfa0ffa43a`, with a clean checkout in both jobs. Windows Server build 20348 and Windows Server build 26100 both used X64 compiled targets and reported ElevatedDiagnostic tokens. These are not standard-user desktop trials. The harness records M2 exit status INCOMPLETE even though fixtures pass.

The matrix exposed a validation-tool executable-resolution issue and a native fixture failure on the older server image. The source now selects one tool executable and uses the precise UTC FILETIME checkpoint rather than the coarse timestamp. Both jobs passed after those fixes; safe failure diagnostics retain only public test names and typed coverage codes.

## Local fixture rerun - 2026-11-01

Source 6568037, Rust 1.99.1 GNU, Windows 11 25H2 build 26200.9711; controlled native fixtures reported StandardUser. The restricted full run reproduced AccessDenied in the first installation fixture, followed by poisoned-lock failures in the remaining six cases. Running those seven existing tests with normal directory access passed all seven. The complete suite then passed with the same access: 80 tests plus two doctests, zero failures and one intentionally ignored child helper. Formatting, strict Clippy and rustdoc with warnings denied also passed. No code, ACL, no-reparse guard or script-execution policy was changed to obtain this result.

This supersedes the older claim that the filesystem fixtures could not be exercised locally: the restriction belongs to the sandboxed execution context, not to all local test runs. The direct PowerShell harness was not rerun, so its historical script-policy block is not cleared by this evidence. No pinned real assistant or Windows 10 trial ran. [The M2 closure checklist](../m2-completion.md) remains incomplete.
