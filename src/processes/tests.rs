use super::*;
use std::collections::VecDeque;

struct Fake {
    rows: VecDeque<Row>,
    captures: usize,
    queries: usize,
    failure: Option<GapReason>,
    metadata_failure: Option<GapReason>,
    start: Option<u64>,
    image: Option<PathBuf>,
    exited: bool,
    boot_id: Option<String>,
}
impl Fake {
    fn new() -> Self {
        Self {
            rows: VecDeque::from([
                Row {
                    pid: 1,
                    basename: "codex.exe".into(),
                },
                Row {
                    pid: 2,
                    basename: "claude.exe".into(),
                },
                Row {
                    pid: 3,
                    basename: "node.exe".into(),
                },
            ]),
            captures: 0,
            queries: 0,
            failure: None,
            metadata_failure: None,
            start: Some(10),
            image: Some(PathBuf::from(if cfg!(windows) {
                r"C:\fixture\codex.exe"
            } else {
                "/fixture/codex.exe"
            })),
            exited: false,
            boot_id: None,
        }
    }
}
impl Provider for Fake {
    fn capture(&mut self) -> Result<Context, GapReason> {
        self.captures += 1;
        if let Some(reason) = self.failure {
            Err(reason)
        } else {
            Ok(Context {
                checkpoint: 1000,
                boot_id: self.boot_id.clone(),
                session: 7,
            })
        }
    }
    fn next(&mut self) -> Result<Option<Row>, GapReason> {
        Ok(self.rows.pop_front())
    }
    fn session(&mut self, pid: u32) -> Result<u32, GapReason> {
        Ok(if pid == 2 { 8 } else { 7 })
    }
    fn metadata(
        &mut self,
        _: u32,
        _budget: &mut EnrichmentBudget<'_>,
    ) -> Result<Metadata, GapReason> {
        self.queries += 1;
        if let Some(reason) = self.metadata_failure {
            return Err(reason);
        }
        Ok(Metadata {
            identity: None,
            start: self.start,
            image: self.image.clone(),
            state: if self.exited {
                ProcessState::ExitedDuringEnrichment
            } else {
                ProcessState::AliveAtSnapshot
            },
            gaps: vec![],
        })
    }
}
fn request(limits: Limits) -> ProcessRequest {
    ProcessRequest::new(true, ScopeId(1), limits).unwrap()
}
fn run(fake: &mut Fake) -> ProcessResult {
    collect_with(
        &request(Limits::default()),
        &AtomicBool::new(false),
        None,
        fake,
    )
}

#[test]
fn consent_cancel_and_zero_budgets_precede_native_capture() {
    assert!(ProcessRequest::new(false, ScopeId(1), Limits::default()).is_err());
    assert!(ProcessRequest::new(true, ScopeId(0), Limits::default()).is_err());
    let mut fake = Fake::new();
    let cancelled = collect_with(
        &request(Limits::default()),
        &AtomicBool::new(true),
        None,
        &mut fake,
    );
    assert_eq!(fake.captures, 0);
    assert_eq!(cancelled.execution, ScanExecution::Cancelled);
    for limits in [
        Limits {
            max_entries: 0,
            ..Limits::default()
        },
        Limits {
            max_candidates: 0,
            ..Limits::default()
        },
        Limits {
            max_elapsed: Duration::ZERO,
            ..Limits::default()
        },
    ] {
        let result = collect_with(&request(limits), &AtomicBool::new(false), None, &mut fake);
        assert_eq!(fake.captures, 0);
        assert!(result.coverage.gaps.contains(&GapReason::BudgetExceeded));
    }
}

