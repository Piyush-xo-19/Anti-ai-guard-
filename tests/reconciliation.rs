use anti_ai_guard::model::ProcessInstance;
use anti_ai_guard::runtime::reconcile::{BaselineReconciler, ReconcileError, ReconcileLimits};
use anti_ai_guard::runtime::{
    Degradation, Health, Limits, Loss, LossReporting, Outcome, ProcessEvent, ProcessEventKind,
    ReceiptId, RunId, RuntimeEvent, SourceError, SourceGeneration, StopReason, StreamId,
};
use std::time::{Duration, SystemTime};

const STREAM: StreamId = StreamId {
    run: RunId(1),
    generation: SourceGeneration(1),
};
const READY: RuntimeEvent = RuntimeEvent::SourceReady(LossReporting::Observable);

fn instance(pid: u32, start: u64) -> ProcessInstance {
    ProcessInstance {
        boot_id: Some("synthetic".into()),
        pid,
        start_time: Some(start),
    }
}

fn monitor(pending: usize, instances: usize) -> BaselineReconciler {
    BaselineReconciler::new(
        STREAM,
        Limits {
            deadline: Duration::from_secs(1),
            max_receipts: 32.try_into().unwrap(),
        },
        ReconcileLimits {
            max_pending: pending.try_into().unwrap(),
            max_instances: instances.try_into().unwrap(),
            max_boot_id_bytes: 32.try_into().unwrap(),
        },
    )
}

fn receive(
    monitor: &mut BaselineReconciler,
    at: u64,
    event: &RuntimeEvent,
) -> Result<Outcome, ReconcileError> {
    monitor.receive(STREAM, Duration::from_millis(at), event)
}

fn event(receipt: u64, kind: ProcessEventKind, instance: ProcessInstance) -> RuntimeEvent {
    RuntimeEvent::Process(ProcessEvent {
        receipt: ReceiptId(receipt),
        kind,
        instance,
        observed_at: Some(SystemTime::UNIX_EPOCH),
    })
}

fn begin(monitor: &mut BaselineReconciler) {
    assert_eq!(receive(monitor, 1, &READY), Ok(Outcome::Applied));
    assert_eq!(
        monitor.begin_baseline(STREAM, Duration::from_millis(2)),
        Ok(Outcome::Applied)
    );
}

fn finish(
    monitor: &mut BaselineReconciler,
    instances: &[ProcessInstance],
) -> Result<Outcome, ReconcileError> {
    monitor.finish_baseline(STREAM, Duration::from_millis(20), instances)
}

fn assert_loss(monitor: &BaselineReconciler, loss: Loss) {
    assert_eq!(
        monitor.health(),
        Health::Degraded(Degradation::Interrupted(loss))
    );
    assert_eq!(monitor.first_loss(), Some(loss));
}

#[test]
fn listener_readiness_and_a_completed_baseline_precede_healthy() {
    let mut monitor = monitor(4, 4);
    assert_eq!(
        finish(&mut monitor, &[]),
        Err(ReconcileError::BaselineNotStarted)
    );
    assert_eq!(
        monitor.begin_baseline(STREAM, Duration::ZERO),
        Err(ReconcileError::SourceNotReady)
    );
    assert_eq!(
        receive(&mut monitor, 1, &RuntimeEvent::BaselineComplete),
        Err(ReconcileError::BaselineMethodRequired)
    );
    begin(&mut monitor);
    assert_eq!(monitor.health(), Health::Starting);
    assert_eq!(
        monitor.begin_baseline(STREAM, Duration::from_millis(3)),
        Err(ReconcileError::BaselineAlreadyStarted)
    );
    assert_eq!(finish(&mut monitor, &[]), Ok(Outcome::Applied));
    assert_eq!(monitor.health(), Health::Healthy);
    assert_eq!(monitor.pending_len(), 0);
}

#[test]
fn events_before_during_and_after_baseline_preserve_evidence() {
    let mut monitor = monitor(8, 8);
    receive(&mut monitor, 1, &READY).unwrap();
    let before = instance(1, 100);
    receive(
        &mut monitor,
        2,
        &event(1, ProcessEventKind::Started, before.clone()),
    )
    .unwrap();
    monitor
        .begin_baseline(STREAM, Duration::from_millis(3))
        .unwrap();
    receive(
        &mut monitor,
        4,
        &event(2, ProcessEventKind::Stopped, before),
    )
    .unwrap();
    let during = instance(2, 101);
    receive(
        &mut monitor,
        5,
        &event(3, ProcessEventKind::Started, during.clone()),
    )
    .unwrap();
    assert_eq!(monitor.pending_len(), 3);
    finish(&mut monitor, &[instance(1, 100), during]).unwrap();
    receive(
        &mut monitor,
        21,
        &event(4, ProcessEventKind::Started, instance(3, 102)),
    )
    .unwrap();
    let records = monitor.records();
    assert_eq!(records.len(), 3);
    assert!(records[0].at_baseline && records[0].start_observed && records[0].stop_observed);
    assert!(records[1].at_baseline && records[1].start_observed && !records[1].stop_observed);
    assert!(!records[2].at_baseline && records[2].start_observed && !records[2].stop_observed);
}

