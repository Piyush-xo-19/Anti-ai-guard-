//! One consented, bounded diagnostic invocation. No persistence or verification.
use crate::{inventory, model::*, policy, processes, ArgumentError, InventoryOptions, ScanOptions};
use std::ffi::OsStr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Clone, Debug, Default)]
pub struct Metrics {
    pub inventory_probes: usize,
    pub metadata_bytes: usize,
    pub process_entries: usize,
    pub process_queries: usize,
    pub collection_elapsed_ms: u64,
}
#[derive(Clone, Debug)]
pub struct LocationLink {
    pub process_observation: usize,
    pub installation_observation: usize,
}
#[derive(Clone, Debug)]
pub struct ScanResult {
    pub diagnostic: policy::DiagnosticReport,
    pub metrics: Metrics,
    pub location_links: Vec<LocationLink>,
}

fn combine(a: ScanExecution, b: ScanExecution) -> ScanExecution {
    if a == ScanExecution::Cancelled || b == ScanExecution::Cancelled {
        ScanExecution::Cancelled
    } else if a == ScanExecution::Failed || b == ScanExecution::Failed {
        ScanExecution::Failed
    } else if a == ScanExecution::Partial || b == ScanExecution::Partial {
        ScanExecution::Partial
    } else {
        ScanExecution::Finished
    }
}

pub fn run(
    options: ScanOptions,
    path: Option<&OsStr>,
    cancelled: &AtomicBool,
) -> Result<ScanResult, ArgumentError> {
    if !options.consent {
        return Err(ArgumentError::Unsupported);
    }
    // Validate all filesystem scope before even probing platform/process metadata.
    let request = if !options.roots.is_empty() || options.include_path {
        Some(crate::inventory_request(
            InventoryOptions {
                roots: options.roots,
                include_path: options.include_path,
            },
            path,
        )?)
    } else {
        None
    };
    if request.is_none() && !options.processes {
        return Err(ArgumentError::Unsupported);
    }
    let began = std::time::Instant::now();
    let observed_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64;
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let mut snapshot = Snapshot {
        scan_id: format!(
            "scan-{observed_at_ms}-{}",
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ),
        observed_at_ms,
        platform: if cancelled.load(Ordering::Relaxed) {
            Platform {
                os: OperatingSystem::Unknown,
                build: None,
                architecture: Architecture::Unknown,
                inspection_mode: InspectionMode::Unknown,
            }
        } else {
            crate::platform::current()
        },
        scopes: vec![],
        requested: vec![],
        execution: ScanExecution::Finished,
        coverage: vec![],
        observations: vec![],
    };
    let mut metrics = Metrics::default();
    let mut location_links = vec![];
    let inventory_result = request
        .as_ref()
        .map(|request| inventory::collect(request, cancelled));
    if let Some(result) = &inventory_result {
        snapshot.scopes = result.scopes.clone();
        snapshot.coverage = result.coverage.clone();
        snapshot.requested = result.coverage.iter().map(|c| c.target).collect();
        snapshot.observations = result
            .findings
            .iter()
            .map(|f| f.observation.clone())
            .collect();
        snapshot.execution = combine(snapshot.execution, result.execution);
        metrics.inventory_probes = result.probes;
        metrics.metadata_bytes = result.bytes_read;
    }
    if options.processes {
        let scope = ScopeId(snapshot.scopes.len() as u32 + 1);
        let process_request =
            processes::ProcessRequest::new(true, scope, processes::Limits::default())
                .map_err(|_| ArgumentError::Unsupported)?;
        let link = request
            .as_ref()
            .zip(inventory_result.as_ref())
            .map(|(request, result)| processes::InventoryLink { request, result });
        let result = processes::collect(&process_request, cancelled, link);
        let offset = snapshot.observations.len();
        for (index, finding) in result.findings.iter().enumerate() {
            if let Some(installation) = finding.installation_indicator {
                location_links.push(LocationLink {
                    process_observation: offset + index,
                    installation_observation: installation,
                });
            }
        }
        snapshot.scopes.push(result.scope);
        snapshot.requested.push(result.coverage.target);
        snapshot.coverage.push(result.coverage);
        snapshot
            .observations
            .extend(result.findings.into_iter().map(|f| f.observation));
        snapshot.execution = combine(snapshot.execution, result.execution);
        metrics.process_entries = result.enumerated;
        metrics.process_queries = result.queried;
    }
    metrics.collection_elapsed_ms = began.elapsed().as_millis().min(u64::MAX as u128) as u64;
    Ok(ScanResult {
        diagnostic: policy::evaluate(snapshot),
        metrics,
        location_links,
    })
}
