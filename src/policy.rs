//! Pure diagnostic Policy A evaluation. No session transitions or OS actions.

use crate::model::*;

pub const POLICY_VERSION: &str = "policy-a-diagnostic-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuggestedAction {
    InventoryOnly,
    NoAction,
    RequestReview,
    RequestClosureAndRescan,
    RecheckProcess,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionReason {
    InstallationIsNotExecution,
    NoRecognizedIdentity,
    IdentityNeedsReview,
    NotACodingAssistant,
    SupportedAssistantObserved,
    ProcessNoLongerPresent,
    MissingProcessIdentity,
    ObservationOutsideScope,
    InvalidObservationSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Suggestion {
    pub action: SuggestedAction,
    pub reason: ActionReason,
}

/// Evaluate one already-scoped, preclassified observation. Prefer `evaluate`
/// when scope and coverage checks are needed. Never infers execution from inventory.
pub fn suggest_action(observation: &Observation) -> Suggestion {
    if !source_matches_kind(observation) {
        return Suggestion {
            action: SuggestedAction::RequestReview,
            reason: ActionReason::InvalidObservationSource,
        };
    }
    let (action, reason) = match &observation.kind {
        ObservationKind::Installation => (
            SuggestedAction::InventoryOnly,
            ActionReason::InstallationIsNotExecution,
        ),
        ObservationKind::Process { instance, state } => match &observation.identity.tool {
            None if observation.identity.class == EvidenceClass::InventoryIndicator => (
                SuggestedAction::NoAction,
                ActionReason::NoRecognizedIdentity,
            ),
            None => (
                SuggestedAction::RequestReview,
                ActionReason::IdentityNeedsReview,
            ),
            Some(tool) if tool.category != ToolCategory::CodingAssistant => {
                (SuggestedAction::NoAction, ActionReason::NotACodingAssistant)
            }
            Some(_) if *state == ProcessState::ExitedDuringEnrichment => (
                SuggestedAction::RecheckProcess,
                ActionReason::ProcessNoLongerPresent,
            ),
            Some(_) if !instance.has_stable_identity() => (
                SuggestedAction::RequestReview,
                ActionReason::MissingProcessIdentity,
            ),
            Some(tool)
                if !tool.id.trim().is_empty()
                    && matches!(
                        observation.identity.class,
                        EvidenceClass::Corroborated | EvidenceClass::ConfirmedSupported
                    )
                    && observation
                        .identity
                        .rule_match
                        .as_ref()
                        .is_some_and(RuleMatch::permits_closure) =>
            {
                (
                    SuggestedAction::RequestClosureAndRescan,
                    ActionReason::SupportedAssistantObserved,
                )
            }
            Some(_) => (
                SuggestedAction::RequestReview,
                ActionReason::IdentityNeedsReview,
            ),
        },
    };
    Suggestion { action, reason }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverageSummary {
    /// Only the explicitly requested scopes; never means the device is clean.
    InspectedRequestedScope,
    Incomplete,
    NotRequested,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eligibility {
    /// M2 has no API value capable of granting or suspending verification.
    NotIssued,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverageConcernReason {
    InvalidScanId,
    CollectorOutsideMilestone,
    UnrequestedRecord,
    ScanNotFinished,
    InvalidScope,
    DuplicateRequest,
    MissingRecord,
    DuplicateRecord,
    CollectorIncomplete,
    ObservationOutsideScope,
    MissingProcessIdentity,
    ProcessExited,
    InvalidObservationSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoverageConcern {
    pub target: Option<CoverageTarget>,
    pub reason: CoverageConcernReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticReport {
    pub schema_version: u32,
    pub policy_version: &'static str,
    pub snapshot: Snapshot,
    /// Same order as snapshot.observations, retaining source/evidence references.
    pub suggestions: Vec<Suggestion>,
    pub coverage: CoverageSummary,
    pub coverage_concerns: Vec<CoverageConcern>,
    pub eligibility: Eligibility,
}

fn source_matches_kind(observation: &Observation) -> bool {
    matches!(
        (&observation.kind, observation.source),
        (
            ObservationKind::Installation,
            ObservationSource::PackageMetadata | ObservationSource::ExecutableArtifact
        ) | (
            ObservationKind::Process { .. },
            ObservationSource::ProcessSnapshot
        )
    )
}

fn target_is_declared(snapshot: &Snapshot, target: CoverageTarget) -> bool {
    let scopes: Vec<_> = snapshot
        .scopes
        .iter()
        .filter(|s| s.id == target.scope)
        .collect();
    matches!(scopes.as_slice(), [scope] if matches!(
        (target.collector, scope.kind),
        (Collector::InstallationInventory, ScopeKind::InstallationRoot)
            | (Collector::ProcessSnapshot, ScopeKind::CurrentSessionProcesses)
            | (Collector::RuntimeEvents, ScopeKind::CurrentSessionProcesses)
    ))
}

fn observation_target(observation: &Observation) -> CoverageTarget {
    CoverageTarget {
        scope: observation.scope,
        collector: match observation.kind {
            ObservationKind::Installation => Collector::InstallationInventory,
            ObservationKind::Process { .. } => Collector::ProcessSnapshot,
        },
    }
}

/// In-memory evaluation only: no rendering, serialization, collection or storage.
pub fn evaluate(snapshot: Snapshot) -> DiagnosticReport {
    let mut concerns = Vec::new();
    if snapshot.scan_id.trim().is_empty() {
        concerns.push(CoverageConcern {
            target: None,
            reason: CoverageConcernReason::InvalidScanId,
        });
    }
    if snapshot.execution != ScanExecution::Finished {
        concerns.push(CoverageConcern {
            target: None,
            reason: CoverageConcernReason::ScanNotFinished,
        });
    }
    let mut checked_targets = Vec::new();
    for target in &snapshot.requested {
        if checked_targets.contains(target) {
            continue;
        }
        checked_targets.push(*target);
        let entries: Vec<_> = snapshot
            .coverage
            .iter()
            .filter(|c| c.target == *target)
            .collect();
        let mut add = |reason| {
            concerns.push(CoverageConcern {
                target: Some(*target),
                reason,
            })
        };
        if !target_is_declared(&snapshot, *target) {
            add(CoverageConcernReason::InvalidScope);
        }
        let outside_milestone = target.collector == Collector::RuntimeEvents;
        if outside_milestone {
            add(CoverageConcernReason::CollectorOutsideMilestone);
        }
        if snapshot.requested.iter().filter(|t| *t == target).count() != 1 {
            add(CoverageConcernReason::DuplicateRequest);
        }
        match entries.as_slice() {
            [] => add(CoverageConcernReason::MissingRecord),
            [_] if outside_milestone => {}
            [entry] if entry.state == CoverageState::OutsideMilestone => {
                add(CoverageConcernReason::CollectorOutsideMilestone)
            }
            [entry] if entry.state == CoverageState::Inspected && entry.gaps.is_empty() => {}
            [_] => add(CoverageConcernReason::CollectorIncomplete),
            _ => add(CoverageConcernReason::DuplicateRecord),
        }
    }
    for entry in &snapshot.coverage {
        if !snapshot.requested.contains(&entry.target)
            && !matches!(
                entry.state,
                CoverageState::NotRequested | CoverageState::OutsideMilestone
            )
        {
            concerns.push(CoverageConcern {
                target: Some(entry.target),
                reason: CoverageConcernReason::UnrequestedRecord,
            });
        }
    }
    let suggestions = snapshot
        .observations
        .iter()
        .map(|observation| {
            let target = observation_target(observation);
            if !snapshot.requested.contains(&target) || !target_is_declared(&snapshot, target) {
                concerns.push(CoverageConcern {
                    target: Some(target),
                    reason: CoverageConcernReason::ObservationOutsideScope,
                });
                Suggestion {
                    action: SuggestedAction::RequestReview,
                    reason: ActionReason::ObservationOutsideScope,
                }
            } else {
                if !source_matches_kind(observation) {
                    concerns.push(CoverageConcern {
                        target: Some(target),
                        reason: CoverageConcernReason::InvalidObservationSource,
                    });
                }
                if let ObservationKind::Process { instance, state } = &observation.kind {
                    if !instance.has_stable_identity() {
                        concerns.push(CoverageConcern {
                            target: Some(target),
                            reason: CoverageConcernReason::MissingProcessIdentity,
                        });
                    }
                    if *state == ProcessState::ExitedDuringEnrichment {
                        concerns.push(CoverageConcern {
                            target: Some(target),
                            reason: CoverageConcernReason::ProcessExited,
                        });
                    }
                }
                suggest_action(observation)
            }
        })
        .collect();
    // Concerns describe a reason per collector/scope, not observation counts.
    // Preserve first-occurrence order while removing identical target/reason pairs.
    let mut unique_concerns = Vec::new();
    concerns.retain(|concern| {
        if unique_concerns.contains(concern) {
            false
        } else {
            unique_concerns.push(*concern);
            true
        }
    });
    let coverage = if !concerns.is_empty() {
        CoverageSummary::Incomplete
    } else if snapshot.requested.is_empty() {
        CoverageSummary::NotRequested
    } else {
        CoverageSummary::InspectedRequestedScope
    };
    DiagnosticReport {
        schema_version: 1,
        policy_version: POLICY_VERSION,
        snapshot,
        suggestions,
        coverage,
        coverage_concerns: concerns,
        eligibility: Eligibility::NotIssued,
    }
}
