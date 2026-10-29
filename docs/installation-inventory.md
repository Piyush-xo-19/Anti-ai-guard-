# M2.3: bounded Windows installation inventory

Updated: 2026-10-29 | Installation candidates, not process detection

[Development](development.md) | [M2 plan](m2-plan.md) | [Coverage contract](coverage-matrix.md)

The `inventory` command checks fixed installation selectors in explicitly approved roots. It never launches discovered tools, runs package scripts, reads executable/script contents, searches recursively, installs software or makes model requests. A finding does not prove execution, inactivity elsewhere, integrity or competition eligibility.

## Run and consent

Build after the [toolchain setup](development.md):

```powershell
cargo fetch --locked
cargo build --locked --offline
.\target\debug\anti-ai-guard.exe inventory --help
.\target\debug\anti-ai-guard.exe inventory --consent --bin-root "C:\approved\bin"
.\target\debug\anti-ai-guard.exe inventory --consent --npm-prefix "C:\approved\npm-prefix"
```

Replace the example directories with roots you choose to inspect. Repeat the root flags as needed. Run from a normal terminal: there is no elevation request or access bypass. The current process token determines access; the program does not claim to have measured its elevation mode. Windows 10/11 are prototype targets; exact desktop lab and real-tool validation remain pending.

`--include-path` additionally opts into current PATH directories with the npm-prefix selectors. PATH is read only for that option, after explicit CLI consent. Empty, relative, drive-root, device, UNC, parent-traversing and over-budget scopes fail validation before collection. Mapped network/removable drives are rejected by the Windows adapter. No registry, home-folder, project-tree or whole-disk search occurs. Declare a known native executable directory yourself; for example, Claude documents a Windows launcher under the user's `.local\bin` directory. [Claude setup](https://code.claude.com/docs/en/setup)

| Exit code | Meaning |
| --- | --- |
| 0 | All requested fixed selectors inspected; not a clean-device assertion |
| 2 | Invalid arguments, missing consent or invalid scope; no collection |
| 3 | Partial/unavailable/cancelled coverage; inspect the printed reasons |

Absolute roots and raw arguments are not echoed. Output uses numbered roots and fixed relative selector names. No report file is written.

## Selectors and evidence

| Root kind | Fixed selectors | Candidate identity |
| --- | --- | --- |
| Both | `codex.exe`, `codex.cmd`, `codex.ps1` | Codex CLI |
| Both | `claude.exe`, `claude.cmd`, `claude.ps1` | Claude Code |
| Both | `opencode.exe`, `opencode.cmd`, `opencode.ps1` | OpenCode, unknown generation |
| npm prefix | `node_modules/@openai/codex/package.json` | Exact package name `@openai/codex` |
| npm prefix | `node_modules/@anthropic-ai/claude-code/package.json` | Exact package name `@anthropic-ai/claude-code` |
| npm prefix | `node_modules/opencode-ai/package.json` | OpenCode 1 |
| npm prefix | `node_modules/@opencode/cli/package.json` | OpenCode 2 |

Windows npm global packages reside under `{prefix}/node_modules`, with command links directly in the prefix. Package names come from the projects' installation documentation. A shared `opencode` command cannot identify its generation. [npm layout](https://docs.npmjs.com/cli/v11/configuring-npm/folders/), [Codex](https://github.com/openai/codex), [Claude](https://code.claude.com/docs/en/setup), [OpenCode 1](https://opencode.ai/docs/), [OpenCode 2](https://opencode.ai/v2/docs/)

Selected manifests are parsed with `serde_json`. Only recognized `name` and bounded `version` values leave the parser. Versions allow at most 64 ASCII alphanumeric/dot/plus/hyphen/underscore characters; this controls output size/control characters, not authenticity. Other JSON fields and parser error text are discarded. Invalid JSON, mismatched names or invalid versions produce an explicit metadata gap. A matching writable manifest remains a candidate, even if it describes a native binary.

All findings are M2.2 `Installation` observations with `Candidate` identity and no approved rule match. Policy A treats them as `InventoryOnly`. There is no executable hash/publisher check, package entry-point execution or production trusted identity catalog.

## Filesystem boundary

