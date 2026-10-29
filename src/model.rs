//! In-memory contracts for future collectors and diagnostic reports.
//!
//! These types do not inspect or authenticate anything. Callers must supply
//! minimized metadata and assessments from reviewed identity rules. There is
//! deliberately no raw command line, environment, prompt or document field.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperatingSystem {
    Windows10,
    Windows11,
    WindowsServer,
    Linux,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Architecture {
    X64,
    Arm64,
    X86,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectionMode {
    StandardUser,
    ElevatedDiagnostic,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Platform {
    pub os: OperatingSystem,
    pub build: Option<String>,
    pub architecture: Architecture,
    pub inspection_mode: InspectionMode,
}

/// An opaque, report-local scope reference, not an absolute personal path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScopeId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    InstallationRoot,
    CurrentSessionProcesses,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclaredScope {
    pub id: ScopeId,
    pub kind: ScopeKind,
    /// Caller-minimized label, e.g. "declared package root 1"; not a raw path.
    pub label: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collector {
    InstallationInventory,
    ProcessSnapshot,
    /// Representable as outside M2; no event collector is implemented.
    RuntimeEvents,
}

/// Coverage is recorded for each requested collector/scope pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoverageTarget {
    pub collector: Collector,
    pub scope: ScopeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverageState {
    Inspected,
    Partial,
    Unavailable,
    NotRequested,
    OutsideMilestone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GapReason {
    AccessDenied,
    ProcessExited,
    MissingStartIdentity,
    ScopeBoundary,
    BudgetExceeded,
    Cancelled,
    CollectorFailed,
    Unsupported,
    InvalidMetadata,
    MissingRoot,
    InvalidScope,
    SharingConflict,
    MissingBootIdentity,
    ProcessIdentityChanged,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectorCoverage {
    pub target: CoverageTarget,
    pub state: CoverageState,
    pub gaps: Vec<GapReason>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanExecution {
    Finished,
    Partial,
    Failed,
    Cancelled,
}

/// Start time is the collector's stable process-creation value, not scan time.
/// The boot ID and start time must use the same source/domain when compared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessInstance {
    pub boot_id: Option<String>,
    pub pid: u32,
    pub start_time: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Correlation {
    SameInstance,
    DifferentInstance,
    Indeterminate,
}

impl ProcessInstance {
    fn normalized_boot_id(&self) -> Option<&str> {
        self.boot_id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
    }
    pub fn has_stable_identity(&self) -> bool {
        self.normalized_boot_id().is_some() && self.start_time.is_some()
    }

    /// PID equality never establishes identity on its own.
    pub fn correlate(&self, other: &Self) -> Correlation {
        let boot = self.normalized_boot_id();
        let other_boot = other.normalized_boot_id();
        if self.pid != other.pid
            || matches!((boot, other_boot), (Some(a), Some(b)) if a != b)
            || matches!((self.start_time, other.start_time), (Some(a), Some(b)) if a != b)
        {
            return Correlation::DifferentInstance;
        }
        if boot.is_some()
            && other_boot.is_some()
            && self.start_time.is_some()
            && other.start_time.is_some()
        {
            Correlation::SameInstance
        } else {
            Correlation::Indeterminate
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolCategory {
    CodingAssistant,
    LocalInferenceRuntime,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolIdentity {
    /// A catalog identifier; fixture identifiers must be clearly synthetic.
    pub id: String,
    pub category: ToolCategory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceClass {
    InventoryIndicator,
    Candidate,
    Corroborated,
    ConfirmedSupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleApproval {
    Unreviewed,
    /// Approval for a diagnostic closure suggestion, never session enforcement.
    DiagnosticClosure,
}

/// A reference to an upstream rule assessment, NOT a rule matcher or proof.
/// Merely constructing this value does not authenticate a catalog or artifact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleMatch {
    pub catalog_version: String,
    pub rule_id: String,
    pub rule_version: String,
    pub approval: RuleApproval,
}

impl RuleMatch {
    pub(crate) fn permits_closure(&self) -> bool {
        self.approval == RuleApproval::DiagnosticClosure
            && [&self.catalog_version, &self.rule_id, &self.rule_version]
                .iter()
                .all(|value| !value.trim().is_empty())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityAssessment {
    pub tool: Option<ToolIdentity>,
    pub class: EvidenceClass,
    /// Required for a supported execution suggestion. Names/paths/package
    /// metadata alone must remain candidates regardless of a rule reference.
    pub rule_match: Option<RuleMatch>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationSource {
    PackageMetadata,
    ExecutableArtifact,
    ProcessSnapshot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState {
    AliveAtSnapshot,
    ExitedDuringEnrichment,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservationKind {
    Installation,
    Process {
        instance: ProcessInstance,
        state: ProcessState,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub scope: ScopeId,
    pub source: ObservationSource,
    pub kind: ObservationKind,
    pub identity: IdentityAssessment,
}

/// A point-in-time input. No consent receipt is implied by constructing it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub scan_id: String,
    /// UTC milliseconds since Unix epoch, supplied by the future caller.
    pub observed_at_ms: u64,
    pub platform: Platform,
    pub scopes: Vec<DeclaredScope>,
    pub requested: Vec<CoverageTarget>,
    pub execution: ScanExecution,
    pub coverage: Vec<CollectorCoverage>,
    pub observations: Vec<Observation>,
}
