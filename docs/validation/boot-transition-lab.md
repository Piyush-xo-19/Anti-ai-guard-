# Manual boot-transition checkpoint

This Windows-only opt-in test compares the existing self-header boot provider
across a deliberate power transition. It never restarts, sleeps, hibernates,
restores or elevates the computer. Save work before manually changing power state.
Windows 10 VM work remains deferred.

The baseline is reports/validation/boot-transition.local.json, ignored by Git.
It stores a random nonce, a comparison fingerprint, provider/schema identifiers
and a source-revision label. It stores no raw boot token, PID, path or arguments.
The fingerprint is a private comparison aid, not anonymization or attestation;
never upload the baseline. Use it only on the same Windows installation. OS-image
restore/cloning can reuse correlation data and needs separate interpretation.

## Record before the transition

Run from a clean committed checkout with the normal Rust environment configured:

```powershell
if (git status --porcelain) { throw 'Commit or set aside changes before recording evidence' }
$env:GUARD_BOOT_LAB_CONSENT = 'yes'
$env:GUARD_BOOT_LAB_PHASE = 'record'
$env:GUARD_BOOT_LAB_REVISION = (git rev-parse HEAD).Trim()
try {
    cargo test --locked --offline --lib lab::boot::transition -- --ignored --exact --nocapture
} finally {
    Remove-Item Env:GUARD_BOOT_LAB_CONSENT
    Remove-Item Env:GUARD_BOOT_LAB_PHASE
    Remove-Item Env:GUARD_BOOT_LAB_REVISION
}
```

Record refuses to overwrite an existing baseline. All directory ancestors are
opened without following reparse points and held without write/delete sharing.
Only the fixed repository reports/validation directory may be created. Compare
reads a regular bounded file without following a reparse point; unknown fields,
schema/provider values or malformed encodings are refused. Unavailable boot
capability fails the trial and cannot produce a fabricated result.

## Compare

Use the same clean source revision for record and compare; a changed revision
label is refused. The label is supplied by the operator, not remotely attested.
Run the same test with GUARD_BOOT_LAB_CONSENT=yes and GUARD_BOOT_LAB_PHASE set to:

| Phase | Actual action before comparison | Interpretation |
| --- | --- | --- |
| same-boot | None | Must retain the baseline identity |
| restart | Manually choose Windows Restart after saving work | Must differ from the baseline identity |
| resume | Manually sleep and resume | Expected to retain the same kernel boot |
| fast-startup | Explicitly arrange the declared shutdown/start configuration | Record Same/Changed as OBSERVED; no automatic acceptance |
| restore | Use a consenting disposable restore lab | Record Same/Changed as OBSERVED; no automatic acceptance |

For example, after an actual manual restart:

```powershell
$env:GUARD_BOOT_LAB_CONSENT = 'yes'
$env:GUARD_BOOT_LAB_PHASE = 'restart'
$env:GUARD_BOOT_LAB_REVISION = (git rev-parse HEAD).Trim()
try {
    cargo test --locked --offline --lib lab::boot::transition -- --ignored --exact --nocapture
} finally {
    Remove-Item Env:GUARD_BOOT_LAB_CONSENT
    Remove-Item Env:GUARD_BOOT_LAB_PHASE
    Remove-Item Env:GUARD_BOOT_LAB_REVISION
}
```

The phase is a human-supplied label. The helper cannot prove a restart occurred;
record the actual operation independently. Keep the baseline unchanged during a
trial. Before starting another power-transition trial, explicitly archive the
old private baseline after review; there is no automatic rotation/deletion.
Same-boot PASS alone never establishes restart, restore or Fast Startup behavior.