#[test]
fn current_session_candidates_do_not_flag_interpreters_or_other_sessions() {
    let mut fake = Fake::new();
    let result = run(&mut fake);
    assert_eq!(result.findings.len(), 1);
    assert_eq!(fake.queries, 1);
    assert_eq!(result.skipped_other_sessions, 1);
    assert_eq!(
        result.findings[0].observation.identity.class,
        EvidenceClass::Candidate
    );
    assert_eq!(result.findings[0].observation.identity.rule_match, None);
    assert!(result
        .coverage
        .gaps
        .contains(&GapReason::MissingBootIdentity));
    assert_eq!(
        crate::policy::suggest_action(&result.findings[0].observation).action,
        crate::policy::SuggestedAction::RequestReview
    );
    for name in [
        "node.exe",
        "npm.exe",
        "npx.exe",
        "python.exe",
        "ollama.exe",
        "git.exe",
        "echo codex.exe",
        "notcodex.exe",
    ] {
        assert_eq!(tool_id(std::ffi::OsStr::new(name)), None);
    }
}

#[test]
fn failed_snapshot_is_not_empty_success() {
    let mut fake = Fake::new();
    fake.failure = Some(GapReason::AccessDenied);
    let result = run(&mut fake);
    assert!(result.findings.is_empty());
    assert_eq!(result.execution, ScanExecution::Failed);
    assert_eq!(result.coverage.state, CoverageState::Unavailable);
    // Capture never supplied context: report the real failure, not an inferred
    // boot-capability verdict based on the fake provider's default value.
    assert_eq!(result.coverage.gaps, vec![GapReason::AccessDenied]);
}

#[test]
fn boot_capability_gap_does_not_depend_on_snapshot_contents() {
    for boot in [None, Some(" "), Some("fixture.boot-1")] {
        for basename in [None, Some("node.exe"), Some("codex.exe")] {
            let mut fake = Fake::new();
            fake.boot_id = boot.map(str::to_owned);
            fake.rows = basename
                .into_iter()
                .map(|name| Row {
                    pid: 1,
                    basename: name.into(),
                })
                .collect();
            let result = run(&mut fake);
            let missing = boot.is_none_or(|id| id.trim().is_empty());
            assert_eq!(
                result.coverage.gaps,
                if missing {
                    vec![GapReason::MissingBootIdentity]
                } else {
                    vec![]
                }
            );
            assert_eq!(
                result.coverage.state,
                if missing {
                    CoverageState::Partial
                } else {
                    CoverageState::Inspected
                }
            );
            assert_eq!(
                result.execution,
                if missing {
                    ScanExecution::Partial
                } else {
                    ScanExecution::Finished
                }
            );
            assert_eq!(
                result.findings.len(),
                usize::from(basename == Some("codex.exe"))
            );
            assert_eq!(fake.queries, result.findings.len());
        }
    }
}

#[test]
fn exited_denied_and_changed_processes_remain_distinct() {
    for reason in [
        GapReason::ProcessExited,
        GapReason::AccessDenied,
        GapReason::ProcessIdentityChanged,
    ] {
        let mut fake = Fake::new();
        fake.metadata_failure = Some(reason);
        let result = run(&mut fake);
        assert!(result.coverage.gaps.contains(&reason));
        if reason == GapReason::AccessDenied {
            assert_eq!(result.findings.len(), 1);
            assert!(result
                .coverage
                .gaps
                .contains(&GapReason::MissingStartIdentity));
        } else {
            assert!(result.findings.is_empty());
        }
    }
    let mut fake = Fake::new();
    fake.exited = true;
    let result = run(&mut fake);
    assert_eq!(
        crate::policy::suggest_action(&result.findings[0].observation).action,
        crate::policy::SuggestedAction::RecheckProcess
    );
}

#[test]
fn pid_reuse_and_image_races_cannot_reuse_snapshot_identity() {
    let mut newer = Fake::new();
    newer.start = Some(1001);
    let mut replaced = Fake::new();
    replaced.image = Some(PathBuf::from("replacement.exe"));
    for mut fake in [newer, replaced] {
        let result = run(&mut fake);
        assert!(result.findings.is_empty());
        assert!(result
            .coverage
            .gaps
            .contains(&GapReason::ProcessIdentityChanged));
    }
}

