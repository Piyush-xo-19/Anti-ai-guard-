use anti_ai_guard::model::{Correlation, ProcessInstance};
use anti_ai_guard::runtime::{
    Degradation, Health, HealthReducer, Limits, Loss, LossReporting, Outcome, ProcessEvent,
    ProcessEventKind, ReceiptId, RunId, RuntimeEvent, SourceError, SourceGeneration, StopReason,
    StreamId,
};
use std::time::{Duration, SystemTime};

const STREAM: StreamId = StreamId {
    run: RunId(11),
    generation: SourceGeneration(3),
};
const READY: RuntimeEvent = RuntimeEvent::SourceReady(LossReporting::Observable);
const BASELINE: RuntimeEvent = RuntimeEvent::BaselineComplete;

fn reducer(max_receipts: usize) -> HealthReducer {
    HealthReducer::new(
        STREAM,
        Limits {
            deadline: Duration::from_secs(8),
            max_receipts: max_receipts.try_into().unwrap(),
        },
    )
}

fn apply(reducer: &mut HealthReducer, received_ms: u64, event: &RuntimeEvent) -> Outcome {
    reducer.apply(STREAM, Duration::from_millis(received_ms), event)
}

fn process(receipt: u64, kind: ProcessEventKind, start_time: Option<u64>) -> ProcessEvent {
    ProcessEvent {
        receipt: ReceiptId(receipt),
        kind,
        observed_at: start_time.map(|time| SystemTime::UNIX_EPOCH + Duration::from_secs(time)),
        instance: ProcessInstance {
            boot_id: Some("synthetic-boot".into()),
            pid: 10,
            start_time,
        },
    }
}

#[test]
fn health_requires_a_baseline_after_readiness() {
    let mut monitor = reducer(4);
    assert_eq!(monitor.health(), Health::Starting);
    assert_eq!(
        apply(&mut monitor, 1, &BASELINE),
        Outcome::BaselineBeforeReadiness
    );
    assert_eq!(apply(&mut monitor, 2, &READY), Outcome::Applied);
    assert_eq!(
        apply(&mut monitor, 1, &BASELINE),
        Outcome::BaselineBeforeReadiness
    );
    assert_eq!(monitor.health(), Health::Starting);
    assert_eq!(apply(&mut monitor, 3, &BASELINE), Outcome::Applied);
    assert_eq!(monitor.health(), Health::Healthy);
}

#[test]
fn baseline_and_later_readiness_signals_cannot_hide_unknown_loss_reporting() {
    let mut monitor = reducer(4);
    let unknown = RuntimeEvent::SourceReady(LossReporting::Unknown);
    assert_eq!(apply(&mut monitor, 1, &unknown), Outcome::Applied);
    assert_eq!(apply(&mut monitor, 2, &BASELINE), Outcome::Applied);
    assert_eq!(apply(&mut monitor, 3, &READY), Outcome::AlreadyReady);
    assert_eq!(
        monitor.health(),
        Health::Degraded(Degradation::ContinuityUnknown)
    );
    assert_eq!(monitor.first_loss(), None);
}

#[test]
fn reordered_start_stop_and_receipt_deduplication_leave_identity_unchanged() {
    let mut monitor = reducer(4);
    let start = process(10, ProcessEventKind::Started, Some(123));
    let mut stop = process(11, ProcessEventKind::Stopped, Some(123));
    stop.observed_at = start.observed_at.map(|time| time + Duration::from_secs(2));
    assert_eq!(
        start.instance.correlate(&stop.instance),
        Correlation::SameInstance
    );

    let stop = RuntimeEvent::Process(stop);
    let start = RuntimeEvent::Process(start);
    assert_eq!(apply(&mut monitor, 15, &stop), Outcome::Applied);
    assert_eq!(apply(&mut monitor, 14, &start), Outcome::Applied);
    assert_eq!(apply(&mut monitor, 16, &stop), Outcome::DuplicateReceipt);
    assert_eq!(monitor.health(), Health::Starting);
}

#[test]
fn missing_identity_and_pid_reuse_remain_distinct_from_delivery() {
    let mut monitor = reducer(4);
    let first = process(7, ProcessEventKind::Started, Some(123));
    let reused = process(8, ProcessEventKind::Started, Some(124));
    let incomplete = process(9, ProcessEventKind::Stopped, None);
    assert_eq!(first.instance.pid, reused.instance.pid);
    assert_eq!(
        first.instance.correlate(&reused.instance),
        Correlation::DifferentInstance
    );
    assert_eq!(
        reused.instance.correlate(&incomplete.instance),
        Correlation::Indeterminate
    );

    for event in [first, reused, incomplete] {
        assert_eq!(
            apply(&mut monitor, 1, &RuntimeEvent::Process(event)),
            Outcome::Applied
        );
    }
}

