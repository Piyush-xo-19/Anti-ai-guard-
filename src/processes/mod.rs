//! Bounded current-session process-name candidates. Never reads command lines,
//! environment, memory, modules or terminal contents; never controls processes.
mod windows;

use crate::{inventory, model::*};
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_entries: usize,
    pub max_candidates: usize,
    pub max_elapsed: Duration,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_entries: 4096,
            max_candidates: 512,
            max_elapsed: Duration::from_secs(2),
        }
    }
}
#[derive(Clone, Debug)]
pub struct ProcessRequest {
    scope: ScopeId,
    limits: Limits,
}
impl ProcessRequest {
    pub fn new(
        consent: bool,
        scope: ScopeId,
        limits: Limits,
    ) -> Result<Self, inventory::RequestError> {
        if !consent {
            return Err(inventory::RequestError::ConsentRequired);
        }
        if scope.0 == 0 {
            return Err(inventory::RequestError::InvalidScope);
        }
        if limits.max_entries > 16384
            || limits.max_candidates > 512
            || limits.max_elapsed > Duration::from_secs(10)
        {
            return Err(inventory::RequestError::InvalidLimits);
        }
        Ok(Self { scope, limits })
    }
}

pub struct InventoryLink<'a> {
    pub request: &'a inventory::InventoryRequest,
    pub result: &'a inventory::InventoryResult,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessFinding {
    pub observation: Observation,
    /// Only a weak same-declared-location association in this report. No artifact
    /// identity or package-to-process proof; installation cannot upgrade identity.
    pub installation_indicator: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessResult {
    pub scope: DeclaredScope,
    pub coverage: CollectorCoverage,
    pub findings: Vec<ProcessFinding>,
    pub execution: ScanExecution,
    pub enumerated: usize,
    pub queried: usize,
    pub skipped_other_sessions: usize,
}

struct Row {
    pid: u32,
    basename: OsString,
}
struct Context {
    checkpoint: u64,
    session: u32,
}
struct Metadata {
    start: Option<u64>,
    image: Option<PathBuf>,
    state: ProcessState,
    gaps: Vec<GapReason>,
}
trait Provider {
    fn capture(&mut self) -> Result<Context, GapReason>;
    fn next(&mut self) -> Result<Option<Row>, GapReason>;
    fn session(&mut self, pid: u32) -> Result<u32, GapReason>;
    fn metadata(&mut self, pid: u32) -> Result<Metadata, GapReason>;
}

fn tool_id(name: &std::ffi::OsStr) -> Option<&'static str> {
    let name = name.to_str()?;
    if [
        "codex.exe",
        "codex-x86_64-pc-windows-msvc.exe",
        "codex-aarch64-pc-windows-msvc.exe",
    ]
    .iter()
    .any(|n| name.eq_ignore_ascii_case(n))
    {
        Some("codex-cli")
    } else if name.eq_ignore_ascii_case("claude.exe") {
        Some("claude-code")
    } else if name.eq_ignore_ascii_case("opencode.exe") {
        Some("opencode-unknown-generation")
    } else {
        None
    }
}
fn add_gap(result: &mut ProcessResult, reason: GapReason) {
    if !result.coverage.gaps.contains(&reason) {
        result.coverage.gaps.push(reason);
    }
    result.coverage.state = CoverageState::Partial;
}

