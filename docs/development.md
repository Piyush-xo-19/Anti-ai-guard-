# Building and testing candidate preflight

Updated: 2026-10-30 | M2.1-M2.5 implemented; M2.6 validation tooling with open desktop gates

[Public status](../PROJECT_STATUS.md) | [M2 plan](m2-plan.md) | [Contributing](../CONTRIBUTING.md)

## What runs today

The CLI provides help/version and consent-gated `inventory` for exact installation selectors in approved Windows roots. `serde_json` parses bounded manifests, with dependencies pinned in Cargo.lock. M2.4 supplies current-session candidate snapshots; M2.5 adds explicitly scoped combined scans and text/JSON output. Competition verification remains outside M2. See the [inventory guide](installation-inventory.md).

Help/version inspect no installations. Inventory reads only disclosed root/ancestor metadata and selected manifests after consent. It does not enumerate processes or collect telemetry. Invalid-argument errors deliberately do not echo raw argument values, which might contain accidentally pasted private data.

M2.2 adds library types and a pure diagnostic evaluator. It takes preclassified observations and produces suggestions, explicit coverage concerns and `NotIssued` eligibility. It contains no real identity catalog/matcher. See the [shared-contract guide](shared-contracts.md).

## Prerequisites

- Git and the Rust toolchain manager, rustup.
- Rust 1.99.1, pinned in `rust-toolchain.toml`, with rustfmt and Clippy.
- On the normal Windows MSVC toolchain, the linker and Windows SDK supplied by the Visual Studio C++ build tools.
- Alternatively, an appropriately installed Windows GNU toolchain. The initial local checks used `x86_64-pc-windows-gnu`; the hosted Windows workflow passed separately, and local MSVC validation was not performed.

