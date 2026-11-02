# Frozen native artifact lab scope

Updated: 2026-11-02 (Asia/Calcutta). These exact x64 versions are lab pins, not
claims about the latest release. The [compiled catalog](../../catalog/windows-artifacts.json)
records executable sizes, SHA-256 digests and official acquisition URLs.
Catalog version: windows-artifact-pins-2026-11-02-v2. A pin permits Corroborated
diagnostic identity with guarded live-image binding, never eligibility.

| Tool | Version | Acquisition provenance |
| --- | --- | --- |
| Codex CLI | 0.166.0 | Executable digest independently matched the official OpenAI GitHub release asset digest |
| Claude Code | 2.1.316 | Executable digest matched a detached-signature-verified official manifest |
| OpenCode 1 | 1.19.5 | Official GitHub ZIP digest independently matched, then executable extracted and hashed |
| OpenCode 2 | 2.2.5 | Official docs download endpoint resolved to a versioned official HTTPS ZIP; locally computed hashes only |

Claude [manifest](https://downloads.claude.ai/claude-code-releases/2.1.316/manifest.json)
and its .sig were verified using the published signing fingerprint
31DDDE24DDFAB679F42D7BD2BAA929FF1A7ECACE, in an isolated lab keyring. The signature
was valid; system keyring/trust settings were not changed. See the
[official verification procedure](https://code.claude.com/docs/en/setup#verify-the-manifest-signature).

OpenCode 1 ZIP SHA-256:
09a6e693157f0887e504f2cf4357a8cbddc5ead5d0dffb9a269dcc47c48640f5.
OpenCode 2 ZIP SHA-256:
2f3bfd6b3527c60c2a882557b20ddc9ba6bdff4b5a8fe61c853da8fcfb3076e8.
The latter is a reproducibility fingerprint, **not** an independently published
upstream digest or verified signature. Its trust rests on official HTTPS acquisition
and repository review of the frozen pin. Do not describe all four pins as signed.
The [OpenCode 2 docs](https://opencode.ai/v2/docs/) identify standalone Windows CLI
downloads. Do not substitute OpenCode 1's GitHub release for generation 2.

Each native pin expands INST-01 and RUN-01 separately. The owned suspended-image
trial checks mapped executable identity while alive, with a ten-second collection
budget. It does not validate interactive coding, package runners, background modes,
renamed images, memory integrity or full acceptance/recall/performance targets.
Inactive scans must remain InventoryOnly; all results remain NotIssued.

Benign self-built images padded to each of the four exact sizes must remain
Candidate with no approved rule. A distinct byte-identical file must fail native
binding. OpenCode's shared filename may select both pins, but an exact digest
must identify the generation; package metadata alone cannot identify a process.

Only the consenting Windows 11 25H2 build 26200.9711 standard-user lab is currently
available. The maintainer deferred Windows 10 VM work on 2026-11-02. Elevated,
reboot/Fast Startup/restore and broader package/failure rows remain independent
requirements. No automatic elevation, reboot or system-wide installation occurs.
