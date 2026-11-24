//! Pure runtime observation and health contracts. The caller supplies minimized
//! events and elapsed times from a single monotonic clock for the run.

pub mod reconcile;

use crate::model::ProcessInstance;
use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::time::{Duration, SystemTime};

/// An opaque identifier assigned by the run owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunId(pub u64);

/// An opaque attachment identifier, unique within its run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceGeneration(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamId {
    pub run: RunId,
    pub generation: SourceGeneration,
}

/// An immutable delivery identifier, unique within a stream. It can repeat on
/// redelivery of the same event and carries no process identity or ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReceiptId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessEventKind {
    Started,
    Stopped,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessEvent {
    pub receipt: ReceiptId,
    pub kind: ProcessEventKind,
    /// Source observation time, if the adapter can express it as wall time.
    /// It may differ from receive time and is never used for the run deadline.
    pub observed_at: Option<SystemTime>,
    /// Creation identity may be incomplete. Use ProcessInstance::correlate for
    /// identity within a reviewed, comparable boot and process time domain.
    pub instance: ProcessInstance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LossReporting {
    Observable,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loss {
    SourceReported,
    QueueOverflow,
    ReceiptLimit,
    ReconciliationLimit,
    InvalidMetadata,
    BaselineFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Degradation {
    ContinuityUnknown,
    Interrupted(Loss),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceError {
    AccessDenied,
    Disconnected,
    Unsupported,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    Cancelled,
    Deadline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Health {
    Starting,
    /// The source is ready, a subsequent baseline reconciled, and no loss has
    /// been reported by the observable controls. This is a diagnostic state.
    Healthy,
    Degraded(Degradation),
    Unavailable(SourceError),
    Stopped(StopReason),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeEvent {
    /// The caller confirmed actual source readiness through its reviewed probe.
    SourceReady(LossReporting),
    /// The caller completed and reconciled the baseline for this stream.
    BaselineComplete,
    Process(ProcessEvent),
    Loss(Loss),
    SourceUnavailable(SourceError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Applied,
    OtherStream,
    Inactive,
    AlreadyReady,
    BaselineBeforeReadiness,
    DuplicateReceipt,
    ReceiptLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Elapsed time from run start at which observation stops.
    pub deadline: Duration,
    /// Maximum unique process receipts retained for this generation.
    pub max_receipts: NonZeroUsize,
}

/// Reduces one stream. Reattachment uses a new reducer and generation.
/// Only receipt IDs are retained, with no queue or process table.
pub struct HealthReducer {
    stream: StreamId,
    limits: Limits,
    source: Option<(LossReporting, Duration)>,
    baseline_complete: bool,
    receipts: BTreeSet<ReceiptId>,
    first_loss: Option<Loss>,
    error: Option<SourceError>,
    stop: Option<StopReason>,
}

impl HealthReducer {
    pub fn new(stream: StreamId, limits: Limits) -> Self {
        let mut reducer = Self {
            stream,
            limits,
            source: None,
            baseline_complete: false,
            receipts: BTreeSet::new(),
            first_loss: None,
            error: None,
            stop: None,
        };
        reducer.advance(Duration::ZERO);
        reducer
    }

    pub fn health(&self) -> Health {
        if let Some(reason) = self.stop {
            return Health::Stopped(reason);
        }
        if let Some(error) = self.error {
            return Health::Unavailable(error);
        }
        if let Some(loss) = self.first_loss {
            return Health::Degraded(Degradation::Interrupted(loss));
        }
        match self.source.map(|(reporting, _)| reporting) {
            Some(LossReporting::Unknown) => Health::Degraded(Degradation::ContinuityUnknown),
            Some(LossReporting::Observable) if self.baseline_complete => Health::Healthy,
            _ => Health::Starting,
        }
    }

    /// The first known interruption remains visible after stopping or failure.
    pub fn first_loss(&self) -> Option<Loss> {
        self.first_loss
    }

    /// Advance the run deadline even when no source events arrive.
    pub fn advance(&mut self, elapsed: Duration) {
        if elapsed >= self.limits.deadline {
            self.stop.get_or_insert(StopReason::Deadline);
        }
    }

    pub fn cancel(&mut self) {
        self.stop.get_or_insert(StopReason::Cancelled);
    }

    /// The receipt time is elapsed time from run start, supplied at ingress.
    /// Delivery order and wall time are carried independently of health.
    /// An event at or after the deadline is discarded. Callers must also advance
    /// with their current monotonic clock before draining a delayed queue.
    pub fn apply(
        &mut self,
        stream: StreamId,
        received_at: Duration,
        event: &RuntimeEvent,
    ) -> Outcome {
        if stream != self.stream {
            return Outcome::OtherStream;
        }
        self.advance(received_at);
        if self.stop.is_some() || self.error.is_some() {
            return Outcome::Inactive;
        }
        match event {
            RuntimeEvent::SourceReady(_) if self.source.is_some() => return Outcome::AlreadyReady,
            RuntimeEvent::SourceReady(reporting) => self.source = Some((*reporting, received_at)),
            RuntimeEvent::BaselineComplete
                if !self
                    .source
                    .is_some_and(|(_, ready_at)| received_at >= ready_at) =>
            {
                return Outcome::BaselineBeforeReadiness;
            }
            RuntimeEvent::BaselineComplete => self.baseline_complete = true,
            RuntimeEvent::Process(event) => return self.record_receipt(event.receipt),
            RuntimeEvent::Loss(loss) => {
                self.first_loss.get_or_insert(*loss);
            }
            RuntimeEvent::SourceUnavailable(error) => self.error = Some(*error),
        }
        Outcome::Applied
    }

    fn record_receipt(&mut self, receipt: ReceiptId) -> Outcome {
        if self.receipts.contains(&receipt) {
            return Outcome::DuplicateReceipt;
        }
        if self.receipts.len() == self.limits.max_receipts.get() {
            self.first_loss.get_or_insert(Loss::ReceiptLimit);
            return Outcome::ReceiptLimit;
        }
        self.receipts.insert(receipt);
        Outcome::Applied
    }
}
