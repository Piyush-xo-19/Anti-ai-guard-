#![cfg(windows)]
//! Real filesystem/CLI fixtures. No installed assistants or personal roots are scanned.
use anti_ai_guard::inventory::*;
use anti_ai_guard::model::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

static NEXT: AtomicUsize = AtomicUsize::new(0);
// The collector deliberately guards common ancestors against writers. Test
// setup must not mutate those shared ancestors while another fixture scans.
static SERIAL: Mutex<()> = Mutex::new(());
struct Fixture {
    path: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/inventory-fixtures");
        let path = base.join(format!(
            "{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }
    fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.path.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    fn collect(&self, kind: RootKind) -> InventoryResult {
        let request = InventoryRequest::new(
            true,
            vec![InventoryRoot {
                path: self.path.clone(),
                kind,
            }],
            Limits {
                max_elapsed: Duration::from_secs(10),
                ..Limits::default()
            },
        )
        .unwrap();
        collect(&request, &AtomicBool::new(false))
    }
    fn run(&self, flag: &str) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_anti-ai-guard"))
            .args(["inventory", "--consent", flag])
            .arg(&self.path)
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/inventory-fixtures");
        assert!(self.path.starts_with(base) && self.path.file_name().is_some());
        fs::remove_dir_all(&self.path).unwrap();
    }
}

#[test]
fn inventory_reads_exact_package_metadata_and_preserves_generation() {
    let _serial = SERIAL.lock().unwrap();
    let fixture = Fixture::new();
    for (relative, package) in [
        ("node_modules/@openai/codex/package.json", "@openai/codex"),
        (
            "node_modules/@anthropic-ai/claude-code/package.json",
            "@anthropic-ai/claude-code",
        ),
        ("node_modules/opencode-ai/package.json", "opencode-ai"),
        ("node_modules/@opencode/cli/package.json", "@opencode/cli"),
    ] {
        fixture.write(relative, format!("{{\"name\":\"{package}\",\"version\":\"1.2.3\",\"scripts\":{{\"postinstall\":\"must-not-run\"}}}}").as_bytes());
    }
    let result = fixture.collect(RootKind::WindowsNpmPrefix);
    assert_eq!(result.execution, ScanExecution::Finished);
    assert_eq!(result.findings.len(), 4);
    let ids: Vec<_> = result
        .findings
        .iter()
        .map(|f| f.observation.identity.tool.as_ref().unwrap().id.as_str())
        .collect();
    assert_eq!(
        ids,
        ["codex-cli", "claude-code", "opencode-v1", "opencode-v2"]
    );
    assert!(result
        .findings
        .iter()
        .all(|f| f.observation.identity.class == EvidenceClass::Candidate));
}

#[test]
fn no_recursive_discovery_or_tool_execution_occurs() {
    let _serial = SERIAL.lock().unwrap();
    let fixture = Fixture::new();
    fixture.write(
        "codex.cmd",
        b"@echo should-not-execute>execution-marker.txt\r\n",
    );
    fixture.write("nested/claude.exe", b"not-an-executable");
    fixture.write("personal.txt", b"claude codex opencode fake-private-token");
    fixture.write("node_modules/unrelated/package.json", b"malformed-not-json");
    let result = fixture.collect(RootKind::WindowsNpmPrefix);
    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.findings[0].selector, "codex.cmd");
    assert_eq!(result.bytes_read, 0);
    assert!(!fixture.path.join("execution-marker.txt").exists());
    assert_eq!(
        fs::read(fixture.path.join("personal.txt")).unwrap(),
        b"claude codex opencode fake-private-token"
    );
    assert_eq!(result.coverage[0].state, CoverageState::Inspected);
}

