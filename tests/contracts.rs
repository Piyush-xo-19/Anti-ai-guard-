//! Synthetic preclassified inputs only. No real assistant artifacts, catalog
//! entries, process enumeration or participant data are used by these tests.

use anti_ai_guard::model::*;
use anti_ai_guard::policy::*;

fn instance() -> ProcessInstance {
    ProcessInstance {
        boot_id: Some("synthetic-boot".into()),
        pid: 42,
        start_time: Some(100),
    }
}

fn identity(class: EvidenceClass) -> IdentityAssessment {
    IdentityAssessment {
        tool: Some(ToolIdentity {
            id: "fixture.coding-assistant".into(),
            category: ToolCategory::CodingAssistant,
        }),
        class,
        rule_match: Some(RuleMatch {
            catalog_version: "fixture-only".into(),
            rule_id: "fixture.preclassified-match".into(),
            rule_version: "1".into(),
            approval: RuleApproval::DiagnosticClosure,
        }),
    }
}

fn process(class: EvidenceClass) -> Observation {
    Observation {
        scope: ScopeId(2),
        source: ObservationSource::ProcessSnapshot,
        kind: ObservationKind::Process {
            instance: instance(),
            state: ProcessState::AliveAtSnapshot,
        },
        identity: identity(class),
    }
}

fn installation() -> Observation {
    Observation {
        scope: ScopeId(1),
        source: ObservationSource::PackageMetadata,
        kind: ObservationKind::Installation,
        identity: identity(EvidenceClass::ConfirmedSupported),
    }
}

fn snapshot(observations: Vec<Observation>) -> Snapshot {
    let requested = vec![
        CoverageTarget {
            collector: Collector::InstallationInventory,
            scope: ScopeId(1),
        },
        CoverageTarget {
            collector: Collector::ProcessSnapshot,
            scope: ScopeId(2),
        },
    ];
    Snapshot {
        scan_id: "fixture-scan".into(),
        observed_at_ms: 1234,
        platform: Platform {
            os: OperatingSystem::Windows11,
            build: None,
            architecture: Architecture::X64,
            inspection_mode: InspectionMode::StandardUser,
        },
        scopes: vec![
            DeclaredScope {
                id: ScopeId(1),
                kind: ScopeKind::InstallationRoot,
                label: "declared root 1".into(),
            },
            DeclaredScope {
                id: ScopeId(2),
                kind: ScopeKind::CurrentSessionProcesses,
                label: "current session".into(),
            },
        ],
        coverage: requested
            .iter()
            .map(|target| CollectorCoverage {
                target: *target,
                state: CoverageState::Inspected,
                gaps: vec![],
            })
            .collect(),
        requested,
        execution: ScanExecution::Finished,
        observations,
    }
}

#[test]
fn installed_assistant_never_becomes_execution_even_with_strong_identity() {
    for class in [
        EvidenceClass::InventoryIndicator,
        EvidenceClass::Candidate,
        EvidenceClass::Corroborated,
        EvidenceClass::ConfirmedSupported,
    ] {
        let mut observed = installation();
        observed.identity.class = class;
        assert_eq!(
            suggest_action(&observed),
            Suggestion {
                action: SuggestedAction::InventoryOnly,
                reason: ActionReason::InstallationIsNotExecution
            }
        );
    }
}

#[test]
fn supported_running_assistant_suggests_closure_without_issuing_eligibility() {
    for class in [
        EvidenceClass::Corroborated,
        EvidenceClass::ConfirmedSupported,
    ] {
        let input = snapshot(vec![installation(), process(class)]);
        let report = evaluate(input.clone());
        assert_eq!(report.suggestions[0].action, SuggestedAction::InventoryOnly);
        assert_eq!(
            report.suggestions[1].action,
            SuggestedAction::RequestClosureAndRescan
        );
        assert_eq!(
            report.suggestions[1].reason,
            ActionReason::SupportedAssistantObserved
        );
        assert_eq!(report.eligibility, Eligibility::NotIssued);
        assert_eq!(
            report.snapshot, input,
            "evaluation retains evidence and has no input-side effects"
        );
    }
}