fn collect_with(
    request: &ProcessRequest,
    cancelled: &AtomicBool,
    inventory: Option<InventoryLink<'_>>,
    provider: &mut impl Provider,
) -> ProcessResult {
    let mut result = ProcessResult {
        scope: DeclaredScope {
            id: request.scope,
            kind: ScopeKind::CurrentSessionProcesses,
            label: "current Windows session; known native name indicators".into(),
        },
        coverage: CollectorCoverage {
            target: CoverageTarget {
                collector: Collector::ProcessSnapshot,
                scope: request.scope,
            },
            state: CoverageState::Inspected,
            gaps: vec![],
        },
        findings: vec![],
        execution: ScanExecution::Finished,
        enumerated: 0,
        queried: 0,
        skipped_other_sessions: 0,
    };
    let began = Instant::now();
    let stop = |entries: usize, candidates: usize| {
        if cancelled.load(Ordering::Relaxed) {
            Some(GapReason::Cancelled)
        } else if entries >= request.limits.max_entries
            || candidates >= request.limits.max_candidates
            || began.elapsed() >= request.limits.max_elapsed
        {
            Some(GapReason::BudgetExceeded)
        } else {
            None
        }
    };
    if let Some(reason) = stop(0, 0) {
        add_gap(&mut result, reason);
    } else {
        match provider.capture() {
            Err(reason) => {
                result.coverage.state = CoverageState::Unavailable;
                result.coverage.gaps.push(reason);
            }
            Ok(context) => loop {
                if let Some(reason) = stop(result.enumerated, result.findings.len()) {
                    add_gap(&mut result, reason);
                    break;
                }
                let row = match provider.next() {
                    Ok(Some(row)) => row,
                    Ok(None) => break,
                    Err(reason) => {
                        add_gap(&mut result, reason);
                        break;
                    }
                };
                result.enumerated += 1;
                let Some(id) = tool_id(&row.basename) else {
                    continue;
                };
                match provider.session(row.pid) {
                    Ok(session) if session != context.session => {
                        result.skipped_other_sessions += 1;
                        continue;
                    }
                    Ok(_) => {}
                    Err(reason) => {
                        add_gap(&mut result, reason);
                        continue;
                    }
                }
                result.queried += 1;
                let metadata = match provider.metadata(row.pid) {
                    Ok(metadata) => metadata,
                    Err(
                        reason @ (GapReason::ProcessExited | GapReason::ProcessIdentityChanged),
                    ) => {
                        add_gap(&mut result, reason);
                        continue;
                    }
                    Err(reason) => {
                        add_gap(&mut result, reason);
                        Metadata {
                            start: None,
                            image: None,
                            state: ProcessState::AliveAtSnapshot,
                            gaps: vec![],
                        }
                    }
                };
                // A new process after the pre-capture checkpoint must not be
                // associated with a potentially reused PID from the snapshot.
                if metadata
                    .start
                    .is_some_and(|start| start > context.checkpoint)
                    || metadata
                        .image
                        .as_ref()
                        .and_then(|p| p.file_name())
                        .is_some_and(|name| !name.eq_ignore_ascii_case(&row.basename))
                {
                    add_gap(&mut result, GapReason::ProcessIdentityChanged);
                    continue;
                }
                for reason in &metadata.gaps {
                    add_gap(&mut result, *reason);
                }
                if metadata.start.is_none() {
                    add_gap(&mut result, GapReason::MissingStartIdentity);
                }
                if metadata.state == ProcessState::ExitedDuringEnrichment {
                    add_gap(&mut result, GapReason::ProcessExited);
                }
                // Do not fabricate a boot ID from wall-clock/uptime estimates.
                // Boot-scoped cross-scan identity remains explicitly incomplete.
                add_gap(&mut result, GapReason::MissingBootIdentity);
                let installation_indicator = inventory.as_ref().and_then(|link| {
                    metadata
                        .image
                        .as_ref()
                        .and_then(|path| link.request.finding_at_path(link.result, path))
                });
                result.findings.push(ProcessFinding {
                    observation: Observation {
                        scope: request.scope,
                        source: ObservationSource::ProcessSnapshot,
                        kind: ObservationKind::Process {
                            instance: ProcessInstance {
                                boot_id: None,
                                pid: row.pid,
                                start_time: metadata.start,
                            },
                            state: metadata.state,
                        },
                        identity: IdentityAssessment {
                            tool: Some(ToolIdentity {
                                id: id.into(),
                                category: ToolCategory::CodingAssistant,
                            }),
                            class: EvidenceClass::Candidate,
                            rule_match: None,
                        },
                    },
                    installation_indicator,
                });
            },
        }
    }
    if cancelled.load(Ordering::Relaxed) {
        add_gap(&mut result, GapReason::Cancelled);
    } else if began.elapsed() >= request.limits.max_elapsed {
        add_gap(&mut result, GapReason::BudgetExceeded);
    }
    if result.coverage.gaps.contains(&GapReason::Cancelled) {
        result.execution = ScanExecution::Cancelled;
    } else if result.coverage.state == CoverageState::Unavailable {
        result.execution = ScanExecution::Failed;
    } else if result.coverage.state != CoverageState::Inspected {
        result.execution = ScanExecution::Partial;
    }
    result
}

pub fn collect(
    request: &ProcessRequest,
    cancelled: &AtomicBool,
    inventory: Option<InventoryLink<'_>>,
) -> ProcessResult {
    collect_with(
        request,
        cancelled,
        inventory,
        &mut windows::NativeProvider::default(),
    )
}

#[cfg(test)]
mod tests;