#[test]
fn fast_exit_missing_from_snapshot_is_retained_even_when_stop_arrives_first() {
    let mut monitor = monitor(4, 4);
    begin(&mut monitor);
    let stopped = event(2, ProcessEventKind::Stopped, instance(5, 100));
    receive(&mut monitor, 5, &stopped).unwrap();
    let mut started = event(1, ProcessEventKind::Started, instance(5, 100));
    if let RuntimeEvent::Process(event) = &mut started {
        event.observed_at = None;
    }
    receive(&mut monitor, 4, &started).unwrap();
    assert_eq!(
        receive(&mut monitor, 6, &stopped),
        Ok(Outcome::DuplicateReceipt)
    );
    assert_eq!(monitor.pending_len(), 2);
    finish(&mut monitor, &[]).unwrap();
    let record = &monitor.records()[0];
    assert!(record.start_observed && record.stop_observed && !record.at_baseline);
    assert_eq!(monitor.records().len(), 1);
}

#[test]
fn pid_reuse_and_incomplete_identity_never_merge_by_pid() {
    let mut monitor = monitor(8, 8);
    begin(&mut monitor);
    let old = instance(7, 100);
    let reused = instance(7, 101);
    let incomplete = ProcessInstance {
        start_time: None,
        ..old.clone()
    };
    let no_boot = ProcessInstance {
        boot_id: None,
        ..old.clone()
    };
    receive(
        &mut monitor,
        5,
        &event(1, ProcessEventKind::Stopped, old.clone()),
    )
    .unwrap();
    receive(
        &mut monitor,
        6,
        &event(2, ProcessEventKind::Started, reused.clone()),
    )
    .unwrap();
    receive(
        &mut monitor,
        7,
        &event(3, ProcessEventKind::Stopped, incomplete.clone()),
    )
    .unwrap();
    receive(
        &mut monitor,
        8,
        &event(4, ProcessEventKind::Stopped, no_boot),
    )
    .unwrap();
    finish(&mut monitor, &[old, reused, incomplete]).unwrap();
    assert_eq!(monitor.records().len(), 5);
    assert!(monitor.records()[0].stop_observed);
    assert!(!monitor.records()[1].stop_observed);
    assert!(monitor.records()[1].start_observed);
    assert!(!monitor.records()[2].stop_observed);
}

#[test]
fn a_full_buffer_records_loss_before_a_baseline_can_become_healthy() {
    let mut monitor = monitor(1, 4);
    begin(&mut monitor);
    let first = event(1, ProcessEventKind::Started, instance(1, 100));
    receive(&mut monitor, 5, &first).unwrap();
    assert_eq!(
        receive(&mut monitor, 6, &first),
        Ok(Outcome::DuplicateReceipt)
    );
    assert_eq!(
        receive(
            &mut monitor,
            7,
            &event(2, ProcessEventKind::Stopped, instance(1, 100))
        ),
        Err(ReconcileError::QueueOverflow)
    );
    assert_eq!(monitor.pending_len(), 1);
    finish(&mut monitor, &[]).unwrap();
    assert_eq!(monitor.records().len(), 1);
    assert!(!monitor.records()[0].stop_observed);
    assert_loss(&monitor, Loss::QueueOverflow);
}

#[test]
fn snapshot_and_buffer_capacity_failure_is_atomic_and_keeps_earlier_gap() {
    let mut monitor = monitor(4, 1);
    begin(&mut monitor);
    receive(
        &mut monitor,
        5,
        &event(1, ProcessEventKind::Started, instance(2, 100)),
    )
    .unwrap();
    assert_eq!(
        finish(&mut monitor, &[instance(1, 100)]),
        Err(ReconcileError::InstanceLimit)
    );
    assert_eq!(monitor.records().len(), 0);
    assert_eq!(monitor.pending_len(), 1);
    assert_loss(&monitor, Loss::ReconciliationLimit);
    finish(&mut monitor, &[]).unwrap();
    assert_loss(&monitor, Loss::ReconciliationLimit);
    assert_eq!(monitor.records().len(), 1);
    assert_eq!(
        receive(
            &mut monitor,
            21,
            &event(2, ProcessEventKind::Started, instance(3, 100))
        ),
        Err(ReconcileError::InstanceLimit)
    );
    assert_eq!(monitor.records().len(), 1);
}

