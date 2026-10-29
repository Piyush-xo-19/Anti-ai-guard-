# M2.6 available evidence

As of 2026-10-30 (Asia/Calcutta). Full M2 desktop acceptance remains INCOMPLETE.

| Check | Evidence | Result |
| --- | --- | --- |
| M2.4 candidate snapshots | [9888b70](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/9888b70f4122794b0ee94c86b3ef2746d943494e), [hosted CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40929011706) | PASS: 55 tests, 1 controlled-child helper ignored; no desktop/strong-identity claim |
| M2.5 report integration | [87772a3](https://github.com/Piyush-xo-19/Anti-ai-guard-/commit/87772a3a176e77e08400175d10bb0a8a33b95de9), [hosted CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40931370505) | PASS: 64 tests, 1 controlled-child helper ignored |
| Local selected GNU checks | Windows 11 build 26200, X64 compiled target, StandardUser token; Rust/Cargo 1.99.1 | PASS: format, strict Clippy, build, 57 selected tests including controlled native process fixture; 1 helper ignored |
| Local installation filesystem fixtures | Seven positive filesystem cases | BLOCKED: ancestor attributes denied by restricted session, Windows error 5 |
| Local PowerShell validation harness | Direct script invocation | BLOCKED: machine script-execution policy; policy unchanged, no bypass |
| Two hosted server fixture jobs | M2.6 workflow on windows-2022/windows-2025 | PENDING: result recorded after publication |
| Windows 10 pinned-tool desktop lab | No such lab attached | NOT RUN |
| Windows 11 complete pinned-tool desktop lab | Only restricted fixture environment available | NOT RUN |
| Strong assistant identity and renamed/script-hosted variants | No reviewed trusted artifact catalog | BLOCKED / NOT RUN; names remain candidates |
| Boot-scoped same-instance correlation | No trusted boot identifier | INCOMPLETE; explicit coverage gap |

The ignored local record is a maintainer artifact, not required contributor context. Public CI artifacts contain only minimized platform/toolchain/source/check summaries. No real participant inventory, PID list, absolute paths, command lines or genuine assistant trial data is published.

A passing fixture suite does not satisfy the 33 original desktop lab scenarios. The [lab template](windows-lab-template.json) leaves both OS configurations and every scenario NOT RUN. Complete the required variant/scope review, trusted identity work and real lab trials before promoting any desktop support or marking M2 complete.
