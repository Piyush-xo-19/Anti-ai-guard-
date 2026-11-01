# Pinned native artifact identity (M2 pilot)

Implemented 2026-11-01. This is a narrow, offline diagnostic matcher, not a claim
that every version, packaging method, renamed program or coding behavior is known.
[Boot identity](boot-identity.md) | [M2 completion](m2-completion.md)

## Trust root and initial scope

[catalog/windows-artifacts.json](../catalog/windows-artifacts.json) is compiled
into the scanner. It initially contains only Codex CLI 0.166.0, Windows x64,
with its exact executable size and SHA-256 from the official OpenAI GitHub release
API. The lab download independently verified the bytes against that pin. Catalog
changes require repository review; there is no participant-provided catalog flag,
runtime network lookup, automatic update or acceptance of arbitrary upstream
rule strings. This trusts the reviewed release provenance and SHA-256, not an
unrelated writable package name, pathname or publisher-looking metadata.

The validator bounds catalog size/rows/identifiers, checks the schema, architecture,
approval, exact official URL shapes and digest encoding, and rejects duplicate
rules/digests. Unsupported versions and the Claude/OpenCode variants have no strong
pin yet and remain candidates. Do not treat this pilot as completion of that matrix.

## Binding a digest to the actual process image

1. Consent, current-session filtering and known native-name selection still happen
   before enrichment. Unknown/generic interpreters are not opened for hashing.
2. First check the selected tool family has a reviewed pin; unpinned selectors skip all image file I/O. Catalog errors remain visible. Only pinned families request QUERY_INFORMATION in addition to limited query/synchronize, with a
   limited-query fallback if denied. No VM read, write, inject, suspend or terminate
   right is requested by the scanner. Ring 3 and the current token are retained.
3. Open the selected image on a fixed local drive without following reparse points.
   Retain ancestor and final-file no-write/no-delete-sharing guards throughout the
   operation. The final file handle includes read/execute/attribute/synchronize
   access for the OS comparison; that access does not launch the file.
4. If no reviewed pin has exactly this size, read no executable content and leave
   the observation a candidate. Otherwise require a native image-file mapping
   comparison against the opened file handle, and a negative comparison against
   the unrelated scanner process. A same-path file or same-bytes copy is insufficient.
5. Stream SHA-256 through Windows CNG with 64 KiB buffers, cooperative cancellation
   and the existing process time budget. Pins are capped at 512 MiB per executable;
   all image reads share a 1 GiB invocation budget. No artifact hash cache is used.
6. Recheck native file/image binding and liveness while guards and the original
   process handle remain held. Only a complete digest match can produce
   Corroborated evidence with the compiled diagnostic-closure rule reference.
   Missing boot/start identity still prevents stable-instance closure suggestions.

The native comparison uses NtQueryInformationProcess class 44 with a file handle
input. It is an **experimental native compatibility capability**, not an officially
documented Win32 identity contract. [Microsoft warns that the native query may
change](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryinformationprocess).
The positive/negative controls, exact-success checks and file guards fail closed
on unsupported behavior. Access failures or unavailable identity checks stay
candidates with explicit gaps; no unsupported mapping result counts as proof.
Windows 10 and additional desktop builds still require trials.

This identifies the process's original mapped executable artifact; it does not
attest memory integrity or rule out injection/hollowing, an administrator-controlled
OS, or a separate tool calling APIs. There is no automatic enforcement or eligibility.
The existing selected-name boundary still misses arbitrary renamed/custom agents.

## Reports and compatibility

Schema v2 gains optional `artifact_rule_id` and `artifact_catalog_version` on
findings with an exact approved catalog reference. They are omitted otherwise;
arbitrary upstream strings remain suppressed. Text shows the same approved IDs.
The obsolete `no-trusted-strong-identity-catalog` limitation is replaced by
`strong-identity-limited-to-reviewed-native-artifact-pins`, an extensible diagnostic
code change. The synthetic goldens intentionally reflect that limitation change;
the schema version and action semantics are unchanged. Artifact hashing is bounded
internally; the existing metadata_bytes metric remains inventory metadata bytes,
not a claim about total image hashing I/O.

## Controlled evidence and reproduction

Default tests use self-built benign processes, CNG standard vectors, identical-file
copy rejection, a same-name benign image padded to the exact pinned size, budget/
cancellation checks, allowlisted report IDs and malformed catalog cases. They do
not download or execute real assistants. The ordinary native child helper and a
separate real-tool lab test are intentionally ignored in automatic discovery.

The explicit real-tool test requires both GUARD_PINNED_LAB_CONSENT=yes and an
absolute GUARD_PINNED_LAB_ARTIFACT path to the official pinned executable. It hashes
and verifies the guarded file before creating a process. It creates only that
owned process suspended with a fixed --version command, observes its image, then
resumes it and waits; timeout cleanup can target only that test-owned handle.
No paid/model request, installation, user-profile scan or collector process-control
operation is performed. This suspended-image trial demonstrates live image identity,
not interactive assistant use, recall, or coverage of every installation variant.

```powershell
$env:GUARD_PINNED_LAB_CONSENT = 'yes'
$env:GUARD_PINNED_LAB_ARTIFACT = 'C:\approved\lab\codex.exe'
try {
    cargo test --locked --offline --lib artifact::pinned_codex_native_lab -- --ignored --exact --nocapture
} finally {
    Remove-Item Env:GUARD_PINNED_LAB_CONSENT
    Remove-Item Env:GUARD_PINNED_LAB_ARTIFACT
}
```

Download the exact artifact named in the compiled catalog into a consenting lab
scope and independently check its official provenance. Never substitute a copied
filename as ground truth. Keep the artifact and real reports ignored/private;
public evidence contains only reviewed status/platform/catalog summaries.
