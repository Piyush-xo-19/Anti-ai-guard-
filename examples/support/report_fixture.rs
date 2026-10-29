//! Shared synthetic fixture; no host collection or real identity catalog.
use anti_ai_guard::{
    model::*,
    policy,
    scan::{Metrics, ScanResult},
};
pub fn fixture() -> ScanResult {
    let installation = ScopeId(1);
    let process = ScopeId(2);
    let target = CoverageTarget {
        collector: Collector::InstallationInventory,
        scope: installation,
    };
    let process_target = CoverageTarget {
        collector: Collector::ProcessSnapshot,
        scope: process,
    };
    let identity = IdentityAssessment {
        tool: Some(ToolIdentity {
            id: "codex-cli".into(),
            category: ToolCategory::CodingAssistant,
        }),
        class: EvidenceClass::Candidate,
        rule_match: None,
    };
    ScanResult {
        diagnostic: policy::evaluate(Snapshot {
            scan_id: "scan-1234-0".into(),
            observed_at_ms: 1234,
            platform: Platform {
                os: OperatingSystem::Unknown,
                build: None,
                architecture: Architecture::Unknown,
                inspection_mode: InspectionMode::Unknown,
            },
            scopes: vec![
                DeclaredScope {
                    id: installation,
                    kind: ScopeKind::InstallationRoot,
                    label: "synthetic installation root".into(),
                },
                DeclaredScope {
                    id: process,
                    kind: ScopeKind::CurrentSessionProcesses,
                    label: "synthetic current-session selector".into(),
                },
            ],
            requested: vec![target, process_target],
            execution: ScanExecution::Partial,
            coverage: vec![
                CollectorCoverage {
                    target,
                    state: CoverageState::Inspected,
                    gaps: vec![],
                },
                CollectorCoverage {
                    target: process_target,
                    state: CoverageState::Partial,
                    gaps: vec![GapReason::AccessDenied, GapReason::MissingBootIdentity],
                },
            ],
            observations: vec![
                Observation {
                    scope: installation,
                    source: ObservationSource::PackageMetadata,
                    kind: ObservationKind::Installation,
                    identity: identity.clone(),
                },
                Observation {
                    scope: process,
                    source: ObservationSource::ProcessSnapshot,
                    kind: ObservationKind::Process {
                        instance: ProcessInstance {
                            boot_id: None,
                            pid: 42,
                            start_time: Some(100),
                        },
                        state: ProcessState::AliveAtSnapshot,
                    },
                    identity,
                },
            ],
        }),
        metrics: Metrics::default(),
        location_links: vec![],
    }
}
