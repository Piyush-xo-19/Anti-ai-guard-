//! Bounded, source-independent buffering and baseline reconciliation.
//! The caller supplies minimized instances in the same reviewed identity domain.

use super::{
    Health, HealthReducer, Limits, Loss, Outcome, ProcessEvent, ProcessEventKind, RuntimeEvent,
    StreamId,
};
use crate::model::{Correlation, ProcessInstance};
use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReconcileLimits {
    pub max_pending: NonZeroUsize,
    pub max_instances: NonZeroUsize,
    pub max_boot_id_bytes: NonZeroUsize,
}

/// Observed evidence for an instance. These flags do not assert current liveness.
/// Incomplete identities remain separate records even when their PIDs match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessEvidence {
    pub instance: ProcessInstance,
    pub at_baseline: bool,
    pub start_observed: bool,
    pub stop_observed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconcileError {
    SourceNotReady,
    BaselineAlreadyStarted,
    BaselineNotStarted,
    BaselineMethodRequired,
    InvalidTime,
    QueueOverflow,
    InstanceLimit,
    MetadataLimit,
}

#[derive(Clone, Copy)]
enum Phase {
    Waiting,
    Capturing(Duration),
    Live,
}

/// One run and source generation. Use a single owner for all calls.
/// A later source adapter must supply its own synchronized callback transport.
pub struct BaselineReconciler {
    stream: StreamId,
    health: HealthReducer,
    limits: ReconcileLimits,
    ready_at: Option<Duration>,
    phase: Phase,
    pending: VecDeque<ProcessEvent>,
    records: Vec<ProcessEvidence>,
}

impl BaselineReconciler {
    pub fn new(stream: StreamId, health_limits: Limits, limits: ReconcileLimits) -> Self {
        Self {
            stream,
            health: HealthReducer::new(stream, health_limits),
            limits,
            ready_at: None,
            phase: Phase::Waiting,
            pending: VecDeque::new(),
            records: Vec::new(),
        }
    }

    pub fn health(&self) -> Health {
        self.health.health()
    }

    pub fn first_loss(&self) -> Option<Loss> {
        self.health.first_loss()
    }

