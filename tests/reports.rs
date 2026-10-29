use anti_ai_guard::{
    model::*,
    parse_args, policy, report,
    scan::{self, Metrics, ScanResult},
    Command,
};
use std::sync::atomic::AtomicBool;

fn fixture() -> ScanResult {
    let scope = ScopeId(1);
    let target = CoverageTarget {
        collector: Collector::InstallationInventory,
        scope,
    };
    ScanResult {
        diagnostic: policy::evaluate(Snapshot {
            scan_id: "scan-1234-1".into(),
            observed_at_ms: 1234,
            platform: Platform {
                os: OperatingSystem::Windows11,
                build: Some("26200".into()),
                architecture: Architecture::X64,
                inspection_mode: InspectionMode::StandardUser,
            },
            scopes: vec![DeclaredScope {
                id: scope,
                kind: ScopeKind::InstallationRoot,
                label: "private-folder-fake-secret".into(),
            }],
            requested: vec![target],
            execution: ScanExecution::Finished,
            coverage: vec![CollectorCoverage {
                target,
                state: CoverageState::Inspected,
                gaps: vec![],
            }],
            observations: vec![Observation {
                scope,
                source: ObservationSource::PackageMetadata,
                kind: ObservationKind::Installation,
                identity: IdentityAssessment {
                    tool: Some(ToolIdentity {
                        id: "codex-cli".into(),
                        category: ToolCategory::CodingAssistant,
                    }),
                    class: EvidenceClass::Candidate,
                    rule_match: None,
                },
            }],
        }),
        metrics: Metrics::default(),
        location_links: vec![],
    }
}

#[test]
fn text_and_json_share_identity_action_scope_and_coverage() {
    let result = fixture();
    let public = report::project(&result);
    let json: serde_json::Value = serde_json::from_str(&public.json()).unwrap();
    let text = public.text();
    assert_eq!(&json, public.value());
    assert_eq!(json["eligibility"], "NotIssued");
    assert_eq!(json["findings"][0]["action"], "InventoryOnly");
    assert_eq!(
        json["findings"][0]["indicator_rule_id"],
        "npm-package-name-indicators-v1"
    );
    for token in [
        "scan-1234-1",
        "InspectedRequestedScope",
        "Candidate",
        "InventoryOnly",
        "codex-cli",
        "26200",
    ] {
        assert!(text.contains(token));
    }
    assert_eq!(public.exit_code(), 0);
}

#[test]
fn all_free_text_identity_labels_builds_and_rule_strings_are_minimized() {
    let mut result = fixture();
    let secret = "fake-private-token-7c6";
    result.diagnostic.snapshot.scan_id = secret.into();
    result.diagnostic.snapshot.platform.build = Some(secret.into());
    result.diagnostic.snapshot.observations[0]
        .identity
        .tool
        .as_mut()
        .unwrap()
        .id = secret.into();
    result.diagnostic.snapshot.observations[0]
        .identity
        .rule_match = Some(RuleMatch {
        catalog_version: secret.into(),
        rule_id: secret.into(),
        rule_version: secret.into(),
        approval: RuleApproval::Unreviewed,
    });
    let public = report::project(&result);
    for text in [public.json(), public.text()] {
        assert!(!text.contains(secret));
        assert!(!text.contains("private-folder-fake-secret"));
    }
    assert_eq!(public.value()["findings"][0]["tool_id"], "unknown");
    assert_eq!(public.value()["platform"]["build"], serde_json::Value::Null);
}

#[test]
fn incomplete_empty_reports_and_cancellation_cannot_look_complete() {
    for execution in [
        ScanExecution::Partial,
        ScanExecution::Failed,
        ScanExecution::Cancelled,
    ] {
        let mut result = fixture();
        result.diagnostic.snapshot.observations.clear();
        result.diagnostic.snapshot.execution = execution;
        result.diagnostic = policy::evaluate(result.diagnostic.snapshot);
        let public = report::project(&result);
        assert_eq!(public.exit_code(), 3);
        assert_eq!(public.value()["coverage"], "Incomplete");
        assert!(public.text().contains(&format!("{execution:?}")));
        assert_eq!(public.value()["eligibility"], "NotIssued");
    }
}

#[test]
fn oversized_reports_are_bounded_and_explicitly_incomplete() {
    let mut result = fixture();
    let observation = result.diagnostic.snapshot.observations[0].clone();
    result.diagnostic.snapshot.observations = vec![observation; 600];
    result.diagnostic = policy::evaluate(result.diagnostic.snapshot);
    let public = report::project(&result);
    assert_eq!(public.value()["findings"].as_array().unwrap().len(), 512);
    assert_eq!(public.value()["report_truncated"], true);
    assert_eq!(public.exit_code(), 3);
    assert!(public.text().contains("ReportTruncated"));
    assert!(public.json().len() < 1024 * 1024);
}

#[test]
fn invalid_combined_scope_is_rejected_before_process_collection() {
    let args = ["scan", "--consent", "--processes", "--bin-root", "relative"];
    let Command::Collect(options) = parse_args(args.map(Into::into)).unwrap() else {
        panic!("collection expected");
    };
    assert!(scan::run(options, None, &AtomicBool::new(false)).is_err());
}

#[test]
fn pre_cancelled_scan_returns_explicit_cancelled_scopes_without_native_collection() {
    let Command::Collect(options) =
        parse_args(["scan", "--consent", "--processes"].map(Into::into)).unwrap()
    else {
        panic!("collection expected");
    };
    let result = scan::run(options, None, &AtomicBool::new(true)).unwrap();
    assert_eq!(
        result.diagnostic.snapshot.execution,
        ScanExecution::Cancelled
    );
    assert_eq!(result.metrics.process_entries, 0);
    assert_eq!(report::project(&result).exit_code(), 3);
}

#[test]
fn bad_path_environment_is_ignored_without_opt_in_and_rejected_with_it() {
    let args = ["scan", "--consent", "--processes", "--include-path"];
    let Command::Collect(options) = parse_args(args.map(Into::into)).unwrap() else {
        panic!("collection expected");
    };
    assert!(scan::run(
        options,
        Some(std::ffi::OsStr::new("relative")),
        &AtomicBool::new(false)
    )
    .is_err());
    let Command::Collect(options) =
        parse_args(["scan", "--consent", "--processes"].map(Into::into)).unwrap()
    else {
        panic!("collection expected");
    };
    assert!(scan::run(
        options,
        Some(std::ffi::OsStr::new("relative")),
        &AtomicBool::new(true)
    )
    .is_ok());
}
