//! Bounded, minimized human/JSON projections from the SAME diagnostic result.
//! Never serializes a raw Snapshot, Debug dump, path, argument or free-text label.
use crate::{model::*, policy::CoverageSummary, scan::ScanResult};
use serde_json::{json, Value};

const MAX_FINDINGS: usize = 512;
const MAX_COVERAGE: usize = 64;
const MAX_CONCERNS: usize = 1024;

pub struct PublicReport {
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
    let findings:Vec<_>=snapshot.observations.iter().take(MAX_FINDINGS).enumerate().map(|(index,o)| {
        let (kind,pid,start,state,boot)=match &o.kind {
            ObservationKind::Installation=>("Installation",None,None,None,false),
            ObservationKind::Process {instance,state}=>("Process",Some(instance.pid),instance.start_time,Some(format!("{state:?}")),instance.boot_id.is_some()),
        };
        let suggestion=report.suggestions.get(index);
        let rule=match (o.identity.class,o.source) {
            (EvidenceClass::Candidate,ObservationSource::PackageMetadata)=>Some("npm-package-name-indicators-v1"),
            (EvidenceClass::Candidate,ObservationSource::ExecutableArtifact)=>Some("native-file-name-indicators-v1"),
            (EvidenceClass::Candidate,ObservationSource::ProcessSnapshot)=>Some("process-basename-indicators-v1"),
            _=>None,
        };
        json!({"index":index,"scope_id":o.scope.0,"kind":kind,"source":format!("{:?}",o.source),"tool_id":tool(o.identity.tool.as_ref().map(|t|t.id.as_str())),
            "evidence_class":format!("{:?}",o.identity.class),"indicator_rule_id":rule,"upstream_rule_reference_omitted":o.identity.rule_match.is_some(),
            "action":suggestion.map(|s|format!("{:?}",s.action)).unwrap_or_else(||"RequestReview".into()),"reason":suggestion.map(|s|format!("{:?}",s.reason)).unwrap_or_else(||"MissingSuggestion".into()),
            "process":pid.map(|pid|json!({"pid":pid,"creation_filetime":start,"state":state,"boot_identity_available":boot}))})
    }).collect();
    let scopes:Vec<_>=snapshot.scopes.iter().take(MAX_COVERAGE).map(|s|json!({"id":s.id.0,"kind":format!("{:?}",s.kind),"label":match s.kind {ScopeKind::InstallationRoot=>format!("installation-root-{}",s.id.0),ScopeKind::CurrentSessionProcesses=>"current-session-known-native-name-selectors".into()}})).collect();
    let collectors:Vec<_>=snapshot.coverage.iter().take(MAX_COVERAGE).map(|c|json!({"collector":format!("{:?}",c.target.collector),"scope_id":c.target.scope.0,"state":format!("{:?}",c.state),"gaps":c.gaps.iter().take(32).map(|g|format!("{g:?}")).collect::<Vec<_>>()})).collect();
    let requested:Vec<_>=snapshot.requested.iter().take(MAX_COVERAGE).map(|target|json!({"collector":format!("{:?}",target.collector),"scope_id":target.scope.0})).collect();
    let concerns:Vec<_>=report.coverage_concerns.iter().take(MAX_CONCERNS).map(|c|json!({"scope_id":c.target.map(|t|t.scope.0),"collector":c.target.map(|t|format!("{:?}",t.collector)),"reason":format!("{:?}",c.reason)})).collect();
    let links:Vec<_>=result.location_links.iter().take(MAX_FINDINGS).filter(|l|l.process_observation<findings.len() && l.installation_observation<findings.len()).map(|l|json!({"process_finding":l.process_observation,"installation_finding":l.installation_observation,"basis":"same-declared-location-only-not-artifact-proof"})).collect();
    let build = snapshot
        .platform
        .build
        .as_deref()
        .filter(|s| !s.is_empty() && s.len() <= 20 && s.bytes().all(|b| b.is_ascii_digit()));
    let value = json!({"schema_version":1,"policy_version":crate::policy::POLICY_VERSION,"scan_id":scan_id(&snapshot.scan_id),"observed_at_ms":snapshot.observed_at_ms,
        "platform":{"os":format!("{:?}",snapshot.platform.os),"build":build,"architecture":format!("{:?}",snapshot.platform.architecture),"architecture_basis":"compiled-target","inspection_mode":format!("{:?}",snapshot.platform.inspection_mode)},
        "collection_execution":format!("{:?}",snapshot.execution),"coverage":format!("{coverage:?}"),"eligibility":"NotIssued","report_truncated":truncated,"indicator_catalog_version":"windows-name-and-package-indicators-v1",
        "scopes":scopes,"requested":requested,"collectors":collectors,"coverage_concerns":concerns,"findings":findings,"location_links":links,
        "metrics":{"inventory_probes":result.metrics.inventory_probes,"metadata_bytes":result.metrics.metadata_bytes,"process_entries":result.metrics.process_entries,"process_queries":result.metrics.process_queries,"collection_elapsed_ms":result.metrics.collection_elapsed_ms},
        "limitations":["only-explicit-installation-roots-and-known-native-name-selectors","no-trusted-strong-identity-catalog","missing-boot-identity-limits-process-correlation","generic-interpreters-renamed-custom-and-guest-agents-not-identified","snapshot-not-continuous-monitoring","empty-findings-never-prove-clean-device","this-report-does-not-establish-desktop-platform-support","no-eligibility-or-automatic-process-control"]});
    PublicReport {
        value,
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
        serde_json::to_string_pretty(&self.value).expect("bounded JSON value serializes") + "\n"
    }
    pub fn text(&self) -> String {
        fn string(value: &Value) -> &str {
            value.as_str().unwrap_or("unknown")
        }
        let v = &self.value;
        let mut out = format!(
            "Anti-AI Guard diagnostic {}\nExecution: {}; coverage: {}; eligibility: NotIssued\n",
            string(&v["scan_id"]),
            string(&v["collection_execution"]),
            string(&v["coverage"])
        );
        out.push_str(&format!(
            "Platform: {} build {} architecture {} (compiled target), mode {}\n",
            string(&v["platform"]["os"]),
            string(&v["platform"]["build"]),
            string(&v["platform"]["architecture"]),
            string(&v["platform"]["inspection_mode"])
        ));
        for finding in v["findings"].as_array().expect("constructed array") {
            out.push_str(&format!(
                "Finding {} scope {}: {} | {} | {} ({})",
                finding["index"],
                finding["scope_id"],
                string(&finding["tool_id"]),
                string(&finding["evidence_class"]),
                string(&finding["action"]),
                string(&finding["reason"])
            ));
            if let Some(pid) = finding["process"]["pid"].as_u64() {
                out.push_str(&format!(
                    " PID {pid} creation={} state={} boot_identity={}",
                    finding["process"]["creation_filetime"],
                    string(&finding["process"]["state"]),
                    finding["process"]["boot_identity_available"]
                ));
            }
            out.push_str(&format!(
                " source={} indicator_rule={}",
                string(&finding["source"]),
                string(&finding["indicator_rule_id"])
            ));
            out.push('\n');
        }
        for collector in v["collectors"].as_array().expect("constructed array") {
            out.push_str(&format!(
                "Scope {} {}: {} {}\n",
                collector["scope_id"],
                string(&collector["collector"]),
                string(&collector["state"]),
                collector["gaps"]
            ));
        }
        for concern in v["coverage_concerns"]
            .as_array()
            .expect("constructed array")
        {
            out.push_str(&format!(
                "Coverage concern scope {}: {}\n",
                concern["scope_id"],
                string(&concern["reason"])
            ));
        }
        if v["report_truncated"] == true {
            out.push_str("ReportTruncated: bounded output; coverage incomplete.\n");
        }
        for link in v["location_links"].as_array().expect("constructed array") {
            out.push_str(&format!(
                "Location link {} -> {}: {}\n",
                link["process_finding"],
                link["installation_finding"],
                string(&link["basis"])
            ));
        }
        out.push_str(&format!("Metrics: {}\n", v["metrics"]));
        out.push_str("Scope is limited to documented selectors; an empty inventory does not mean a clean device.\n");
        for limitation in v["limitations"].as_array().expect("constructed array") {
            out.push_str(&format!("Limit: {}\n", string(limitation)));
        }
        out
    }
}
