//! Bounded, minimized typed public projections. Raw snapshots never serialize.
use crate::{model::*, policy::*, scan::ScanResult};
use serde::Serialize;
use serde_json::Value;
use std::fmt;

const MAX_FINDINGS: usize = 512;
const MAX_COVERAGE: usize = 64;
const MAX_CONCERNS: usize = 1024;

mod wire;
use wire::*;

#[derive(Serialize)]
enum FindingKind {
    #[serde(rename = "Installation")]
    Installation,
    #[serde(rename = "Process")]
    Process,
}
#[derive(Serialize)]
struct PlatformData {
    os: Os,
    build: Option<String>,
    architecture: Arch,
    architecture_basis: &'static str,
    inspection_mode: Mode,
}
#[derive(Serialize)]
struct ScopeData {
    id: u32,
    kind: ScopeType,
    label: String,
}
#[derive(Serialize)]
struct TargetData {
    collector: CollectorType,
    scope_id: u32,
}
#[derive(Serialize)]
struct CollectorData {
    collector: CollectorType,
    scope_id: u32,
    state: CollectorState,
    gaps: Vec<Gap>,
}
#[derive(Serialize)]
struct ConcernData {
    scope_id: Option<u32>,
    collector: Option<CollectorType>,
    reason: ConcernReason,
}
#[derive(Serialize)]
struct ProcessData {
    pid: u32,
    creation_filetime: Option<u64>,
    state: State,
    boot_identity_available: bool,
}
#[derive(Serialize)]
struct FindingData {
    index: usize,
    scope_id: u32,
    kind: FindingKind,
    source: Source,
    tool_id: &'static str,
    evidence_class: Evidence,
    indicator_rule_id: Option<&'static str>,
    upstream_rule_reference_omitted: bool,
    action: ActionWire,
    reason: Reason,
    process: Option<ProcessData>,
}
#[derive(Serialize)]
struct LinkData {
    process_finding: usize,
    installation_finding: usize,
    basis: &'static str,
}
#[derive(Serialize)]
struct MetricsData {
    inventory_probes: usize,
    metadata_bytes: usize,
    process_entries: usize,
    process_queries: usize,
    collection_elapsed_ms: u64,
}
#[derive(Serialize)]
struct ReportData {
    schema_version: u32,
    policy_version: &'static str,
    scan_id: String,
    observed_at_ms: u64,
    platform: PlatformData,
    collection_execution: Execution,
    coverage: Coverage,
    eligibility: &'static str,
    report_truncated: bool,
    indicator_catalog_version: &'static str,
    scopes: Vec<ScopeData>,
    requested: Vec<TargetData>,
    collectors: Vec<CollectorData>,
    coverage_concerns: Vec<ConcernData>,
    findings: Vec<FindingData>,
    location_links: Vec<LinkData>,
    metrics: MetricsData,
    limitations: Vec<&'static str>,
}
pub struct PublicReport {
    data: ReportData,
    // Compatibility view for callers. Constructed only from the typed projection.
    value: Value,
    exit_code: u8,
}
fn tool(id: Option<&str>) -> &'static str {
    match id {
        Some("codex-cli") => "codex-cli",
        Some("claude-code") => "claude-code",
        Some("opencode-v1") => "opencode-v1",
        Some("opencode-v2") => "opencode-v2",
        Some("opencode-unknown-generation") => "opencode-unknown-generation",
        _ => "unknown",
    }
}
fn scan_id(value: &str) -> &str {
    if value.len() > 50 {
        return "redacted-scan-id";
    }
    let parts: Vec<_> = value.split('-').collect();
    if parts.len() == 3
        && parts[0] == "scan"
        && parts[1..]
            .iter()
            .all(|s| !s.is_empty() && s.len() <= 20 && s.bytes().all(|b| b.is_ascii_digit()))
    {
        value
    } else {
        "redacted-scan-id"
    }
}
pub fn project(result: &ScanResult) -> PublicReport {
    let report = &result.diagnostic;
    let snapshot = &report.snapshot;
    let truncated = report.suggestions.len() != snapshot.observations.len()
        || snapshot.requested.len() > MAX_COVERAGE
        || snapshot.observations.len() > MAX_FINDINGS
        || snapshot.scopes.len() > MAX_COVERAGE
        || snapshot.coverage.len() > MAX_COVERAGE
        || report.coverage_concerns.len() > MAX_CONCERNS
        || snapshot.coverage.iter().any(|c| c.gaps.len() > 32)
        || result.location_links.len() > MAX_FINDINGS;
    let coverage = if truncated {
        CoverageSummary::Incomplete
    } else {
        report.coverage
    };
    let findings = snapshot
        .observations
        .iter()
        .take(MAX_FINDINGS)
        .enumerate()
        .map(|(index, o)| {
            let (kind, process) = match &o.kind {
                ObservationKind::Installation => (FindingKind::Installation, None),
                ObservationKind::Process { instance, state } => (
                    FindingKind::Process,
                    Some(ProcessData {
                        pid: instance.pid,
                        creation_filetime: instance.start_time,
                        state: (*state).into(),
                        boot_identity_available: instance.boot_id.is_some(),
                    }),
                ),
            };
            let suggestion = report.suggestions.get(index);
            FindingData {
                index,
                scope_id: o.scope.0,
                kind,
                source: o.source.into(),
                tool_id: tool(o.identity.tool.as_ref().map(|t| t.id.as_str())),
                evidence_class: o.identity.class.into(),
                indicator_rule_id: match (o.identity.class, o.source) {
                    (EvidenceClass::Candidate, ObservationSource::PackageMetadata) => {
                        Some("npm-package-name-indicators-v1")
                    }
                    (EvidenceClass::Candidate, ObservationSource::ExecutableArtifact) => {
                        Some("native-file-name-indicators-v1")
                    }
                    (EvidenceClass::Candidate, ObservationSource::ProcessSnapshot) => {
                        Some("process-basename-indicators-v1")
                    }
                    _ => None,
                },
                upstream_rule_reference_omitted: o.identity.rule_match.is_some(),
                action: suggestion
                    .map(|s| s.action.into())
                    .unwrap_or(ActionWire::RequestReview),
                reason: suggestion
                    .map(|s| s.reason.into())
                    .unwrap_or(Reason::MissingSuggestion),
                process,
            }
        })
        .collect::<Vec<_>>();
    let data = ReportData {
        schema_version: 1,
        policy_version: POLICY_VERSION,
        scan_id: scan_id(&snapshot.scan_id).into(),
        observed_at_ms: snapshot.observed_at_ms,
        platform: PlatformData {
            os: snapshot.platform.os.into(),
            build: snapshot
                .platform
                .build
                .as_ref()
                .filter(|s| !s.is_empty() && s.len() <= 20 && s.bytes().all(|b| b.is_ascii_digit()))
                .cloned(),
            architecture: snapshot.platform.architecture.into(),
            architecture_basis: "compiled-target",
            inspection_mode: snapshot.platform.inspection_mode.into(),
        },
        collection_execution: snapshot.execution.into(),
        coverage: coverage.into(),
        eligibility: "NotIssued",
        report_truncated: truncated,
        indicator_catalog_version: "windows-name-and-package-indicators-v1",
        scopes: snapshot
            .scopes
            .iter()
            .take(MAX_COVERAGE)
            .map(|s| ScopeData {
                id: s.id.0,
                kind: s.kind.into(),
                label: match s.kind {
                    ScopeKind::InstallationRoot => format!("installation-root-{}", s.id.0),
                    ScopeKind::CurrentSessionProcesses => {
                        "current-session-known-native-name-selectors".into()
                    }
                },
            })
            .collect(),
        requested: snapshot
            .requested
            .iter()
            .take(MAX_COVERAGE)
            .map(|t| TargetData {
                collector: t.collector.into(),
                scope_id: t.scope.0,
            })
            .collect(),
        collectors: snapshot
            .coverage
            .iter()
            .take(MAX_COVERAGE)
            .map(|c| CollectorData {
                collector: c.target.collector.into(),
                scope_id: c.target.scope.0,
                state: c.state.into(),
                gaps: c.gaps.iter().take(32).copied().map(Into::into).collect(),
            })
            .collect(),
        coverage_concerns: report
            .coverage_concerns
            .iter()
            .take(MAX_CONCERNS)
            .map(|c| ConcernData {
                scope_id: c.target.map(|t| t.scope.0),
                collector: c.target.map(|t| t.collector.into()),
                reason: c.reason.into(),
            })
            .collect(),
        location_links: result
            .location_links
            .iter()
            .take(MAX_FINDINGS)
            .filter(|l| {
                l.process_observation < findings.len()
                    && l.installation_observation < findings.len()
            })
            .map(|l| LinkData {
                process_finding: l.process_observation,
                installation_finding: l.installation_observation,
                basis: "same-declared-location-only-not-artifact-proof",
            })
            .collect(),
        findings,
        metrics: MetricsData {
            inventory_probes: result.metrics.inventory_probes,
            metadata_bytes: result.metrics.metadata_bytes,
            process_entries: result.metrics.process_entries,
            process_queries: result.metrics.process_queries,
            collection_elapsed_ms: result.metrics.collection_elapsed_ms,
        },
        limitations: vec![
            "only-explicit-installation-roots-and-known-native-name-selectors",
            "no-trusted-strong-identity-catalog",
            "missing-boot-identity-limits-process-correlation",
            "generic-interpreters-renamed-custom-and-guest-agents-not-identified",
            "snapshot-not-continuous-monitoring",
            "empty-findings-never-prove-clean-device",
            "this-report-does-not-establish-desktop-platform-support",
            "no-eligibility-or-automatic-process-control",
        ],
    };
    PublicReport {
        value: serde_json::to_value(&data).expect("bounded typed projection serializes"),
        data,
        exit_code: if snapshot.execution == ScanExecution::Finished
            && coverage == CoverageSummary::InspectedRequestedScope
        {
            0
        } else {
            3
        },
    }
}
impl PublicReport {
    pub fn value(&self) -> &Value {
        &self.value
    }
    pub fn exit_code(&self) -> u8 {
        self.exit_code
    }
    pub fn json(&self) -> String {
        serde_json::to_string_pretty(&self.value).expect("bounded typed projection serializes")
            + "\n"
    }
    pub fn text(&self) -> String {
        let v = &self.data;
        let mut out = format!(
            "Anti-AI Guard diagnostic {}\nExecution: {}; coverage: {}; eligibility: NotIssued\n",
            v.scan_id, v.collection_execution, v.coverage
        );
        out.push_str(&format!(
            "Platform: {} build {} architecture {} (compiled target), mode {}\n",
            v.platform.os,
            v.platform.build.as_deref().unwrap_or("unknown"),
            v.platform.architecture,
            v.platform.inspection_mode
        ));
        for finding in &v.findings {
            out.push_str(&format!(
                "Finding {} scope {}: {} | {} | {} ({})",
                finding.index,
                finding.scope_id,
                finding.tool_id,
                finding.evidence_class,
                finding.action,
                finding.reason
            ));
            if let Some(process) = &finding.process {
                out.push_str(&format!(
                    " PID {} creation={} state={} boot_identity={}",
                    process.pid,
                    serde_json::to_value(process.creation_filetime).expect("integer serializes"),
                    process.state,
                    process.boot_identity_available
                ));
            }
            out.push_str(&format!(
                " source={} indicator_rule={}\n",
                finding.source,
                finding.indicator_rule_id.unwrap_or("unknown")
            ));
        }
        for collector in &v.collectors {
            out.push_str(&format!(
                "Scope {} {}: {} {}\n",
                collector.scope_id,
                collector.collector,
                collector.state,
                serde_json::to_value(&collector.gaps).expect("wire enums serialize")
            ));
        }
        for concern in &v.coverage_concerns {
            out.push_str(&format!(
                "Coverage concern scope {}: {}\n",
                serde_json::to_value(concern.scope_id).expect("integer serializes"),
                concern.reason
            ));
        }
        if v.report_truncated {
            out.push_str("ReportTruncated: bounded output; coverage incomplete.\n");
        }
        for link in &v.location_links {
            out.push_str(&format!(
                "Location link {} -> {}: {}\n",
                link.process_finding, link.installation_finding, link.basis
            ));
        }
        out.push_str(&format!(
            "Metrics: {}\n",
            serde_json::to_value(&v.metrics).expect("integer metrics serialize")
        ));
        out.push_str("Scope is limited to documented selectors; an empty inventory does not mean a clean device.\n");
        for limitation in &v.limitations {
            out.push_str(&format!("Limit: {limitation}\n"));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Serialize, Debug)]
    enum RenamedRustVariant {
        #[serde(rename = "InventoryOnly")]
        InternalInventoryName,
    }
    #[test]
    fn wire_names_survive_rust_debug_identifier_changes() {
        let renamed = RenamedRustVariant::InternalInventoryName;
        assert_ne!(format!("{renamed:?}"), "InventoryOnly");
        assert_eq!(
            serde_json::to_value(renamed).unwrap(),
            serde_json::to_value(ActionWire::from(SuggestedAction::InventoryOnly)).unwrap()
        );
    }
}