#[test]
fn foreign_runs_and_generations_do_not_change_health_or_consume_receipts() {
    let mut monitor = reducer(1);
    let other_generation = StreamId {
        generation: SourceGeneration(2),
        ..STREAM
    };
    let other_run = StreamId {
        run: RunId(12),
        ..STREAM
    };
    let event = RuntimeEvent::Process(process(1, ProcessEventKind::Started, None));
    for other in [other_generation, other_run] {
        assert_eq!(
            monitor.apply(other, Duration::from_millis(1), &event),
            Outcome::OtherStream
        );
        assert_eq!(
            monitor.apply(
                other,
                Duration::MAX,
                &RuntimeEvent::Loss(Loss::SourceReported)
            ),
            Outcome::OtherStream
        );
    }
    assert_eq!(monitor.health(), Health::Starting);
    assert_eq!(apply(&mut monitor, 2, &event), Outcome::Applied);
}

#[test]
fn receipt_limit_rejects_new_events_and_keeps_duplicate_tracking() {
    let mut monitor = reducer(1);
    let event = RuntimeEvent::Process(process(1, ProcessEventKind::Started, None));
    let next = RuntimeEvent::Process(process(2, ProcessEventKind::Stopped, None));
    assert_eq!(apply(&mut monitor, 1, &event), Outcome::Applied);
    assert_eq!(apply(&mut monitor, 2, &event), Outcome::DuplicateReceipt);
    assert_eq!(monitor.first_loss(), None);
    assert_eq!(apply(&mut monitor, 3, &next), Outcome::ReceiptLimit);
    assert_eq!(apply(&mut monitor, 4, &event), Outcome::DuplicateReceipt);
    assert_eq!(
        monitor.health(),
        Health::Degraded(Degradation::Interrupted(Loss::ReceiptLimit))
    );
}

#[test]
fn reported_loss_and_queue_overflow_survive_later_markers_and_stop() {
    for loss in [Loss::SourceReported, Loss::QueueOverflow] {
        let mut monitor = reducer(2);
        assert_eq!(apply(&mut monitor, 1, &READY), Outcome::Applied);
        assert_eq!(apply(&mut monitor, 2, &BASELINE), Outcome::Applied);
        assert_eq!(monitor.health(), Health::Healthy);

        assert_eq!(
            apply(&mut monitor, 3, &RuntimeEvent::Loss(loss)),
            Outcome::Applied
        );
        assert_eq!(apply(&mut monitor, 4, &BASELINE), Outcome::Applied);
        assert_eq!(
            monitor.health(),
            Health::Degraded(Degradation::Interrupted(loss))
        );

        monitor.cancel();
        assert_eq!(monitor.health(), Health::Stopped(StopReason::Cancelled));
        assert_eq!(monitor.first_loss(), Some(loss));
        assert_eq!(apply(&mut monitor, 5, &READY), Outcome::Inactive);
    }
}

#[test]
fn unavailable_source_requires_a_new_generation() {
    let mut monitor = reducer(2);
    let denied = RuntimeEvent::SourceUnavailable(SourceError::AccessDenied);
    assert_eq!(apply(&mut monitor, 1, &denied), Outcome::Applied);
    assert_eq!(apply(&mut monitor, 2, &READY), Outcome::Inactive);
    assert_eq!(apply(&mut monitor, 3, &BASELINE), Outcome::Inactive);
    assert_eq!(
        monitor.health(),
        Health::Unavailable(SourceError::AccessDenied)
    );
}

#[test]
fn deadline_discards_events_at_the_boundary_and_closes_silent_runs() {
    let mut at_deadline = reducer(2);
    assert_eq!(apply(&mut at_deadline, 8_000, &READY), Outcome::Inactive);
    assert_eq!(at_deadline.health(), Health::Stopped(StopReason::Deadline));

    let mut silent = reducer(2);
    silent.advance(Duration::from_millis(7_999));
    assert_eq!(silent.health(), Health::Starting);
    silent.advance(Duration::from_secs(8));
    silent.cancel();
    assert_eq!(silent.health(), Health::Stopped(StopReason::Deadline));
    assert_eq!(apply(&mut silent, 2, &READY), Outcome::Inactive);
}
