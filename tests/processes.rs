#![cfg(windows)]
//! Controlled self-built process fixture only; no genuine assistant is launched.
use anti_ai_guard::{model::*, processes::*};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::time::Duration;

#[test]
#[ignore = "only launched as a controlled subprocess by the native fixture test"]
fn fixture_child() {
    if std::env::var_os("GUARD_FIXTURE_CHILD").is_none() {
        return;
    }
    println!("GUARD_FIXTURE_READY");
    std::io::stdout().flush().unwrap();
    let _ = std::io::stdin().read(&mut [0u8; 1]);
}

struct Fixture {
    child: Child,
    path: PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        // Only this controlled fixture is stopped by test cleanup; collector code
        // never requests process-control rights or invokes terminate APIs.
        self.child.stdin.take();
        let _ = self.child.wait();
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/process-fixtures");
        assert!(self.path.starts_with(base));
        fs::remove_dir_all(&self.path).unwrap();
    }
}

#[test]
fn controlled_candidate_is_observed_without_retaining_paths_or_arguments() {
    controlled_candidate(false);
}

#[test]
fn a_same_name_same_size_benign_image_never_matches_the_artifact_pin() {
    controlled_candidate(true);
}

fn controlled_candidate(padded: bool) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/process-fixtures")
        .join(format!(
            "{}-{}",
            std::process::id(),
            if padded { "padded" } else { "normal" }
        ));
    fs::create_dir_all(&path).unwrap();
    let binary = path.join("codex.exe");
    fs::copy(std::env::current_exe().unwrap(), &binary).unwrap();
    if padded {
        let catalog: serde_json::Value =
            serde_json::from_str(include_str!("../catalog/windows-artifacts.json")).unwrap();
        let length = catalog["artifacts"][0]["bytes"].as_u64().unwrap();
        // Preserve the self-built fixture code; a zero-filled PE overlay makes
        // its file size equal the pin without executing any third-party code.
        fs::OpenOptions::new()
            .write(true)
            .open(&binary)
            .unwrap()
            .set_len(length)
            .unwrap();
    }
    let secret = "fixture-private-token-7c6";
    let child = Command::new(&binary)
        .args([
            "--ignored",
            "--exact",
            "fixture_child",
            secret,
            "--nocapture",
        ])
        .env("GUARD_FIXTURE_CHILD", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut fixture = Fixture { child, path };
    let stdout = fixture.child.stdout.take().unwrap();
    let mut ready = false;
    for line in BufReader::new(stdout).lines().take(20) {
        if line.unwrap().contains("GUARD_FIXTURE_READY") {
            ready = true;
            break;
        }
    }
    assert!(ready, "controlled subprocess did not become ready");
    let request = ProcessRequest::new(
        true,
        ScopeId(1),
        Limits {
            max_elapsed: Duration::from_secs(10),
            ..Limits::default()
        },
    )
    .unwrap();
    let result = collect(&request, &AtomicBool::new(false), None);
    if !result.findings.iter().any(|f|matches!(&f.observation.kind,ObservationKind::Process {instance,..} if instance.pid==fixture.child.id())) {
        println!("NATIVE_FIXTURE_DIAGNOSTIC state={:?} gaps={:?}",result.coverage.state,result.coverage.gaps);
    }
    let finding=result.findings.iter().find(|f|matches!(&f.observation.kind,ObservationKind::Process {instance,..} if instance.pid==fixture.child.id())).expect("controlled current-session candidate must be visible");
    assert_eq!(finding.observation.identity.class, EvidenceClass::Candidate);
    assert_eq!(finding.observation.identity.rule_match, None);
    assert!(
        matches!(&finding.observation.kind,ObservationKind::Process {instance,..} if instance.start_time.is_some())
    );
    let minimized = format!("{result:?}");
    assert!(!minimized.contains(secret));
    assert!(!minimized.contains(&fixture.path.to_string_lossy().to_string()));
    let ObservationKind::Process { instance, .. } = &finding.observation.kind else {
        panic!("process fixture")
    };
    assert_eq!(
        result
            .coverage
            .gaps
            .contains(&GapReason::MissingBootIdentity),
        !instance.has_boot_identity()
    );
}
