# M2.6 available evidence

Updated 2026-11-01 (Asia/Calcutta); the dated evidence below is historical unless stated otherwise. Full M2 desktop acceptance remains INCOMPLETE.

| Check | Evidence | Result |
| --- | --- | --- |
| M2.4 candidate snapshots | [9888b70](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/9888b70f4122794b0ee94c86b3ef2746d943494e), [hosted CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40929011706) | PASS: 55 tests, 1 controlled-child helper ignored; no desktop/strong-identity claim |
| M2.5 report integration | [87772a3](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/87772a3a176e77e08400175d10bb0a8a33b95de9), [hosted CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40931370505) | PASS: 64 tests, 1 controlled-child helper ignored |
| Local selected GNU checks | Windows 11 build 26200, X64 compiled target, StandardUser token; Rust/Cargo 1.99.1 | PASS: format, strict Clippy, build, 57 selected tests including controlled native process fixture; 1 helper ignored |
| Local installation filesystem fixtures | Seven positive filesystem cases | Restricted-session run: BLOCKED by ancestor access; normal-access rerun on 2026-11-01: all seven PASS. See the dated rerun below. |
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

## Native identity pilot - 2026-11-01

[Sanitized partial evidence](identity-pilot-2026-11-01.json) records exact source
35fd1ad with a clean checkout for the opt-in Codex0.166.0 x64 trial. On Windows11
25H2 build26200.9711, non-elevated context, inactive inventory remained InventoryOnly.
The owned suspended-image trial produced Corroborated evidence, a diagnostic
RequestClosureAndRescan suggestion, approved rule metadata and NotIssued eligibility.
The downloaded artifact's SHA-256 still matched its official pin after both trials.

The default suite passed 95 tests/doctests with zero failures, two deliberately
ignored helpers/labs; the separately opted-in real-artifact test passed once.
Same-name/same-size benign and distinct-identical-file negatives remain fixtures.
This is a partial Codex native subcase, not completion of an acceptance row across
all variants. The process was deliberately paused for deterministic image identity;
interactive coding, package-runner/background behavior and runtime recall were
not tested. Boot restart/restore, other catalogs/variants and Windows10 still need
separate results. Full M2 remains INCOMPLETE; M3 has not started.

## Four native artifact subcases - 2026-11-02

[Minimized clean-source evidence](native-matrix-2026-11-02.json) records source
4d291be on Windows 11 25H2 build 26200.9711, StandardUser. All four frozen native
artifacts passed separate inactive InventoryOnly and owned suspended-image
Corroborated/RequestClosureAndRescan trials; eligibility stayed NotIssued.
Each artifact digest was unchanged after its trials. The default suite passed
98 tests/doctests, zero failures, two intentional ignored helper/lab tests;
format, strict Clippy and strict rustdoc passed. Same-name/same-size benign
controls cover all four pins and remain candidates. These are partial native
subcases, not interactive coding, full acceptance rows or performance targets.
Windows 10 is deferred; reboot/restore and broader desktop cases remain open.

## Pinned npm metadata and dependency images - 2026-11-02

[Minimized package evidence](npm-inventory-2026-11-02.json) records clean source
d1c719c on the same Windows11 StandardUser lab. Four exact packages were installed
only in ignored repository-local prefixes, with lifecycle scripts disabled, no
audit/funding requests and an isolated cache/config. npm lockfile integrity
matched the official registry metadata. Package metadata inventory passed for
Codex, Claude and both OpenCode generations: Candidate, InventoryOnly, NotIssued;
manifest hashes were unchanged after scanning. Their four x64 native dependency
executables independently matched the compiled pins and passed direct owned
suspended-image trials; post-trial digests were unchanged.

This supplies separate INST-02 metadata and package-native-image subcases. It is
not normal postinstall/launcher validation or RUN-02 package-runner execution.
No lifecycle script, npx assistant, model request or personal package prefix ran.
Baseline CPU variants, background execution and the full desktop matrix remain
open. Native-name inventory cannot infer an OpenCode generation; exact package
metadata and matched process digests are separate evidence sources.

## npm launcher and same-boot checkpoint - 2026-11-03

[Minimized committed-source evidence](launcher-boot-2026-11-03.json) records clean
source 0e0ba31 on Windows 11 25H2 build 26200.9711, StandardUser. All four pinned
npm exec version queries produced the expected job-owned Corroborated image and
diagnostic closure suggestion, with NotIssued eligibility. Version output and
entrypoint hashes were checked, with isolated profiles and requested no-window
launch flags. Controlled stdout backpressure held the version write during the
snapshot. These are partial launcher/headless cases, not daemon or interactive
behavior, launch-event recall or default-budget performance results.

The private boot baseline was created and a same-boot comparison passed. Negative
trials refused overwriting that baseline, a restart label without a restart, and
a mismatched source label; the baseline remained unchanged. No actual power
transition occurred. The baseline nonce/fingerprint stays private and ignored.
Default suite: 106 tests/doctests passed, zero failures, five intentionally ignored
labs/helpers. Strict formatting, Clippy and rustdoc checks passed. M2 remains
incomplete; Windows 10 VM and actual power transitions are not claimed.

## Real Node/npm and denied-directory controls - 2026-11-03

[Clean-source negative controls](negative-controls-2026-11-03.json) use source
63da5fb on the same Windows 11 StandardUser lab. The ordinary npm/Node version
query had both owned interpreters alive around collection and returned zero
owned assistant findings, but its harness did not require completed/inspected
collection. That negative verdict is withdrawn (BLOCKED); a corrected-source
rerun must replace it. The independent denied-directory case below remains valid.

A separate newly created empty lab directory received a temporary explicit deny
of directory-list/read-attributes for the current token. Inventory returned
exit3, Failed, Unavailable, AccessDenied, operational gaps and NotIssued. Its
original DACL was restored in finally and independently compared with the saved
Access descriptor. The same root then returned Finished/NotIssued/exit0. Only
that empty owned directory was removed. No existing user directory ACL changed,
and no raw path or SID is published. This is a standard-user FAIL-01 subcase;
an elevated comparison and the full desktop matrix remain separate.