Follow the official [Rust installation guide](https://doc.rust-lang.org/book/ch01-01-installation.html) and [Windows toolchain guidance](https://rust-lang.github.io/rustup/installation/windows.html). Do not disable certificate checks or change Windows security policy to install build dependencies.

## Build from a normal Rust installation

Clone the repository, enter its root, then explicitly install the pinned toolchain before running Cargo:

```powershell
rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy
cargo fetch --locked
cargo build --locked --offline
```

Toolchain installation and the first locked dependency fetch need network access if not cached. Subsequent build/test commands work offline. Commit Cargo.lock with this application so contributors use the reviewed dependency graph.

## Run and expected behavior

```powershell
.\target\debug\anti-ai-guard.exe --help
.\target\debug\anti-ai-guard.exe --version
```

| Arguments | Behavior | Exit status |
| --- | --- | --- |
| No arguments | Print help to stdout | 0 |
| `--help` or `-h` | Print help to stdout | 0 |
| `--version` or `-V` | Print `anti-ai-guard 0.1.0-alpha.1` to stdout | 0 |
| Unsupported option or bare `scan` without consent/scope | Print a generic usage error to stderr, without echoing the arguments | 2 |
| Extra global help/version arguments, duplicate flags or invalid collection scope | Reject before collection | 2 |

Help/version need no consent. Inventory requires `inventory --consent` and explicit `--bin-root PATH`, `--npm-prefix PATH` and/or `--include-path` scope. Missing consent/invalid scope returns 2 before collection; partial/unavailable coverage returns 3; finished selector inspection returns 0. The [inventory guide](installation-inventory.md) explains scope and remaining limits.

## Required checks for each feature chunk

```powershell
cargo fmt --all -- --check
cargo clippy --locked --offline --all-targets -- -D warnings
cargo build --locked --offline
cargo test --locked --offline
```

The suite now has 18 collector/provider unit tests, 12 CLI tests, 19 shared-contract tests, seven report tests, seven Windows filesystem fixtures and one controlled native process test: 64 passing checks in hosted CI. One ignored helper is launched only by that native process test. They are fixture/contract evidence, not completed pinned-tool desktop trials.

Nineteen additional synthetic contract tests cover inventory versus execution, candidate versus supported assessments, rule approval, legitimate runtimes, process correlation, disappearing processes, scope integrity and incomplete coverage. Run them separately with `cargo test --locked --offline --test contracts`.

Run `cargo run --locked --offline --example diagnostic` for the hard-coded example. It prints inventory/review suggestions, incomplete coverage and no issued eligibility. The example never inspects the host. For real explicitly selected scopes, use the implemented scan command described in the [report guide](diagnostic-reports.md).

The GitHub workflow fetches locked dependencies, then runs these commands and the synthetic example through the [validation harness](../scripts/validate-windows.ps1) on `windows-2022` and `windows-2025`. [The initial M2.1 feature-commit run passed](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40752729560). A hosted Windows runner does not establish detection support on Windows 10/11 desktop configurations. Check the actual workflow result for subsequent changes; local success is not a CI pass.

## Local tooling and the rustup crash observed during setup

The initial development environment had no Rust or Visual Studio C++ toolchain. An isolated GNU Rust installation was placed under ignored `.tools/`, without changing the system PATH. `.tools/` is optional and is never committed.

A `rustup.exe` application-error dialog was observed while the toolchain manager was being used. Its root cause is unresolved. The installed compiler, Cargo, formatter and Clippy were invoked directly with the local toolchain environment, and the build/tests succeeded. This does not establish that the rustup crash itself is fixed.

If that isolated installation exists, this PowerShell session setup uses the installed toolchain binaries directly:

```powershell
$projectRoot = (Get-Location).Path
$toolchainBin = Join-Path $projectRoot '.tools\rustup\toolchains\1.99.1-x86_64-pc-windows-gnu\bin'
$env:CARGO_HOME = Join-Path $projectRoot '.tools\cargo'
$env:RUSTUP_HOME = Join-Path $projectRoot '.tools\rustup'
$env:PATH = $toolchainBin + ';' + $env:PATH
$env:RUSTC = Join-Path $toolchainBin 'rustc.exe'
$env:RUSTDOC = Join-Path $toolchainBin 'rustdoc.exe'
```

Run it from the repository root, then use the Cargo check commands above. These settings apply to the current terminal session. Contributors with a normal Rust installation do not need this setup. The compiled executable can run directly without invoking rustup.

## Verification performed for M2.1

- Compiler: rustc 1.99.1; Cargo 1.99.1; target `x86_64-pc-windows-gnu`.
- Local host reports Windows display version 25H2 and build 26200. Exact OS inspection via CIM was unavailable in the restricted development session; no Windows 10 test run was performed.
- Debug build, formatting, strict Clippy and all seven binary integration tests passed locally.
- The hosted Windows workflow for feature commit `72d62a4` completed successfully, including its formatting, lint, build and test steps.
- Historical M2.1 evidence is not a current support claim. Candidate collectors/reports now exist; full real-tool desktop, strong-identity, privilege and runtime-performance validation remains pending.

## M2.3 verification and environment notes

Local Windows GNU format, strict Clippy and build passed. Eleven collector unit tests, nine CLI tests and nineteen contract tests passed locally. Seven filesystem integration fixtures are locally blocked by this session denying ancestor-directory metadata access (Windows error 5); they are not local passes. [Hosted Windows CI passed all 46 tests](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40925592116) for source commit 5a8f937, including the seven filesystem fixtures. No real assistant or personal root was scanned.

Local Cargo fetching also hit the previously observed Windows Schannel credential failure. An ignored loopback helper under .tools fetched only official crates.io index/archive resources via Node HTTPS with certificate verification enabled. Cargo retained registry checksums and a normal crates.io lockfile. Its cache/source configuration is local-only; normal contributors and hosted CI use `cargo fetch --locked` directly. No certificate checks were disabled.

## Independent testing and issue reports

M2.2 local verification used the same pinned GNU toolchain: formatting, strict Clippy, build, all 26 integration tests (7 CLI + 19 contracts) and the synthetic example passed. [Hosted Windows CI for c3b9c1a passed those checks and the example](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40757983515). The tests use preclassified fixtures, not real approved catalog entries. The 33 scanner scenarios remain NOT RUN; no real device detection or new platform validation follows from these results.

Build/run this chunk before testing future features. For a problem, include the commit, toolchain/target, Windows version/build, redacted invocation, expected versus actual behavior, stdout/stderr and exit code. For the observed rustup dialog, report it as a toolchain-manager issue separately from CLI failures.

Do not attach real credentials, raw participant inventories or personal files. The existing [scanner acceptance plan](acceptance-plan.md) applies when the relevant collectors are implemented; its 33 planned cases remain unexecuted.

## Current commands and validation gates

Use `processes --consent [--json]` or `scan --consent --processes [--bin-root PATH] [--npm-prefix PATH] [--include-path] [--json]`. Choose explicit roots; there is no automatic whole-device scan. See the [report guide](diagnostic-reports.md) and [M2.6 results](validation/results.md). The local environment passed 57 selected tests, with seven installation filesystem cases blocked. Direct .ps1 harness execution is blocked by local script policy; no policy change or bypass is recommended or applied. Hosted server fixtures do not establish Windows 10/11 desktop support.

## Issue maintenance checks (2026-10-30)

[Source 8bae7e6 CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40938599135)
passed on Windows Server 2022/2025 (74 tests/doctests each, one controlled-child
helper ignored) and Ubuntu (65 tests/doctests). All jobs check formatting, strict
Clippy, build, synthetic examples and rustdoc with warnings denied. Linux remains
shared-code validation only. Local selected suites passed 65 tests and two
doctests; the seven filesystem fixtures and direct script restrictions described
above remain local limitations. Shared golden-file tests now run on every OS.

## Schema v2 maintenance validation

[Source 4a68d19 CI](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40940449420)
passed: 82 tests/doctests per Windows Server runner, one controlled-child helper
ignored; 72 on Ubuntu. Local selected checks passed 73 tests and two doctests.
The v1 DTO-only refactor preserved both goldens exactly in 714d603. The separate
v2 commit intentionally updates FILETIME encoding and adds operational_gaps.
The CLI requires schema-aware consumers; validation-artifact schemas are unchanged.
Desktop acceptance and trusted strong identity remain separate incomplete gates.