Only ordinary absolute local fixed-drive roots are supported. The adapter opens every ancestor and target with `FILE_FLAG_OPEN_REPARSE_POINT`, checks attributes on the resulting handle and retains directory handles without write/delete sharing through the read. Reparse points, including junctions, symlinks and cloud placeholders, are refused at roots, intermediate components and selected files. There is no fallback traversal.

Ancestor attributes are inspected, never ancestor directory contents. Guard handles also request read-data/list-directory access because Windows attribute-only opens do not enforce ordinary sharing exclusions. The collector does not actually enumerate directories or read executable/script data. Existing writers, insufficient access and unavailable ancestor metadata yield coverage gaps instead of weakening this boundary. Only regular files are accepted, and manifests are held read-only without write/delete sharing. A narrow `GetDriveTypeW` import checks the fixed-drive constraint; no privilege-changing API is used. [Windows open options](https://doc.rust-lang.org/std/os/windows/fs/trait.OpenOptionsExt.html), [CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)

| Budget | Default / hard limit |
| --- | --- |
| Roots | 32 before deduplication |
| Root path components | 32, without parent traversal |
| Candidate probes | 512 |
| Manifest size/read | 64 KiB each |
| Total manifest reads | 512 KiB |
| Elapsed time | Cooperative 2 seconds; library maximum 10 seconds |
| Recursive traversal | None; fixed selectors only |

Time/cancellation checks occur between bounded operations. A synchronous OS call can exceed the elapsed budget; this is not a hard I/O timeout or runtime latency promise. Oversized files are skipped before reading. Bytes returned before an I/O failure still count against the total read budget. Limits/cancellation produce explicit partial/cancelled results. The library accepts an `AtomicBool` cancellation flag; the CLI registers a best-effort Windows Ctrl+C/Break handler. It requests cooperative cancellation and a final partial/cancelled report when the console supports it. See [cancellation and timing](diagnostic-reports.md#cancellation-and-timing) for limits.

## Output and remaining scope

`InventoryResult` retains minimized scope labels, per-root coverage/gaps, findings, execution status and probe/byte counts. The CLI now uses the [M2.5 shared text/JSON projection](diagnostic-reports.md). Internal bounded package versions are omitted from public reports. Missing roots, access denied, sharing conflicts, reparse boundaries, invalid metadata, unsupported platforms/drives and exhausted budgets remain visible even if no tools are found.

An inspected root means only its fixed selectors were checked. Renamed executables, nested/version directories, package-manager databases, pnpm/Yarn stores, ephemeral runner caches, WSL/containers and other undeclared layouts are not validated. Linux returns unsupported coverage. [M2.4 snapshots](process-snapshots.md) provide current-session native-name candidates; continuous observation remains M3.

## Verification

Local Windows GNU format, strict Clippy and build passed. Eleven collector unit tests, nine CLI tests and nineteen shared-contract tests passed locally. Seven additional Windows filesystem fixture tests cover exact package generation, no recursion/tool launch, malformed/oversized manifests, missing roots, minimized CLI output, junction refusal and sharing conflicts.

The seven filesystem tests are **locally blocked** by the restricted development session denying ancestor attributes with Windows error 5. The collector reports `Unavailable [AccessDenied]`; these are not local passes. [Hosted Windows CI passed all 46 tests](https://github.com/Piyush-xo-19/Anti-ai-guard-/actions/runs/40925592116) for source commit 5a8f937, including these seven filesystem fixtures. Fixtures use only ignored `target/inventory-fixtures`; no installed assistant or personal root was scanned during development.

Fixture results cover portions of INST/SCOPE/NEG/PRIV/FAIL/SAFE requirements, not pinned real-tool installs across both desktop labs. All 33 original scanner acceptance rows remain NOT RUN as end-to-end lab scenarios. Check public status for the exact hosted CI result/commit.

PATH scope validation fails before any collection. Errors name a missing/oversized PATH, a one-based invalid entry index, or the 32 distinct root/selector limit without echoing entries. Empty entries (including a trailing semicolon), unresolved variables, relative/network/device/drive-root paths are rejected. Select explicit roots for long or mixed PATHs; no entries are silently skipped. Equivalent lexical separators/trailing separators are deduplicated before counting the limit, without resolving links.