#[test]
fn bounded_enumeration_discloses_truncation() {
    let result = collect_with(
        &request(Limits {
            max_entries: 1,
            ..Limits::default()
        }),
        &AtomicBool::new(false),
        None,
        &mut Fake::new(),
    );
    assert_eq!(result.enumerated, 1);
    assert!(result.coverage.gaps.contains(&GapReason::BudgetExceeded));
}

#[test]
fn installation_location_link_never_upgrades_process_identity() {
    let path = if cfg!(windows) {
        PathBuf::from("C:/fixture/")
    } else {
        PathBuf::from("/fixture")
    };
    let req = inventory::InventoryRequest::new(
        true,
        vec![inventory::InventoryRoot {
            path,
            kind: inventory::RootKind::BinDirectory,
        }],
        inventory::Limits::default(),
    )
    .unwrap();
    let installed = inventory::InventoryResult {
        scopes: vec![],
        coverage: vec![],
        execution: ScanExecution::Finished,
        probes: 1,
        bytes_read: 0,
        findings: vec![inventory::InstallationFinding {
            selector: "codex.exe",
            package_version: None,
            observation: Observation {
                scope: ScopeId(1),
                source: ObservationSource::ExecutableArtifact,
                kind: ObservationKind::Installation,
                identity: IdentityAssessment {
                    tool: Some(ToolIdentity {
                        id: "codex-cli".into(),
                        category: ToolCategory::CodingAssistant,
                    }),
                    class: EvidenceClass::Candidate,
                    rule_match: None,
                },
            },
        }],
    };
    let result = collect_with(
        &request(Limits::default()),
        &AtomicBool::new(false),
        Some(InventoryLink {
            request: &req,
            result: &installed,
        }),
        &mut Fake::new(),
    );
    assert_eq!(result.findings[0].installation_indicator, Some(0));
    assert_eq!(
        result.findings[0].observation.identity.class,
        EvidenceClass::Candidate
    );
    assert!(result.findings[0].observation.identity.rule_match.is_none());
}

#[test]
fn exact_entry_and_candidate_limits_do_not_claim_truncation() {
    for max_candidates in [1, 2] {
        for extra in [false, true] {
            let mut fake = Fake::new();
            fake.rows = VecDeque::from([Row {
                pid: 1,
                basename: "codex.exe".into(),
            }]);
            if extra {
                fake.rows.push_back(Row {
                    pid: 3,
                    basename: "node.exe".into(),
                });
            }
            let result = collect_with(
                &request(Limits {
                    max_entries: 1,
                    max_candidates,
                    ..Limits::default()
                }),
                &AtomicBool::new(false),
                None,
                &mut fake,
            );
            assert_eq!(
                result.coverage.gaps.contains(&GapReason::BudgetExceeded),
                extra
            );
            assert_eq!(result.enumerated, 1);
            assert_eq!(result.queried, 1);
        }
    }
    let mut fake = Fake::new();
    fake.rows = VecDeque::from([Row {
        pid: 1,
        basename: "codex.exe".into(),
    }]);
    let result = collect_with(
        &request(Limits {
            max_entries: 10,
            max_candidates: 1,
            ..Limits::default()
        }),
        &AtomicBool::new(false),
        None,
        &mut fake,
    );
    assert!(!result.coverage.gaps.contains(&GapReason::BudgetExceeded));
    assert_eq!(result.findings.len(), 1);
    assert!(result
        .coverage
        .gaps
        .contains(&GapReason::MissingBootIdentity));
}

#[test]
fn available_boot_identity_is_attached_without_fabricating_a_fallback() {
    let mut fake = Fake::new();
    fake.boot_id = Some("fixture.boot-1".into());
    let result = run(&mut fake);
    assert!(!result
        .coverage
        .gaps
        .contains(&GapReason::MissingBootIdentity));
    let ObservationKind::Process { instance, .. } = &result.findings[0].observation.kind else {
        panic!("process fixture")
    };
    assert_eq!(instance.boot_id.as_deref(), Some("fixture.boot-1"));
    assert!(instance.has_stable_identity());
    let fallback = run(&mut Fake::new());
    assert!(fallback
        .coverage
        .gaps
        .contains(&GapReason::MissingBootIdentity));
}
