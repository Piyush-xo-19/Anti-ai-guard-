# Building and testing the CLI foundation

Updated: 2026-10-29 | Implemented scope: M2.1 only

[Public status](../PROJECT_STATUS.md) | [M2 plan](m2-plan.md) | [Contributing](../CONTRIBUTING.md)

## What runs today

The `anti-ai-guard` CLI displays help or its package version and rejects unsupported arguments. It has no third-party Rust dependencies. There is no scan command, process collector, installation inventory or competition verification yet.

The executable reads its command-line arguments; it does not enumerate processes, inspect installation roots or collect telemetry. Invalid-argument errors deliberately do not echo raw argument values, which might contain accidentally pasted private data.

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
cargo build --locked --offline
```

The rustup installation step needs network access if components are not cached. The current dependency-free Cargo build/test commands work offline after that setup. Commit `Cargo.lock` with this application so contributors use the reviewed lockfile.

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
| Unsupported option or `scan` | Print a generic usage error to stderr, without echoing the arguments | 2 |
| More than one argument, including after help/version | Reject the invocation | 2 |

No consent/scope interface is needed for these non-inspection commands. Any future command that inspects the device must introduce the agreed consent and scope flow before collection is enabled.

## Required checks for each feature chunk

```powershell
cargo fmt --all -- --check
cargo clippy --locked --offline --all-targets -- -D warnings
cargo build --locked --offline
cargo test --locked --offline
```

Seven binary integration tests cover default help, aliases, version consistency, error status/streams, an unavailable scan command, extra arguments and sensitive-argument redaction. They exercise the compiled executable. These are CLI-contract tests, not scanner acceptance results.

The GitHub workflow runs these commands on `windows-latest`. [The initial feature-commit run passed](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40752729560). A hosted Windows runner does not establish detection support on Windows 10/11 desktop configurations. Check the actual workflow result for subsequent changes; local success is not a CI pass.

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
- Real assistant inventory, process visibility, permission scopes, Windows 10 compatibility and runtime performance remain untested because their collectors are not implemented.

## Independent testing and issue reports

Build/run this chunk before testing future features. For a problem, include the commit, toolchain/target, Windows version/build, redacted invocation, expected versus actual behavior, stdout/stderr and exit code. For the observed rustup dialog, report it as a toolchain-manager issue separately from CLI failures.

Do not attach real credentials, raw participant inventories or personal files. The existing [scanner acceptance plan](acceptance-plan.md) applies when the relevant collectors are implemented; its 33 planned cases remain unexecuted.
