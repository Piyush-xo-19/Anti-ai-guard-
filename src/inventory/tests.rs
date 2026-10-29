use super::*;
use std::cell::RefCell;
use std::rc::Rc;

fn root() -> InventoryRoot {
    InventoryRoot {
        path: if cfg!(windows) {
            PathBuf::from("C:\\fixture-root")
        } else {
            PathBuf::from("/fixture-root")
        },
        kind: RootKind::WindowsNpmPrefix,
    }
}
fn request(limits: Limits) -> InventoryRequest {
    InventoryRequest::new(true, vec![root()], limits).unwrap()
}

struct FakeFs {
    calls: Rc<RefCell<Vec<&'static str>>>,
    denied: bool,
    malformed: bool,
}
impl InventoryFs for FakeFs {
    fn open_root(&mut self, _: &Path) -> Result<Box<dyn RootReader>, GapReason> {
        self.calls.borrow_mut().push("root");
        if self.denied {
            Err(GapReason::AccessDenied)
        } else {
            Ok(Box::new(FakeRoot {
                calls: self.calls.clone(),
                malformed: self.malformed,
            }))
        }
    }
}
struct FakeRoot {
    calls: Rc<RefCell<Vec<&'static str>>>,
    malformed: bool,
}
impl RootReader for FakeRoot {
    fn probe(&mut self, selector: &Selector, max_bytes: usize) -> Result<Probe, GapReason> {
        self.calls.borrow_mut().push(selector.relative);
        if let Some(package) = selector.package {
            let bytes = if self.malformed {
                b"not-json fake-secret".to_vec()
            } else {
                format!("{{\"name\":\"{package}\",\"version\":\"1.2.3-fixture\"}}").into_bytes()
            };
            if bytes.len() > max_bytes {
                return Err(GapReason::BudgetExceeded);
            }
            Ok(Probe::Manifest(bytes))
        } else if selector.relative == "codex.exe" || selector.relative == "opencode.exe" {
            Ok(Probe::Indicator)
        } else {
            Ok(Probe::Absent)
        }
    }
}
fn fake() -> FakeFs {
    FakeFs {
        calls: Rc::new(RefCell::new(vec![])),
        denied: false,
        malformed: false,
    }
}

#[test]
fn consent_and_scope_validation_precede_any_io() {
    assert_eq!(
        InventoryRequest::new(false, vec![], Limits::default()).unwrap_err(),
        RequestError::ConsentRequired
    );
    assert_eq!(
        InventoryRequest::new(true, vec![], Limits::default()).unwrap_err(),
        RequestError::InvalidScope
    );
    let mut relative = root();
    relative.path = PathBuf::from("relative-root");
    assert_eq!(
        InventoryRequest::new(true, vec![relative], Limits::default()).unwrap_err(),
        RequestError::InvalidScope
    );
    assert!(InventoryRequest::new(true, vec![root(); 33], Limits::default()).is_err());
}

#[test]
fn exact_package_selectors_keep_both_opencode_generations_separate() {
    let mut fs = fake();
    let result = collect_with(
        &request(Limits::default()),
        &AtomicBool::new(false),
        &mut fs,
    );
    assert_eq!(result.findings.len(), 6);
    for finding in &result.findings {
        assert_eq!(finding.observation.kind, ObservationKind::Installation);
        assert_eq!(finding.observation.identity.class, EvidenceClass::Candidate);
        assert_eq!(finding.observation.identity.rule_match, None);
        assert_eq!(
            crate::policy::suggest_action(&finding.observation).action,
            crate::policy::SuggestedAction::InventoryOnly
        );
    }
    let ids: Vec<_> = result
        .findings
        .iter()
        .map(|f| f.observation.identity.tool.as_ref().unwrap().id.as_str())
        .collect();
    assert!(ids.contains(&"opencode-v1"));
    assert!(ids.contains(&"opencode-v2"));
    assert!(ids.contains(&"opencode-unknown-generation"));
    assert_eq!(result.coverage[0].state, CoverageState::Inspected);
    assert_eq!(result.probes, 13);
}

