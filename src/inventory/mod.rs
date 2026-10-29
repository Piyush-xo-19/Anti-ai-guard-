//! Consented, bounded installation indicators. No execution or strong identity
//! claims, recursive discovery, discovered-tool launches or writes.
mod windows;

use crate::model::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootKind {
    BinDirectory,
    WindowsNpmPrefix,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryRoot {
    pub path: PathBuf,
    pub kind: RootKind,
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_probes: usize,
    pub max_manifest_bytes: usize,
    pub max_total_bytes: usize,
    /// Cooperative deadline, checked between bounded probes, not an OS I/O timeout.
    pub max_elapsed: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_probes: 512,
            max_manifest_bytes: 64 * 1024,
            max_total_bytes: 512 * 1024,
            max_elapsed: Duration::from_secs(2),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestError {
    ConsentRequired,
    InvalidScope,
    InvalidLimits,
}

/// Private fields prevent bypassing validation. Construction is not a signed
/// consent receipt; CLI callers must obtain the participant's explicit consent.
#[derive(Clone, Debug)]
pub struct InventoryRequest {
    roots: Vec<InventoryRoot>,
    limits: Limits,
}

impl InventoryRequest {
    pub fn new(
        consent: bool,
        roots: Vec<InventoryRoot>,
        limits: Limits,
    ) -> Result<Self, RequestError> {
        if !consent {
            return Err(RequestError::ConsentRequired);
        }
        if roots.is_empty()
            || roots.len() > 32
            || roots.iter().any(|r| !windows::valid_root(&r.path))
        {
            return Err(RequestError::InvalidScope);
        }
        if limits.max_probes > 512
            || limits.max_manifest_bytes > 64 * 1024
            || limits.max_total_bytes > 512 * 1024
            || limits.max_elapsed > Duration::from_secs(10)
        {
            return Err(RequestError::InvalidLimits);
        }
        // Never silently drop a different selector scope for the same root.
        let mut unique: Vec<InventoryRoot> = Vec::new();
        for root in roots {
            if !unique.iter().any(|r| {
                r.kind == root.kind
                    && r.path
                        .as_os_str()
                        .eq_ignore_ascii_case(root.path.as_os_str())
            }) {
                unique.push(root);
            }
        }
        Ok(Self {
            roots: unique,
            limits,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallationFinding {
    /// Fixed selector, never a participant's absolute path or manifest contents.
    pub selector: &'static str,
    pub package_version: Option<String>,
    pub observation: Observation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryResult {
    pub scopes: Vec<DeclaredScope>,
    pub coverage: Vec<CollectorCoverage>,
    pub findings: Vec<InstallationFinding>,
    pub execution: ScanExecution,
    pub probes: usize,
    pub bytes_read: usize,
}

struct Selector {
    relative: &'static str,
    tool: &'static str,
    package: Option<&'static str>,
}

const BIN_SELECTORS: &[Selector] = &[
    Selector {
        relative: "codex.exe",
        tool: "codex-cli",
        package: None,
    },
    Selector {
        relative: "codex.cmd",
        tool: "codex-cli",
        package: None,
    },
    Selector {
        relative: "codex.ps1",
        tool: "codex-cli",
        package: None,
    },
    Selector {
        relative: "claude.exe",
        tool: "claude-code",
        package: None,
    },
    Selector {
        relative: "claude.cmd",
        tool: "claude-code",
        package: None,
    },
    Selector {
        relative: "claude.ps1",
        tool: "claude-code",
        package: None,
    },
    Selector {
        relative: "opencode.exe",
        tool: "opencode-unknown-generation",
        package: None,
    },
    Selector {
        relative: "opencode.cmd",
        tool: "opencode-unknown-generation",
        package: None,
    },
    Selector {
        relative: "opencode.ps1",
        tool: "opencode-unknown-generation",
        package: None,
    },
];
const PACKAGE_SELECTORS: &[Selector] = &[
    Selector {
        relative: "node_modules/@openai/codex/package.json",
        tool: "codex-cli",
        package: Some("@openai/codex"),
    },
    Selector {
        relative: "node_modules/@anthropic-ai/claude-code/package.json",
        tool: "claude-code",
        package: Some("@anthropic-ai/claude-code"),
    },
    Selector {
        relative: "node_modules/opencode-ai/package.json",
        tool: "opencode-v1",
        package: Some("opencode-ai"),
    },
    Selector {
        relative: "node_modules/@opencode/cli/package.json",
        tool: "opencode-v2",
        package: Some("@opencode/cli"),
    },
];

enum Probe {
    Absent,
    Indicator,
    Manifest(Vec<u8>),
}
trait RootReader {
    fn probe(&mut self, selector: &Selector, max_bytes: usize) -> Result<Probe, GapReason>;
}
trait InventoryFs {
    fn open_root(&mut self, path: &Path) -> Result<Box<dyn RootReader>, GapReason>;
}

fn gap(coverage: &mut CollectorCoverage, reason: GapReason) {
    coverage.state = CoverageState::Partial;
    if !coverage.gaps.contains(&reason) {
        coverage.gaps.push(reason);
    }
}

/// Only exact recognized name/version fields leave the parser. Parser errors and
/// arbitrary manifest strings are never included in findings or diagnostics.
fn manifest_version(bytes: &[u8], expected: &str) -> Result<String, GapReason> {
    let manifest: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| GapReason::InvalidMetadata)?;
    if manifest.get("name").and_then(|v| v.as_str()) != Some(expected) {
        return Err(GapReason::InvalidMetadata);
    }
    let version = manifest
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or(GapReason::InvalidMetadata)?;
    if version.is_empty()
        || version.len() > 64
        || !version
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".+-_".contains(&b))
    {
        return Err(GapReason::InvalidMetadata);
    }
    Ok(version.into())
}

fn collect_with(
    request: &InventoryRequest,
    cancelled: &AtomicBool,
    fs: &mut impl InventoryFs,
) -> InventoryResult {
    let started = Instant::now();
    let mut result = InventoryResult {
        scopes: vec![],
        coverage: vec![],
        findings: vec![],
        execution: ScanExecution::Finished,
        probes: 0,
        bytes_read: 0,
    };
    for (index, root) in request.roots.iter().enumerate() {
        let scope = ScopeId(index as u32 + 1);
        result.scopes.push(DeclaredScope {
            id: scope,
            kind: ScopeKind::InstallationRoot,
            label: format!("declared installation root {}", index + 1),
        });
        let mut coverage = CollectorCoverage {
            target: CoverageTarget {
                collector: Collector::InstallationInventory,
                scope,
            },
            state: CoverageState::Inspected,
            gaps: vec![],
        };
        let stop_reason = |probes: usize| {
            if cancelled.load(Ordering::Relaxed) {
                Some(GapReason::Cancelled)
            } else if probes >= request.limits.max_probes
                || started.elapsed() >= request.limits.max_elapsed
            {
                Some(GapReason::BudgetExceeded)
            } else {
                None
            }
        };
        if let Some(reason) = stop_reason(result.probes) {
            gap(&mut coverage, reason);
            result.coverage.push(coverage);
            continue;
        }
        let mut reader = match fs.open_root(&root.path) {
            Ok(reader) => reader,
            Err(reason) => {
                coverage.state = CoverageState::Unavailable;
                coverage.gaps.push(reason);
                result.coverage.push(coverage);
                continue;
            }
        };
        let selectors = BIN_SELECTORS.iter().chain(
            PACKAGE_SELECTORS
                .iter()
                .filter(|_| root.kind == RootKind::WindowsNpmPrefix),
        );
        for selector in selectors {
            if let Some(reason) = stop_reason(result.probes) {
                gap(&mut coverage, reason);
                break;
            }
            let bytes_remaining = request
                .limits
                .max_total_bytes
                .saturating_sub(result.bytes_read);
            let byte_limit = request.limits.max_manifest_bytes.min(bytes_remaining);
            if selector.package.is_some() && byte_limit == 0 {
                gap(&mut coverage, GapReason::BudgetExceeded);
                break;
            }
            result.probes += 1;
            let probe = match reader.probe(selector, byte_limit) {
                Ok(probe) => probe,
                Err(reason) => {
                    gap(&mut coverage, reason);
                    continue;
                }
            };
            let version = match probe {
                Probe::Absent => continue,
                Probe::Indicator => None,
                Probe::Manifest(bytes) => {
                    result.bytes_read += bytes.len();
                    if bytes.len() > byte_limit {
                        gap(&mut coverage, GapReason::BudgetExceeded);
                        continue;
                    }
                    match manifest_version(&bytes, selector.package.expect("manifest selector")) {
                        Ok(version) => Some(version),
                        Err(reason) => {
                            gap(&mut coverage, reason);
                            continue;
                        }
                    }
                }
            };
            result.findings.push(InstallationFinding {
                selector: selector.relative,
                package_version: version,
                observation: Observation {
                    scope,
                    source: if selector.package.is_some() {
                        ObservationSource::PackageMetadata
                    } else {
                        ObservationSource::ExecutableArtifact
                    },
                    kind: ObservationKind::Installation,
                    identity: IdentityAssessment {
                        tool: Some(ToolIdentity {
                            id: selector.tool.into(),
                            category: ToolCategory::CodingAssistant,
                        }),
                        class: EvidenceClass::Candidate,
                        rule_match: None,
                    },
                },
            });
        }
        if cancelled.load(Ordering::Relaxed) {
            gap(&mut coverage, GapReason::Cancelled);
        } else if started.elapsed() >= request.limits.max_elapsed {
            gap(&mut coverage, GapReason::BudgetExceeded);
        }
        result.coverage.push(coverage);
    }
    if result
        .coverage
        .iter()
        .any(|c| c.gaps.contains(&GapReason::Cancelled))
    {
        result.execution = ScanExecution::Cancelled;
    } else if result
        .coverage
        .iter()
        .any(|c| c.state != CoverageState::Inspected)
    {
        result.execution = ScanExecution::Partial;
    }
    result
}

/// Windows only. Callers must inspect coverage even if there are no findings.
/// No tool is launched and no file is created, modified or deleted.
pub fn collect(request: &InventoryRequest, cancelled: &AtomicBool) -> InventoryResult {
    collect_with(request, cancelled, &mut windows::WindowsFs)
}

#[cfg(test)]
mod tests;