#[test]
fn name_or_metadata_candidate_cannot_be_promoted_by_an_approved_rule_reference() {
    for class in [EvidenceClass::InventoryIndicator, EvidenceClass::Candidate] {
        assert_eq!(
            suggest_action(&process(class)).action,
            SuggestedAction::RequestReview
        );
    }
}

#[test]
fn missing_unreviewed_or_incomplete_rules_cannot_suggest_closure() {
    let mut cases = vec![];
    let mut missing = process(EvidenceClass::ConfirmedSupported);
    missing.identity.rule_match = None;
    cases.push(missing);
    let mut unreviewed = process(EvidenceClass::Corroborated);
    unreviewed.identity.rule_match.as_mut().unwrap().approval = RuleApproval::Unreviewed;
    cases.push(unreviewed);
    for field in 0..3 {
        let mut incomplete = process(EvidenceClass::ConfirmedSupported);
        let rule = incomplete.identity.rule_match.as_mut().unwrap();
        match field {
            0 => rule.catalog_version.clear(),
            1 => rule.rule_id = " ".into(),
            _ => rule.rule_version.clear(),
        }
        cases.push(incomplete);
    }
    let mut no_tool_id = process(EvidenceClass::ConfirmedSupported);
    no_tool_id.identity.tool.as_mut().unwrap().id.clear();
    cases.push(no_tool_id);
    for observed in cases {
        assert_eq!(
            suggest_action(&observed).action,
            SuggestedAction::RequestReview
        );
    }
}

#[test]
fn ordinary_processes_and_inference_runtimes_are_not_prohibited_assistants() {
    for category in [ToolCategory::LocalInferenceRuntime, ToolCategory::Other] {
        let mut observed = process(EvidenceClass::ConfirmedSupported);
        observed.identity.tool.as_mut().unwrap().category = category;
        assert_eq!(suggest_action(&observed).action, SuggestedAction::NoAction);
    }
    let mut ordinary = process(EvidenceClass::InventoryIndicator);
    ordinary.identity.tool = None;
    assert_eq!(suggest_action(&ordinary).action, SuggestedAction::NoAction);
    ordinary.identity.class = EvidenceClass::Candidate;
    assert_eq!(
        suggest_action(&ordinary).action,
        SuggestedAction::RequestReview
    );
}

#[test]
fn pid_reuse_and_boot_changes_are_different_instances() {
    let original = instance();
    assert_eq!(
        original.correlate(&original.clone()),
        Correlation::SameInstance
    );
    let variants = [
        ProcessInstance {
            start_time: Some(101),
            ..instance()
        },
        ProcessInstance {
            boot_id: Some("another-fixture-boot".into()),
            ..instance()
        },
        ProcessInstance {
            pid: 43,
            ..instance()
        },
    ];
    for other in variants {
        assert_eq!(original.correlate(&other), Correlation::DifferentInstance);
        assert_eq!(other.correlate(&original), Correlation::DifferentInstance);
    }
}

#[test]
fn missing_or_blank_start_identity_never_correlates_by_pid_alone() {
    for partial in [
        ProcessInstance {
            start_time: None,
            ..instance()
        },
        ProcessInstance {
            boot_id: None,
            ..instance()
        },
        ProcessInstance {
            boot_id: Some(" ".into()),
            ..instance()
        },
        ProcessInstance {
            boot_id: None,
            start_time: None,
            ..instance()
        },
    ] {
        assert_eq!(partial.correlate(&instance()), Correlation::Indeterminate);
        let mut observed = process(EvidenceClass::ConfirmedSupported);
        observed.kind = ObservationKind::Process {
            instance: partial,
            state: ProcessState::AliveAtSnapshot,
        };
        let report = evaluate(snapshot(vec![observed]));
        assert_eq!(
            report.suggestions[0].reason,
            ActionReason::MissingProcessIdentity
        );
        assert_eq!(report.coverage, CoverageSummary::Incomplete);
        assert!(report
            .coverage_concerns
            .iter()
            .any(|c| c.reason == CoverageConcernReason::MissingProcessIdentity));
    }
}