#[test]
fn bin_scope_does_not_read_package_manifests() {
    let mut directory = root();
    directory.kind = RootKind::BinDirectory;
    let request = InventoryRequest::new(true, vec![directory], Limits::default()).unwrap();
    let mut fs = fake();
    let result = collect_with(&request, &AtomicBool::new(false), &mut fs);
    assert_eq!(result.probes, 9);
    assert_eq!(result.bytes_read, 0);
    assert!(fs
        .calls
        .borrow()
        .iter()
        .all(|s| !s.contains("package.json")));
}

#[test]
fn denied_root_is_unavailable_not_clean() {
    let mut fs = fake();
    fs.denied = true;
    let result = collect_with(
        &request(Limits::default()),
        &AtomicBool::new(false),
        &mut fs,
    );
    assert!(result.findings.is_empty());
    assert_eq!(result.execution, ScanExecution::Partial);
    assert_eq!(result.coverage[0].state, CoverageState::Unavailable);
    assert_eq!(result.coverage[0].gaps, vec![GapReason::AccessDenied]);
}

#[test]
fn cancellation_before_start_performs_no_filesystem_access() {
    let mut fs = fake();
    let result = collect_with(&request(Limits::default()), &AtomicBool::new(true), &mut fs);
    assert!(fs.calls.borrow().is_empty());
    assert_eq!(result.execution, ScanExecution::Cancelled);
    assert_eq!(result.coverage[0].gaps, vec![GapReason::Cancelled]);
}

#[test]
fn zero_probe_or_time_budget_performs_no_filesystem_access() {
    for limits in [
        Limits {
            max_probes: 0,
            ..Limits::default()
        },
        Limits {
            max_elapsed: Duration::ZERO,
            ..Limits::default()
        },
    ] {
        let mut fs = fake();
        let result = collect_with(&request(limits), &AtomicBool::new(false), &mut fs);
        assert!(fs.calls.borrow().is_empty());
        assert_eq!(result.probes, 0);
        assert_eq!(result.coverage[0].gaps, vec![GapReason::BudgetExceeded]);
    }
}

#[test]
fn probe_and_byte_limits_stop_with_explicit_gaps() {
    let mut fs = fake();
    let result = collect_with(
        &request(Limits {
            max_probes: 2,
            ..Limits::default()
        }),
        &AtomicBool::new(false),
        &mut fs,
    );
    assert_eq!(result.probes, 2);
    assert!(result.coverage[0].gaps.contains(&GapReason::BudgetExceeded));
    let result = collect_with(
        &request(Limits {
            max_manifest_bytes: 4,
            max_total_bytes: 4,
            ..Limits::default()
        }),
        &AtomicBool::new(false),
        &mut fs,
    );
    assert_eq!(result.bytes_read, 0);
    assert_eq!(result.findings.len(), 2);
    assert!(result.coverage[0].gaps.contains(&GapReason::BudgetExceeded));
}

#[test]
fn malformed_or_spoofed_metadata_never_becomes_a_confirmed_identity() {
    let mut fs = fake();
    fs.malformed = true;
    let result = collect_with(
        &request(Limits::default()),
        &AtomicBool::new(false),
        &mut fs,
    );
    assert_eq!(result.findings.len(), 2);
    assert!(result.coverage[0]
        .gaps
        .contains(&GapReason::InvalidMetadata));
    assert!(!format!("{result:?}").contains("fake-secret"));
    assert!(manifest_version(br#"{"name":"forged","version":"1.0"}"#, "@openai/codex").is_err());
    assert!(manifest_version(
        br#"{"name":"@openai/codex","version":"secret\nvalue"}"#,
        "@openai/codex"
    )
    .is_err());
}

#[test]
fn duplicate_roots_are_deduplicated_without_expanding_scope() {
    let request = InventoryRequest::new(true, vec![root(), root()], Limits::default()).unwrap();
    assert_eq!(request.roots.len(), 1);
    let result = collect_with(&request, &AtomicBool::new(false), &mut fake());
    assert_eq!(result.scopes.len(), 1);
}

#[cfg(windows)]
#[test]
fn windows_rejects_network_device_drive_roots_and_parent_traversal() {
    for path in [
        r"\\server\share\root",
        r"\\?\C:\root",
        r"C:\",
        r"C:\fixture\..\elsewhere",
        r"C:\fixture\file:stream",
        r"C:\fixture.\nested",
    ] {
        assert!(
            !windows::valid_root(Path::new(path)),
            "invalid Windows scope accepted"
        );
    }
}
