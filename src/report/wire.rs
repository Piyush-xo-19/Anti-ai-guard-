//! Explicit, versioned wire names; never derive them from Debug.
use super::*;
#[derive(Clone, Copy, Serialize)]
pub(super) enum Os {
    #[serde(rename = "Windows10")]
    Windows10,
    #[serde(rename = "Windows11")]
    Windows11,
    #[serde(rename = "WindowsServer")]
    WindowsServer,
    #[serde(rename = "Linux")]
    Linux,
    #[serde(rename = "Unknown")]
    Unknown,
}
impl From<OperatingSystem> for Os {
    fn from(value: OperatingSystem) -> Self {
        match value {
            OperatingSystem::Windows10 => Self::Windows10,
            OperatingSystem::Windows11 => Self::Windows11,
            OperatingSystem::WindowsServer => Self::WindowsServer,
            OperatingSystem::Linux => Self::Linux,
            OperatingSystem::Unknown => Self::Unknown,
        }
    }
}
impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Windows10 => "Windows10",
            Self::Windows11 => "Windows11",
            Self::WindowsServer => "WindowsServer",
            Self::Linux => "Linux",
            Self::Unknown => "Unknown",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum Arch {
    #[serde(rename = "X64")]
    X64,
    #[serde(rename = "Arm64")]
    Arm64,
    #[serde(rename = "X86")]
    X86,
    #[serde(rename = "Unknown")]
    Unknown,
}
impl From<Architecture> for Arch {
    fn from(value: Architecture) -> Self {
        match value {
            Architecture::X64 => Self::X64,
            Architecture::Arm64 => Self::Arm64,
            Architecture::X86 => Self::X86,
            Architecture::Unknown => Self::Unknown,
        }
    }
}
impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::X64 => "X64",
            Self::Arm64 => "Arm64",
            Self::X86 => "X86",
            Self::Unknown => "Unknown",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum Mode {
    #[serde(rename = "StandardUser")]
    StandardUser,
    #[serde(rename = "ElevatedDiagnostic")]
    ElevatedDiagnostic,
    #[serde(rename = "Unknown")]
    Unknown,
}
impl From<InspectionMode> for Mode {
    fn from(value: InspectionMode) -> Self {
        match value {
            InspectionMode::StandardUser => Self::StandardUser,
            InspectionMode::ElevatedDiagnostic => Self::ElevatedDiagnostic,
            InspectionMode::Unknown => Self::Unknown,
        }
    }
}
impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::StandardUser => "StandardUser",
            Self::ElevatedDiagnostic => "ElevatedDiagnostic",
            Self::Unknown => "Unknown",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum ScopeType {
    #[serde(rename = "InstallationRoot")]
    InstallationRoot,
    #[serde(rename = "CurrentSessionProcesses")]
    CurrentSessionProcesses,
}
impl From<ScopeKind> for ScopeType {
    fn from(value: ScopeKind) -> Self {
        match value {
            ScopeKind::InstallationRoot => Self::InstallationRoot,
            ScopeKind::CurrentSessionProcesses => Self::CurrentSessionProcesses,
        }
    }
}
impl fmt::Display for ScopeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InstallationRoot => "InstallationRoot",
            Self::CurrentSessionProcesses => "CurrentSessionProcesses",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum CollectorType {
    #[serde(rename = "InstallationInventory")]
    InstallationInventory,
    #[serde(rename = "ProcessSnapshot")]
    ProcessSnapshot,
    #[serde(rename = "RuntimeEvents")]
    RuntimeEvents,
}
impl From<Collector> for CollectorType {
    fn from(value: Collector) -> Self {
        match value {
            Collector::InstallationInventory => Self::InstallationInventory,
            Collector::ProcessSnapshot => Self::ProcessSnapshot,
            Collector::RuntimeEvents => Self::RuntimeEvents,
        }
    }
}
impl fmt::Display for CollectorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InstallationInventory => "InstallationInventory",
            Self::ProcessSnapshot => "ProcessSnapshot",
            Self::RuntimeEvents => "RuntimeEvents",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum CollectorState {
    #[serde(rename = "Inspected")]
    Inspected,
    #[serde(rename = "Partial")]
    Partial,
    #[serde(rename = "Unavailable")]
    Unavailable,
    #[serde(rename = "NotRequested")]
    NotRequested,
    #[serde(rename = "OutsideMilestone")]
    OutsideMilestone,
}
impl From<CoverageState> for CollectorState {
    fn from(value: CoverageState) -> Self {
        match value {
            CoverageState::Inspected => Self::Inspected,
            CoverageState::Partial => Self::Partial,
            CoverageState::Unavailable => Self::Unavailable,
            CoverageState::NotRequested => Self::NotRequested,
            CoverageState::OutsideMilestone => Self::OutsideMilestone,
        }
    }
}
impl fmt::Display for CollectorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Inspected => "Inspected",
            Self::Partial => "Partial",
            Self::Unavailable => "Unavailable",
            Self::NotRequested => "NotRequested",
            Self::OutsideMilestone => "OutsideMilestone",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum Gap {
    #[serde(rename = "AccessDenied")]
    AccessDenied,
    #[serde(rename = "ProcessExited")]
    ProcessExited,
    #[serde(rename = "MissingStartIdentity")]
    MissingStartIdentity,
    #[serde(rename = "ScopeBoundary")]
    ScopeBoundary,
    #[serde(rename = "BudgetExceeded")]
    BudgetExceeded,
    #[serde(rename = "Cancelled")]
    Cancelled,
    #[serde(rename = "CollectorFailed")]
    CollectorFailed,
    #[serde(rename = "Unsupported")]
    Unsupported,
    #[serde(rename = "InvalidMetadata")]
    InvalidMetadata,
    #[serde(rename = "MissingRoot")]
    MissingRoot,
    #[serde(rename = "InvalidScope")]
    InvalidScope,
    #[serde(rename = "SharingConflict")]
    SharingConflict,
    #[serde(rename = "MissingBootIdentity")]
    MissingBootIdentity,
    #[serde(rename = "ProcessIdentityChanged")]
    ProcessIdentityChanged,
}
impl From<GapReason> for Gap {
    fn from(value: GapReason) -> Self {
        match value {
            GapReason::AccessDenied => Self::AccessDenied,
            GapReason::ProcessExited => Self::ProcessExited,
            GapReason::MissingStartIdentity => Self::MissingStartIdentity,
            GapReason::ScopeBoundary => Self::ScopeBoundary,
            GapReason::BudgetExceeded => Self::BudgetExceeded,
            GapReason::Cancelled => Self::Cancelled,
            GapReason::CollectorFailed => Self::CollectorFailed,
            GapReason::Unsupported => Self::Unsupported,
            GapReason::InvalidMetadata => Self::InvalidMetadata,
            GapReason::MissingRoot => Self::MissingRoot,
            GapReason::InvalidScope => Self::InvalidScope,
            GapReason::SharingConflict => Self::SharingConflict,
            GapReason::MissingBootIdentity => Self::MissingBootIdentity,
            GapReason::ProcessIdentityChanged => Self::ProcessIdentityChanged,
        }
    }
}
impl fmt::Display for Gap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AccessDenied => "AccessDenied",
            Self::ProcessExited => "ProcessExited",
            Self::MissingStartIdentity => "MissingStartIdentity",
            Self::ScopeBoundary => "ScopeBoundary",
            Self::BudgetExceeded => "BudgetExceeded",
            Self::Cancelled => "Cancelled",
            Self::CollectorFailed => "CollectorFailed",
            Self::Unsupported => "Unsupported",
            Self::InvalidMetadata => "InvalidMetadata",
            Self::MissingRoot => "MissingRoot",
            Self::InvalidScope => "InvalidScope",
            Self::SharingConflict => "SharingConflict",
            Self::MissingBootIdentity => "MissingBootIdentity",
            Self::ProcessIdentityChanged => "ProcessIdentityChanged",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum Execution {
    #[serde(rename = "Finished")]
    Finished,
    #[serde(rename = "Partial")]
    Partial,
    #[serde(rename = "Failed")]
    Failed,
    #[serde(rename = "Cancelled")]
    Cancelled,
}
impl From<ScanExecution> for Execution {
    fn from(value: ScanExecution) -> Self {
        match value {
            ScanExecution::Finished => Self::Finished,
            ScanExecution::Partial => Self::Partial,
            ScanExecution::Failed => Self::Failed,
            ScanExecution::Cancelled => Self::Cancelled,
        }
    }
}
impl fmt::Display for Execution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Finished => "Finished",
            Self::Partial => "Partial",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum Coverage {
    #[serde(rename = "InspectedRequestedScope")]
    InspectedRequestedScope,
    #[serde(rename = "Incomplete")]
    Incomplete,
    #[serde(rename = "NotRequested")]
    NotRequested,
}
impl From<CoverageSummary> for Coverage {
    fn from(value: CoverageSummary) -> Self {
        match value {
            CoverageSummary::InspectedRequestedScope => Self::InspectedRequestedScope,
            CoverageSummary::Incomplete => Self::Incomplete,
            CoverageSummary::NotRequested => Self::NotRequested,
        }
    }
}
impl fmt::Display for Coverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InspectedRequestedScope => "InspectedRequestedScope",
            Self::Incomplete => "Incomplete",
            Self::NotRequested => "NotRequested",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum Source {
    #[serde(rename = "PackageMetadata")]
    PackageMetadata,
    #[serde(rename = "ExecutableArtifact")]
    ExecutableArtifact,
    #[serde(rename = "ProcessSnapshot")]
    ProcessSnapshot,
}
impl From<ObservationSource> for Source {
    fn from(value: ObservationSource) -> Self {
        match value {
            ObservationSource::PackageMetadata => Self::PackageMetadata,
            ObservationSource::ExecutableArtifact => Self::ExecutableArtifact,
            ObservationSource::ProcessSnapshot => Self::ProcessSnapshot,
        }
    }
}
impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::PackageMetadata => "PackageMetadata",
            Self::ExecutableArtifact => "ExecutableArtifact",
            Self::ProcessSnapshot => "ProcessSnapshot",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum Evidence {
    #[serde(rename = "InventoryIndicator")]
    InventoryIndicator,
    #[serde(rename = "Candidate")]
    Candidate,
    #[serde(rename = "Corroborated")]
    Corroborated,
    #[serde(rename = "ConfirmedSupported")]
    ConfirmedSupported,
}
impl From<EvidenceClass> for Evidence {
    fn from(value: EvidenceClass) -> Self {
        match value {
            EvidenceClass::InventoryIndicator => Self::InventoryIndicator,
            EvidenceClass::Candidate => Self::Candidate,
            EvidenceClass::Corroborated => Self::Corroborated,
            EvidenceClass::ConfirmedSupported => Self::ConfirmedSupported,
        }
    }
}
impl fmt::Display for Evidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InventoryIndicator => "InventoryIndicator",
            Self::Candidate => "Candidate",
            Self::Corroborated => "Corroborated",
            Self::ConfirmedSupported => "ConfirmedSupported",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum State {
    #[serde(rename = "AliveAtSnapshot")]
    AliveAtSnapshot,
    #[serde(rename = "ExitedDuringEnrichment")]
    ExitedDuringEnrichment,
}
impl From<ProcessState> for State {
    fn from(value: ProcessState) -> Self {
        match value {
            ProcessState::AliveAtSnapshot => Self::AliveAtSnapshot,
            ProcessState::ExitedDuringEnrichment => Self::ExitedDuringEnrichment,
        }
    }
}
impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AliveAtSnapshot => "AliveAtSnapshot",
            Self::ExitedDuringEnrichment => "ExitedDuringEnrichment",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum ActionWire {
    #[serde(rename = "InventoryOnly")]
    InventoryOnly,
    #[serde(rename = "NoAction")]
    NoAction,
    #[serde(rename = "RequestReview")]
    RequestReview,
    #[serde(rename = "RequestClosureAndRescan")]
    RequestClosureAndRescan,
    #[serde(rename = "RecheckProcess")]
    RecheckProcess,
}
impl From<SuggestedAction> for ActionWire {
    fn from(value: SuggestedAction) -> Self {
        match value {
            SuggestedAction::InventoryOnly => Self::InventoryOnly,
            SuggestedAction::NoAction => Self::NoAction,
            SuggestedAction::RequestReview => Self::RequestReview,
            SuggestedAction::RequestClosureAndRescan => Self::RequestClosureAndRescan,
            SuggestedAction::RecheckProcess => Self::RecheckProcess,
        }
    }
}
impl fmt::Display for ActionWire {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InventoryOnly => "InventoryOnly",
            Self::NoAction => "NoAction",
            Self::RequestReview => "RequestReview",
            Self::RequestClosureAndRescan => "RequestClosureAndRescan",
            Self::RecheckProcess => "RecheckProcess",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum Reason {
    #[serde(rename = "InstallationIsNotExecution")]
    InstallationIsNotExecution,
    #[serde(rename = "NoRecognizedIdentity")]
    NoRecognizedIdentity,
    #[serde(rename = "IdentityNeedsReview")]
    IdentityNeedsReview,
    #[serde(rename = "NotACodingAssistant")]
    NotACodingAssistant,
    #[serde(rename = "SupportedAssistantObserved")]
    SupportedAssistantObserved,
    #[serde(rename = "ProcessNoLongerPresent")]
    ProcessNoLongerPresent,
    #[serde(rename = "MissingProcessIdentity")]
    MissingProcessIdentity,
    #[serde(rename = "ObservationOutsideScope")]
    ObservationOutsideScope,
    #[serde(rename = "InvalidObservationSource")]
    InvalidObservationSource,
    #[serde(rename = "MissingSuggestion")]
    MissingSuggestion,
}
impl From<ActionReason> for Reason {
    fn from(value: ActionReason) -> Self {
        match value {
            ActionReason::InstallationIsNotExecution => Self::InstallationIsNotExecution,
            ActionReason::NoRecognizedIdentity => Self::NoRecognizedIdentity,
            ActionReason::IdentityNeedsReview => Self::IdentityNeedsReview,
            ActionReason::NotACodingAssistant => Self::NotACodingAssistant,
            ActionReason::SupportedAssistantObserved => Self::SupportedAssistantObserved,
            ActionReason::ProcessNoLongerPresent => Self::ProcessNoLongerPresent,
            ActionReason::MissingProcessIdentity => Self::MissingProcessIdentity,
            ActionReason::ObservationOutsideScope => Self::ObservationOutsideScope,
            ActionReason::InvalidObservationSource => Self::InvalidObservationSource,
        }
    }
}
impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InstallationIsNotExecution => "InstallationIsNotExecution",
            Self::NoRecognizedIdentity => "NoRecognizedIdentity",
            Self::IdentityNeedsReview => "IdentityNeedsReview",
            Self::NotACodingAssistant => "NotACodingAssistant",
            Self::SupportedAssistantObserved => "SupportedAssistantObserved",
            Self::ProcessNoLongerPresent => "ProcessNoLongerPresent",
            Self::MissingProcessIdentity => "MissingProcessIdentity",
            Self::ObservationOutsideScope => "ObservationOutsideScope",
            Self::InvalidObservationSource => "InvalidObservationSource",
            Self::MissingSuggestion => "MissingSuggestion",
        })
    }
}
#[derive(Clone, Copy, Serialize)]
pub(super) enum ConcernReason {
    #[serde(rename = "InvalidScanId")]
    InvalidScanId,
    #[serde(rename = "CollectorOutsideMilestone")]
    CollectorOutsideMilestone,
    #[serde(rename = "UnrequestedRecord")]
    UnrequestedRecord,
    #[serde(rename = "ScanNotFinished")]
    ScanNotFinished,
    #[serde(rename = "InvalidScope")]
    InvalidScope,
    #[serde(rename = "DuplicateRequest")]
    DuplicateRequest,
    #[serde(rename = "MissingRecord")]
    MissingRecord,
    #[serde(rename = "DuplicateRecord")]
    DuplicateRecord,
    #[serde(rename = "CollectorIncomplete")]
    CollectorIncomplete,
    #[serde(rename = "ObservationOutsideScope")]
    ObservationOutsideScope,
    #[serde(rename = "MissingProcessIdentity")]
    MissingProcessIdentity,
    #[serde(rename = "ProcessExited")]
    ProcessExited,
    #[serde(rename = "InvalidObservationSource")]
    InvalidObservationSource,
}
impl From<CoverageConcernReason> for ConcernReason {
    fn from(value: CoverageConcernReason) -> Self {
        match value {
            CoverageConcernReason::InvalidScanId => Self::InvalidScanId,
            CoverageConcernReason::CollectorOutsideMilestone => Self::CollectorOutsideMilestone,
            CoverageConcernReason::UnrequestedRecord => Self::UnrequestedRecord,
            CoverageConcernReason::ScanNotFinished => Self::ScanNotFinished,
            CoverageConcernReason::InvalidScope => Self::InvalidScope,
            CoverageConcernReason::DuplicateRequest => Self::DuplicateRequest,
            CoverageConcernReason::MissingRecord => Self::MissingRecord,
            CoverageConcernReason::DuplicateRecord => Self::DuplicateRecord,
            CoverageConcernReason::CollectorIncomplete => Self::CollectorIncomplete,
            CoverageConcernReason::ObservationOutsideScope => Self::ObservationOutsideScope,
            CoverageConcernReason::MissingProcessIdentity => Self::MissingProcessIdentity,
            CoverageConcernReason::ProcessExited => Self::ProcessExited,
            CoverageConcernReason::InvalidObservationSource => Self::InvalidObservationSource,
        }
    }
}
impl fmt::Display for ConcernReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidScanId => "InvalidScanId",
            Self::CollectorOutsideMilestone => "CollectorOutsideMilestone",
            Self::UnrequestedRecord => "UnrequestedRecord",
            Self::ScanNotFinished => "ScanNotFinished",
            Self::InvalidScope => "InvalidScope",
            Self::DuplicateRequest => "DuplicateRequest",
            Self::MissingRecord => "MissingRecord",
            Self::DuplicateRecord => "DuplicateRecord",
            Self::CollectorIncomplete => "CollectorIncomplete",
            Self::ObservationOutsideScope => "ObservationOutsideScope",
            Self::MissingProcessIdentity => "MissingProcessIdentity",
            Self::ProcessExited => "ProcessExited",
            Self::InvalidObservationSource => "InvalidObservationSource",
        })
    }
}