#[test]
fn large_baselines_and_boot_tokens_are_rejected_before_retention() {
    let mut large_baseline = monitor(2, 1);
    begin(&mut large_baseline);
    assert_eq!(
        finish(&mut large_baseline, &[instance(1, 100), instance(2, 100)]),
        Err(ReconcileError::InstanceLimit)
    );
    assert!(large_baseline.records().is_empty());

    let mut monitor = monitor(2, 2);
    begin(&mut monitor);
    let huge = ProcessInstance {
        boot_id: Some("x".repeat(33)),
        ..instance(1, 100)
    };
    assert_eq!(
        receive(
            &mut monitor,
            5,
            &event(1, ProcessEventKind::Started, huge.clone())
        ),
        Err(ReconcileError::MetadataLimit)
    );
    assert_eq!(monitor.pending_len(), 0);
    assert_eq!(
        finish(&mut monitor, &[huge]),
        Err(ReconcileError::MetadataLimit)
    );
    assert!(monitor.records().is_empty());
    assert_loss(&monitor, Loss::InvalidMetadata);
}

#[test]
fn wrong_stream_cannot_change_baseline_records_or_deadline() {
    let mut monitor = monitor(4, 4);
    let other = StreamId {
        generation: SourceGeneration(2),
        ..STREAM
    };
    assert_eq!(
        monitor.receive(other, Duration::MAX, &READY),
        Ok(Outcome::OtherStream)
    );
    assert_eq!(
        monitor.begin_baseline(other, Duration::MAX),
        Ok(Outcome::OtherStream)
    );
    assert_eq!(
        monitor.finish_baseline(other, Duration::MAX, &[instance(1, 100)]),
        Ok(Outcome::OtherStream)
    );
    assert_eq!(
        monitor.report_baseline_failure(other, Duration::MAX),
        Outcome::OtherStream
    );
    assert_eq!(monitor.health(), Health::Starting);
    begin(&mut monitor);
    finish(&mut monitor, &[]).unwrap();
    assert!(monitor.records().is_empty());
    assert_eq!(monitor.health(), Health::Healthy);
}

#[test]
fn cancellation_deadline_and_unavailability_release_buffer_and_cannot_reopen() {
    for mode in 0..3 {
        let mut monitor = monitor(4, 4);
        begin(&mut monitor);
        receive(
            &mut monitor,
            5,
            &event(1, ProcessEventKind::Started, instance(1, 100)),
        )
        .unwrap();
        match mode {
            0 => monitor.cancel(),
            1 => monitor.advance(Duration::from_secs(1)),
            _ => {
                receive(
                    &mut monitor,
                    6,
                    &RuntimeEvent::SourceUnavailable(SourceError::Disconnected),
                )
                .unwrap();
            }
        }
        assert_eq!(monitor.pending_len(), 0);
        assert_eq!(finish(&mut monitor, &[]), Ok(Outcome::Inactive));
        assert_eq!(receive(&mut monitor, 7, &READY), Ok(Outcome::Inactive));
        assert_eq!(
            monitor.health(),
            match mode {
                0 => Health::Stopped(StopReason::Cancelled),
                1 => Health::Stopped(StopReason::Deadline),
                _ => Health::Unavailable(SourceError::Disconnected),
            }
        );
    }
}

#[test]
fn unknown_loss_reporting_and_baseline_failure_preserve_uncertainty() {
    let mut unknown = monitor(4, 4);
    receive(
        &mut unknown,
        1,
        &RuntimeEvent::SourceReady(LossReporting::Unknown),
    )
    .unwrap();
    unknown
        .begin_baseline(STREAM, Duration::from_millis(2))
        .unwrap();
    finish(&mut unknown, &[]).unwrap();
    assert_eq!(
        unknown.health(),
        Health::Degraded(Degradation::ContinuityUnknown)
    );

    let mut failed = monitor(4, 4);
    begin(&mut failed);
    assert_eq!(
        failed.report_baseline_failure(STREAM, Duration::from_millis(4)),
        Outcome::Applied
    );
    finish(&mut failed, &[]).unwrap();
    assert_loss(&failed, Loss::BaselineFailed);
}

#[test]
fn baseline_timestamps_cannot_precede_readiness_or_capture_start() {
    let mut monitor = monitor(4, 4);
    receive(&mut monitor, 5, &READY).unwrap();
    assert_eq!(
        monitor.begin_baseline(STREAM, Duration::from_millis(4)),
        Err(ReconcileError::InvalidTime)
    );
    monitor
        .begin_baseline(STREAM, Duration::from_millis(6))
        .unwrap();
    assert_eq!(
        monitor.finish_baseline(STREAM, Duration::from_millis(5), &[]),
        Err(ReconcileError::InvalidTime)
    );
    assert_eq!(monitor.health(), Health::Starting);
    finish(&mut monitor, &[]).unwrap();
    assert_eq!(monitor.health(), Health::Healthy);
}