#[test]
fn invalid_or_oversized_manifests_report_partial_without_echoing_contents() {
    let _serial = SERIAL.lock().unwrap();
    let fixture = Fixture::new();
    fixture.write(
        "node_modules/@openai/codex/package.json",
        b"fake-secret malformed JSON",
    );
    fixture.write(
        "node_modules/@anthropic-ai/claude-code/package.json",
        &vec![b'x'; 65537],
    );
    let result = fixture.collect(RootKind::WindowsNpmPrefix);
    assert!(result.findings.is_empty());
    assert!(result.coverage[0]
        .gaps
        .contains(&GapReason::InvalidMetadata));
    assert!(result.coverage[0].gaps.contains(&GapReason::BudgetExceeded));
    let output = fixture.run("--npm-prefix");
    assert_eq!(output.status.code(), Some(3));
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!combined.contains("fake-secret"));
    assert!(!combined.contains(&fixture.path.to_string_lossy().to_string()));
}

#[test]
fn missing_root_does_not_become_a_complete_empty_scan() {
    let _serial = SERIAL.lock().unwrap();
    let fixture = Fixture::new();
    let request = InventoryRequest::new(
        true,
        vec![InventoryRoot {
            path: fixture.path.join("missing"),
            kind: RootKind::BinDirectory,
        }],
        Limits::default(),
    )
    .unwrap();
    let result = collect(&request, &AtomicBool::new(false));
    assert!(result.findings.is_empty());
    assert_eq!(result.coverage[0].state, CoverageState::Unavailable);
    assert_eq!(result.coverage[0].gaps, vec![GapReason::MissingRoot]);
}

#[test]
fn cli_outputs_scoped_inventory_and_empty_scope_warning() {
    let _serial = SERIAL.lock().unwrap();
    let fixture = Fixture::new();
    fixture.write("opencode.exe", b"fixture");
    let output = fixture.run("--bin-root");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Candidate | InventoryOnly"));
    assert!(text.contains("does not mean a clean device"));
    assert!(!text.contains(&fixture.path.to_string_lossy().to_string()));
}

fn junction(link: &Path, target: &Path) {
    let status = Command::new("cmd.exe")
        .args(["/d", "/c", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "fixture junction creation failed: {} {}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );
}

#[test]
fn junction_roots_and_intermediate_package_junctions_are_never_followed() {
    let _serial = SERIAL.lock().unwrap();
    let fixture = Fixture::new();
    let outside = Fixture::new();
    outside.write("codex.exe", b"outside-approved-root");
    let linked = fixture.path.join("linked");
    junction(&linked, &outside.path);
    let request = InventoryRequest::new(
        true,
        vec![InventoryRoot {
            path: linked,
            kind: RootKind::BinDirectory,
        }],
        Limits::default(),
    )
    .unwrap();
    let result = collect(&request, &AtomicBool::new(false));
    assert!(result.findings.is_empty());
    assert_eq!(result.coverage[0].gaps, vec![GapReason::ScopeBoundary]);
    junction(&fixture.path.join("node_modules"), &outside.path);
    let result = fixture.collect(RootKind::WindowsNpmPrefix);
    assert!(result.findings.is_empty());
    assert!(result.coverage[0].gaps.contains(&GapReason::ScopeBoundary));
    // Remove test junction entries explicitly before RAII fixture cleanup.
    fs::remove_dir(fixture.path.join("linked")).unwrap();
    fs::remove_dir(fixture.path.join("node_modules")).unwrap();
    assert!(outside.path.join("codex.exe").exists());
}

#[test]
fn concurrent_writer_is_reported_as_sharing_conflict() {
    let _serial = SERIAL.lock().unwrap();
    let fixture = Fixture::new();
    fixture.write("codex.exe", b"fixture");
    let _writer = fs::OpenOptions::new()
        .write(true)
        .open(fixture.path.join("codex.exe"))
        .unwrap();
    let result = fixture.collect(RootKind::BinDirectory);
    assert!(result.findings.is_empty());
    assert!(result.coverage[0]
        .gaps
        .contains(&GapReason::SharingConflict));
}