#[test]
fn disappearing_process_is_rechecked_and_coverage_remains_incomplete() {
    let mut observed = process(EvidenceClass::ConfirmedSupported);
    observed.kind = ObservationKind::Process {
        instance: instance(),
        state: ProcessState::ExitedDuringEnrichment,
    };
    let report = evaluate(snapshot(vec![observed]));
    assert_eq!(
        report.suggestions[0].action,
        SuggestedAction::RecheckProcess
    );
    assert_eq!(report.coverage, CoverageSummary::Incomplete);
    assert!(report
        .coverage_concerns
        .iter()
        .any(|c| c.reason == CoverageConcernReason::ProcessExited));
}

#[test]
fn empty_findings_with_denied_scope_do_not_produce_complete_coverage() {
    let mut input = snapshot(vec![]);
    input.coverage[1].state = CoverageState::Unavailable;
    input.coverage[1].gaps.push(GapReason::AccessDenied);
    let report = evaluate(input);
    assert!(report.suggestions.is_empty());
    assert_eq!(report.coverage, CoverageSummary::Incomplete);
    assert_eq!(
        report.snapshot.coverage[1].gaps,
        vec![GapReason::AccessDenied]
    );
    assert_eq!(report.eligibility, Eligibility::NotIssued);
}

#[test]
fn every_unfinished_collector_state_and_reported_gap_stays_incomplete() {
    for state in [
        CoverageState::Partial,
        CoverageState::Unavailable,
        CoverageState::NotRequested,
        CoverageState::OutsideMilestone,
    ] {
        let mut input = snapshot(vec![]);
        input.coverage[0].state = state;
        assert_eq!(evaluate(input).coverage, CoverageSummary::Incomplete);
    }
    for gap in [
        GapReason::AccessDenied,
        GapReason::ProcessExited,
        GapReason::MissingStartIdentity,
        GapReason::ScopeBoundary,
        GapReason::BudgetExceeded,
        GapReason::Cancelled,
        GapReason::CollectorFailed,
        GapReason::Unsupported,
    ] {
        let mut input = snapshot(vec![]);
        input.coverage[0].gaps.push(gap);
        assert_eq!(evaluate(input).coverage, CoverageSummary::Incomplete);
    }
}

#[test]
fn missing_or_duplicate_coverage_and_duplicate_requests_are_disclosed() {
    let mut missing = snapshot(vec![]);
    missing.coverage.pop();
    let mut duplicate = snapshot(vec![]);
    duplicate.coverage.push(duplicate.coverage[0].clone());
    let mut duplicate_request = snapshot(vec![]);
    duplicate_request
        .requested
        .push(duplicate_request.requested[0]);
    for (input, reason) in [
        (missing, CoverageConcernReason::MissingRecord),
        (duplicate, CoverageConcernReason::DuplicateRecord),
        (duplicate_request, CoverageConcernReason::DuplicateRequest),
    ] {
        let report = evaluate(input);
        assert_eq!(report.coverage, CoverageSummary::Incomplete);
        assert!(report.coverage_concerns.iter().any(|c| c.reason == reason));
    }
}

#[test]
fn partial_failed_and_cancelled_scans_cannot_look_complete() {
    for execution in [
        ScanExecution::Partial,
        ScanExecution::Failed,
        ScanExecution::Cancelled,
    ] {
        let mut input = snapshot(vec![]);
        input.execution = execution;
        let report = evaluate(input);
        assert_eq!(report.coverage, CoverageSummary::Incomplete);
        assert!(report
            .coverage_concerns
            .iter()
            .any(|c| c.reason == CoverageConcernReason::ScanNotFinished));
    }
}