    pub fn records(&self) -> &[ProcessEvidence] {
        &self.records
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    /// Advance from the current monotonic clock, including during source silence.
    pub fn advance(&mut self, elapsed: Duration) {
        self.health.advance(elapsed);
        if self.inactive() {
            self.pending.clear();
        }
    }

    pub fn cancel(&mut self) {
        self.health.cancel();
        self.pending.clear();
    }

    /// Receive time is elapsed from run start. Advance the current clock before
    /// draining delayed input. Baseline completion goes through finish_baseline.
    pub fn receive(
        &mut self,
        stream: StreamId,
        received_at: Duration,
        event: &RuntimeEvent,
    ) -> Result<Outcome, ReconcileError> {
        if stream != self.stream {
            return Ok(Outcome::OtherStream);
        }
        self.advance(received_at);
        if self.inactive() {
            return Ok(Outcome::Inactive);
        }
        if matches!(event, RuntimeEvent::BaselineComplete) {
            return Err(ReconcileError::BaselineMethodRequired);
        }
        let outcome = self.health.apply(stream, received_at, event);
        if outcome != Outcome::Applied {
            return Ok(outcome);
        }
        match event {
            RuntimeEvent::SourceReady(_) => self.ready_at = Some(received_at),
            RuntimeEvent::Process(process) => {
                if !self.valid_metadata(&process.instance) {
                    self.mark_loss(received_at, Loss::InvalidMetadata);
                    return Err(ReconcileError::MetadataLimit);
                }
                if matches!(self.phase, Phase::Live) {
                    if let Err(error) = merge_record(
                        &mut self.records,
                        &process.instance,
                        Some(process.kind),
                        self.limits.max_instances.get(),
                    ) {
                        self.mark_loss(received_at, Loss::ReconciliationLimit);
                        return Err(error);
                    }
                } else if self.pending.len() < self.limits.max_pending.get() {
                    self.pending.push_back(process.clone());
                } else {
                    self.mark_loss(received_at, Loss::QueueOverflow);
                    return Err(ReconcileError::QueueOverflow);
                }
            }
            RuntimeEvent::SourceUnavailable(_) => self.pending.clear(),
            _ => {}
        }
        Ok(outcome)
    }

    /// Call before capturing the snapshot, after the source is confirmed ready.
    pub fn begin_baseline(
        &mut self,
        stream: StreamId,
        at: Duration,
    ) -> Result<Outcome, ReconcileError> {
        if stream != self.stream {
            return Ok(Outcome::OtherStream);
        }
        self.advance(at);
        if self.inactive() {
            return Ok(Outcome::Inactive);
        }
        let ready_at = self.ready_at.ok_or(ReconcileError::SourceNotReady)?;
        if at < ready_at {
            return Err(ReconcileError::InvalidTime);
        }
        if !matches!(self.phase, Phase::Waiting) {
            return Err(ReconcileError::BaselineAlreadyStarted);
        }
        self.phase = Phase::Capturing(at);
        Ok(Outcome::Applied)
    }

    /// Reconcile a completed, successful baseline and all buffered events.
    /// The snapshot owner must report failed/partial capture with report_baseline_failure.
    /// Bounded validation builds a temporary record set before publishing it.
    pub fn finish_baseline(
        &mut self,
        stream: StreamId,
        at: Duration,
        instances: &[ProcessInstance],
    ) -> Result<Outcome, ReconcileError> {
        if stream != self.stream {
            return Ok(Outcome::OtherStream);
        }
        self.advance(at);
        if self.inactive() {
            return Ok(Outcome::Inactive);
        }
        let Phase::Capturing(started_at) = self.phase else {
            return Err(ReconcileError::BaselineNotStarted);
        };
        if at < started_at {
            return Err(ReconcileError::InvalidTime);
        }
        if instances.len() > self.limits.max_instances.get() {
            self.mark_loss(at, Loss::ReconciliationLimit);
            return Err(ReconcileError::InstanceLimit);
        }
        if instances
            .iter()
            .any(|instance| !self.valid_metadata(instance))
        {
            self.mark_loss(at, Loss::InvalidMetadata);
            return Err(ReconcileError::MetadataLimit);
        }
        let mut records = Vec::new();
        let baseline = instances.iter().map(|instance| (instance, None));
        let buffered = self
            .pending
            .iter()
            .map(|event| (&event.instance, Some(event.kind)));
        let result = baseline.chain(buffered).try_for_each(|(instance, kind)| {
            merge_record(
                &mut records,
                instance,
                kind,
                self.limits.max_instances.get(),
            )
        });
        if let Err(error) = result {
            self.mark_loss(at, Loss::ReconciliationLimit);
            return Err(error);
        }
        self.records = records;
        self.pending.clear();
        self.phase = Phase::Live;
        Ok(self
            .health
            .apply(stream, at, &RuntimeEvent::BaselineComplete))
    }

    /// A retry in this generation cannot erase this uncertainty.
    pub fn report_baseline_failure(&mut self, stream: StreamId, at: Duration) -> Outcome {
        if stream != self.stream {
            return Outcome::OtherStream;
        }
        self.advance(at);
        self.health
            .apply(stream, at, &RuntimeEvent::Loss(Loss::BaselineFailed))
    }

    fn valid_metadata(&self, instance: &ProcessInstance) -> bool {
        instance
            .boot_id
            .as_ref()
            .is_none_or(|id| id.len() <= self.limits.max_boot_id_bytes.get())
    }

    fn mark_loss(&mut self, at: Duration, loss: Loss) {
        self.health
            .apply(self.stream, at, &RuntimeEvent::Loss(loss));
    }

    fn inactive(&self) -> bool {
        matches!(self.health(), Health::Stopped(_) | Health::Unavailable(_))
    }
}

fn merge_record(
    records: &mut Vec<ProcessEvidence>,
    instance: &ProcessInstance,
    kind: Option<ProcessEventKind>,
    max_instances: usize,
) -> Result<(), ReconcileError> {
    let existing = records
        .iter()
        .position(|record| record.instance.correlate(instance) == Correlation::SameInstance);
    let record = if let Some(index) = existing {
        &mut records[index]
    } else {
        if records.len() == max_instances {
            return Err(ReconcileError::InstanceLimit);
        }
        records.push(ProcessEvidence {
            instance: instance.clone(),
            at_baseline: false,
            start_observed: false,
            stop_observed: false,
        });
        records.last_mut().expect("record was just inserted")
    };
    match kind {
        None => record.at_baseline = true,
        Some(ProcessEventKind::Started) => record.start_observed = true,
        Some(ProcessEventKind::Stopped) => record.stop_observed = true,
    }
    Ok(())
}
