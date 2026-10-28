//! A hard-coded synthetic demonstration. Never inspects this computer.
use anti_ai_guard::model::*;
use anti_ai_guard::policy::evaluate;

fn main() {
    let root = ScopeId(1);
    let processes = ScopeId(2);
    let inventory = CoverageTarget {
        collector: Collector::InstallationInventory,
        scope: root,
    };
    let snapshot_target = CoverageTarget {
        collector: Collector::ProcessSnapshot,
        scope: processes,
    };
    let identity = IdentityAssessment {
        tool: Some(ToolIdentity {
            id: "fixture.coding-assistant".into(),
            category: ToolCategory::CodingAssistant,
        }),
        class: EvidenceClass::Candidate,
        rule_match: None,
    };
    let report = evaluate(Snapshot {
        scan_id: "synthetic-demo".into(),
        observed_at_ms: 0,
        platform: Platform {
            os: OperatingSystem::Unknown,
            build: None,
            architecture: Architecture::Unknown,
            inspection_mode: InspectionMode::Unknown,
        },
        scopes: vec![
            DeclaredScope {
                id: root,
                kind: ScopeKind::InstallationRoot,
                label: "fixture root".into(),
            },
            DeclaredScope {
                id: processes,
                kind: ScopeKind::CurrentSessionProcesses,
                label: "fixture session".into(),
            },
        ],
        requested: vec![inventory, snapshot_target],
        execution: ScanExecution::Finished,
        coverage: vec![
            CollectorCoverage {
                target: inventory,
                state: CoverageState::Inspected,
                gaps: vec![],
            },
            CollectorCoverage {
                target: snapshot_target,
                state: CoverageState::Partial,
                gaps: vec![GapReason::AccessDenied],
            },
        ],
        observations: vec![
            Observation {
                scope: root,
                source: ObservationSource::PackageMetadata,
                kind: ObservationKind::Installation,
                identity: identity.clone(),
            },
            Observation {
                scope: processes,
                source: ObservationSource::ProcessSnapshot,
                kind: ObservationKind::Process {
                    instance: ProcessInstance {
                        boot_id: Some("synthetic-boot".into()),
                        pid: 42,
                        start_time: Some(100),
                    },
                    state: ProcessState::AliveAtSnapshot,
                },
                identity,
            },
        ],
    });
    println!("Synthetic demonstration only; no device inspection.");
    for (index, suggestion) in report.suggestions.iter().enumerate() {
        println!(
            "Observation {}: {:?} ({:?})",
            index + 1,
            suggestion.action,
            suggestion.reason
        );
    }
    println!("Coverage: {:?}", report.coverage);
    println!("Eligibility: {:?}", report.eligibility);
}