#[test]
fn no_requested_inspection_is_distinct_from_inspected_scope() {
    let mut input = snapshot(vec![]);
    input.requested.clear();
    input.coverage.clear();
    assert_eq!(evaluate(input).coverage, CoverageSummary::NotRequested);
    let report = evaluate(snapshot(vec![]));
    assert_eq!(report.coverage, CoverageSummary::InspectedRequestedScope);
    assert_eq!(report.eligibility, Eligibility::NotIssued);
}

#[test]
fn unrequested_runtime_events_remain_outside_milestone() {
    let mut input = snapshot(vec![]);
    input.coverage.push(CollectorCoverage {
        target: CoverageTarget {
            collector: Collector::RuntimeEvents,
            scope: ScopeId(2),
        },
        state: CoverageState::OutsideMilestone,
        gaps: vec![GapReason::Unsupported],
    });
    let report = evaluate(input);
    assert_eq!(report.coverage, CoverageSummary::InspectedRequestedScope);
    assert_eq!(
        report.snapshot.coverage[2].state,
        CoverageState::OutsideMilestone
    );
}

#[test]
fn missing_duplicate_or_wrong_kind_scope_prevents_complete_coverage() {
    let mut missing = snapshot(vec![]);
    missing.scopes.pop();
    let mut duplicate = snapshot(vec![]);
    duplicate.scopes.push(duplicate.scopes[0].clone());
    let mut wrong_kind = snapshot(vec![]);
    wrong_kind.scopes[1].kind = ScopeKind::InstallationRoot;
    for input in [missing, duplicate, wrong_kind] {
        let report = evaluate(input);
        assert_eq!(report.coverage, CoverageSummary::Incomplete);
        assert!(report
            .coverage_concerns
            .iter()
            .any(|c| c.reason == CoverageConcernReason::InvalidScope));
    }
}

#[test]
fn out_of_scope_observations_never_suggest_closure() {
    let mut outside = process(EvidenceClass::ConfirmedSupported);
    outside.scope = ScopeId(99);
    let mut unrequested = snapshot(vec![process(EvidenceClass::ConfirmedSupported)]);
    unrequested.requested.pop();
    for input in [snapshot(vec![outside]), unrequested] {
        let report = evaluate(input);
        assert_eq!(report.suggestions[0].action, SuggestedAction::RequestReview);
        assert_eq!(
            report.suggestions[0].reason,
            ActionReason::ObservationOutsideScope
        );
        assert_eq!(report.coverage, CoverageSummary::Incomplete);
    }
}

#[test]
fn active_finding_and_incomplete_coverage_are_reported_independently() {
    let mut input = snapshot(vec![process(EvidenceClass::ConfirmedSupported)]);
    input.coverage[0].state = CoverageState::Unavailable;
    input.coverage[0].gaps.push(GapReason::AccessDenied);
    let report = evaluate(input);
    assert_eq!(
        report.suggestions[0].action,
        SuggestedAction::RequestClosureAndRescan
    );
    assert_eq!(report.coverage, CoverageSummary::Incomplete);
    assert_eq!(report.eligibility, Eligibility::NotIssued);
}

#[test]
fn supported_installation_does_not_upgrade_a_candidate_process() {
    let report = evaluate(snapshot(vec![
        installation(),
        process(EvidenceClass::Candidate),
    ]));
    assert_eq!(report.suggestions[0].action, SuggestedAction::InventoryOnly);
    assert_eq!(report.suggestions[1].action, SuggestedAction::RequestReview);
    assert_eq!(report.policy_version, POLICY_VERSION);
}

#[test]
fn package_metadata_is_not_a_process_observation() {
    let mut observed = process(EvidenceClass::ConfirmedSupported);
    observed.source = ObservationSource::PackageMetadata;
    let report = evaluate(snapshot(vec![observed]));
    assert_eq!(report.suggestions[0].action, SuggestedAction::RequestReview);
    assert_eq!(
        report.suggestions[0].reason,
        ActionReason::InvalidObservationSource
    );
    assert_eq!(report.coverage, CoverageSummary::Incomplete);
}

