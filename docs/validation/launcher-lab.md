# Pinned npm launcher and headless snapshot lab

This opt-in Windows x64 lab runs the four frozen packages through their real npm
entry points using `npm exec --offline --yes=false --ignore-scripts --loglevel=silent -- <command> --version`.
It uses repository-local profiles without inherited credentials. The scanner
remains read-only; process creation and cleanup belong only to test setup.

## Preparation

Use the [frozen artifacts](native-artifacts.md) and the exact npm versions in the
[package acquisition record](npm-inventory-2026-11-02.json). Install the packages
with lifecycle scripts disabled into these disposable prefixes:

| Tool | Package/version | Prefix relative to repository |
| --- | --- | --- |
| Codex | @openai/codex@0.166.0 | .tools/lab/npm-codex-cli |
| Claude | @anthropic-ai/claude-code@2.1.316 | .tools/lab/npm-claude-code |
| OpenCode 1 | opencode-ai@1.19.5 | .tools/lab/npm-opencode-v1 |
| OpenCode 2 | @opencode/cli@2.2.5 | .tools/lab/npm-opencode-v2 |

For example, using a trusted installed Node/npm toolchain:

```powershell
node 'C:\Program Files\nodejs\node_modules\npm\bin\npm-cli.js' install --prefix .tools/lab/npm-codex-cli --ignore-scripts --no-audit --no-fund --save-exact '@openai/codex@0.166.0'
node scripts/prepare-launcher-lab.cjs --consent
```

Prepare all four prefixes before the second command. The setup helper does not download packages. It finalizes the existing
installations, verifying frozen dependency executable hashes and
reviewed postinstall script hashes before running those scripts. Only Claude and
OpenCode's selected postinstall scripts run, offline, with isolated home/config/
temp directories. Their final launcher hashes must match the compiled lab manifest.
This differs from the earlier scripts-disabled inventory trials. The npm executable
and Node interpreter are trusted lab tools, not identity claims about assistants.

OpenCode setup can choose a baseline CPU binary. Those possible setup targets
also have frozen hashes in the lab manifest and must be present and verified.
They are setup-only checks, not additional strong-identity pins or tested baseline
CPU support. Profiles and job ownership do not sandbox arbitrary software or
attest that a vendor made no network request; these are trusted pinned packages
running only a version query in a consenting lab.

## Run

After the normal [Rust toolchain setup](../development.md):

```powershell
$env:GUARD_LAUNCHER_LAB_CONSENT = 'yes'
$env:GUARD_LAUNCHER_LAB_TOOL = 'codex-cli' # or claude-code, opencode-v1, opencode-v2
try {
    cargo test --locked --offline --lib lab::package_launcher_lab -- --ignored --exact --nocapture
} finally {
    Remove-Item Env:GUARD_LAUNCHER_LAB_CONSENT
    Remove-Item Env:GUARD_LAUNCHER_LAB_TOOL
}
```

Default Node location is C:\Program Files\nodejs\node.exe; npm is resolved beside
it under node_modules/npm/bin/npm-cli.js. Set GUARD_LAUNCHER_LAB_NODE and
GUARD_LAUNCHER_LAB_NPM to absolute trusted toolchain paths when necessary. No raw
launcher output is published. Isolated failure logs remain under ignored .tools.

For the ordinary-runner negative, also set GUARD_LAUNCHER_LAB_CASE=benign-node
and use the codex-cli prefix, then run the same test. Remove that environment
variable afterwards. This uses the fixed command `npm exec --call "node --version"`
with the same offline/profile options. A bounded list of the test-owned job's
processes must show both npm's interpreter and its Node child alive before and
after collection; no owned process may be classified as an assistant. Only
job-member image metadata is queried for this control; paths are discarded.
It does not claim the rest of the machine has no assistants.

The test fills a private stdout pipe to its measured capacity. The real version
write then waits while the scanner observes the live image. No initial process
suspension or debugger is used. A helper waits for a parent handshake before
launching anything; the parent first assigns it to a private kill-on-close job.
The match must belong to this exact job, using IsProcessInJob, and must carry the
expected pinned rule. Normal completion drains stdout and verifies the version.
Cleanup can stop only the job/tree created by this test. The scanner receives no
process-control rights and has no access to these test helpers in release builds.

The helper and npm parent request CREATE_NO_WINDOW. Results demonstrate a real
package-launcher image snapshot without terminal-content inspection, under
controlled output backpressure. They are RUN-02 and headless RUN-03 subcases,
not interactive coding, daemon/server-mode behavior, launch-event recall,
performance targets, or a full desktop-support claim. Collection uses the
explicit ten-second library limit, not proof of the default two-second budget.

Relevant primary APIs: [pipes](https://learn.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-createpipe),
[job assignment](https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-assignprocesstojobobject),
[job membership](https://learn.microsoft.com/en-us/windows/win32/api/jobapi/nf-jobapi-isprocessinjob),
[npm exec](https://docs.npmjs.com/cli/v11/commands/npm-exec/).

## Real denied-directory control

The [recorded permission case](negative-controls-2026-11-03.json) used one newly
created empty directory under the ignored lab root. Its original Access DACL was
saved before adding a target-only deny for directory-list/read-attributes. A
try/finally restored that descriptor using a DirectorySecurity object with the
Access section explicitly marked for persistence. The restored descriptor was
compared with the original, inventory recovery was checked, and only that empty
owned fixture was removed. No existing participant or system directory was used.
This is manual lab setup; the scanner never changes ACLs.