#[test]
fn repeated_requests_report_each_target_reason_once() {
    let mut input = snapshot(vec![]);
    let target = input.requested[0];
    input.requested.extend([target, target]);
    input.coverage.retain(|c| c.target != target);
    let report = evaluate(input);
    for reason in [
        CoverageConcernReason::DuplicateRequest,
        CoverageConcernReason::MissingRecord,
    ] {
        assert_eq!(
            report
                .coverage_concerns
                .iter()
                .filter(|c| c.target == Some(target) && c.reason == reason)
                .count(),
            1
        );
    }
}

#[test]
fn surrounding_boot_whitespace_is_normalized_consistently() {
    let first = instance();
    let mut padded = first.clone();
    padded.boot_id = Some(" \t synthetic-boot \n".into());
    assert!(padded.has_stable_identity());
    assert_eq!(first.correlate(&padded), Correlation::SameInstance);
    assert_eq!(padded.correlate(&first), Correlation::SameInstance);
    padded.boot_id = Some("synthetic- boot".into());
    assert_eq!(first.correlate(&padded), Correlation::DifferentInstance);
}

#[test]
fn requested_runtime_events_have_a_milestone_concern_not_a_false_scope_error() {
    let mut input = snapshot(vec![]);
    let target = CoverageTarget {
        collector: Collector::RuntimeEvents,
        scope: ScopeId(2),
    };
    input.requested.push(target);
    input.coverage.push(CollectorCoverage {
        target,
        state: CoverageState::OutsideMilestone,
        gaps: vec![GapReason::Unsupported],
    });
    let report = evaluate(input);
    assert!(report
        .coverage_concerns
        .iter()
        .any(|c| c.target == Some(target)
            && c.reason == CoverageConcernReason::CollectorOutsideMilestone));
    assert!(!report
        .coverage_concerns
        .iter()
        .any(|c| c.target == Some(target)
            && matches!(
                c.reason,
                CoverageConcernReason::InvalidScope | CoverageConcernReason::CollectorIncomplete
            )));
    assert_eq!(report.coverage, CoverageSummary::Incomplete);
}

#[test]
fn unrequested_active_records_are_concerns_but_passive_capability_records_are_allowed() {
    for state in [
        CoverageState::Inspected,
        CoverageState::Partial,
        CoverageState::Unavailable,
        CoverageState::NotRequested,
        CoverageState::OutsideMilestone,
    ] {
        let mut input = snapshot(vec![]);
        let target = CoverageTarget {
            collector: Collector::InstallationInventory,
            scope: ScopeId(99),
        };
        input.coverage.push(CollectorCoverage {
            target,
            state,
            gaps: vec![],
        });
        let report = evaluate(input);
        let active = matches!(
            state,
            CoverageState::Inspected | CoverageState::Partial | CoverageState::Unavailable
        );
        assert_eq!(
            report
                .coverage_concerns
                .iter()
                .any(|c| c.target == Some(target)
                    && c.reason == CoverageConcernReason::UnrequestedRecord),
            active
        );
        assert_eq!(
            report.coverage,
            if active {
                CoverageSummary::Incomplete
            } else {
                CoverageSummary::InspectedRequestedScope
            }
        );
    }
}

#[test]
fn blank_scan_identifiers_are_concerns_and_epoch_zero_is_an_allowed_synthetic_timestamp() {
    for id in ["", " \t\n"] {
        let mut input = snapshot(vec![]);
        input.scan_id = id.into();
        let report = evaluate(input);
        assert!(report
            .coverage_concerns
            .iter()
            .any(|c| c.target.is_none() && c.reason == CoverageConcernReason::InvalidScanId));
        assert_eq!(report.coverage, CoverageSummary::Incomplete);
    }
    let mut input = snapshot(vec![]);
    input.observed_at_ms = 0;
    assert_eq!(
        evaluate(input).coverage,
        CoverageSummary::InspectedRequestedScope
    );
}
