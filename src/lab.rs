//! Explicit owned-process labs only. This module is absent from production builds.
mod boot;
use crate::{artifact, catalog, model::*, processes};
use std::{
    ffi::c_void,
    fs::File,
    io::{Read, Write},
    os::windows::{
        io::{AsHandle, AsRawHandle, FromRawHandle},
        process::CommandExt,
    },
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
type Handle = *mut c_void;
#[repr(C)]
#[derive(Default)]
struct BasicLimits {
    process_time: i64,
    job_time: i64,
    flags: u32,
    min: usize,
    max: usize,
    active: u32,
    affinity: usize,
    priority: u32,
    scheduling: u32,
}
#[repr(C)]
#[derive(Default)]
struct ExtendedLimits {
    basic: BasicLimits,
    io: [u64; 6],
    process_memory: usize,
    job_memory: usize,
    peak_process: usize,
    peak_job: usize,
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreatePipe(read: *mut Handle, write: *mut Handle, security: *mut c_void, size: u32) -> i32;
    fn GetNamedPipeInfo(
        pipe: Handle,
        flags: *mut u32,
        output: *mut u32,
        input: *mut u32,
        instances: *mut u32,
    ) -> i32;
    fn CreateJobObjectW(security: *mut c_void, name: *const u16) -> Handle;
    fn SetInformationJobObject(job: Handle, class: u32, info: *const c_void, size: u32) -> i32;
    fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
    fn IsProcessInJob(process: Handle, job: Handle, result: *mut i32) -> i32;
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> Handle;
    fn CloseHandle(handle: Handle) -> i32;
    fn QueryInformationJobObject(
        job: Handle,
        class: u32,
        info: *mut c_void,
        size: u32,
        returned: *mut u32,
    ) -> i32;
    fn QueryFullProcessImageNameW(
        process: Handle,
        flags: u32,
        image: *mut u16,
        length: *mut u32,
    ) -> i32;
}
struct Job(Handle);
impl Job {
    fn active_node_count(&self) -> usize {
        #[repr(C)]
        struct Pids {
            assigned: u32,
            count: u32,
            ids: [usize; 64],
        }
        let mut list = Pids {
            assigned: 0,
            count: 0,
            ids: [0; 64],
        };
        assert_ne!(
            unsafe {
                QueryInformationJobObject(
                    self.0,
                    3,
                    (&mut list as *mut Pids).cast(),
                    std::mem::size_of::<Pids>() as u32,
                    std::ptr::null_mut(),
                )
            },
            0,
            "owned job process list unavailable"
        );
        assert!(
            list.assigned == list.count && list.count <= 64,
            "owned job process list truncated"
        );
        list.ids[..list.count as usize]
            .iter()
            .filter(|&&pid| {
                let Ok(pid) = u32::try_from(pid) else {
                    return false;
                };
                let handle = unsafe { OpenProcess(0x1000, 0, pid) };
                if handle.is_null() {
                    return false;
                }
                let mut inside = 0;
                if unsafe { IsProcessInJob(handle, self.0, &mut inside) } == 0 || inside == 0 {
                    unsafe {
                        CloseHandle(handle);
                    }
                    return false;
                }
                let mut image = [0u16; 32768];
                let mut length = image.len() as u32;
                let ok = unsafe {
                    QueryFullProcessImageNameW(handle, 0, image.as_mut_ptr(), &mut length)
                };
                unsafe {
                    CloseHandle(handle);
                }
                if ok == 0 {
                    return false;
                }
                let image = String::from_utf16_lossy(&image[..length as usize]);
                Path::new(&image).file_name().is_some_and(|name| {
                    name.eq_ignore_ascii_case(toolchain().0.file_name().unwrap())
                })
            })
            .count()
    }
    fn new() -> Self {
        let job = Self(unsafe { CreateJobObjectW(std::ptr::null_mut(), std::ptr::null()) });
        assert!(!job.0.is_null(), "owned lab job unavailable");
        let mut limits = ExtendedLimits::default();
        limits.basic.flags = 0x2000; // KILL_ON_JOB_CLOSE; no breakaway permission.
        assert_ne!(
            unsafe {
                SetInformationJobObject(
                    job.0,
                    9,
                    (&limits as *const ExtendedLimits).cast(),
                    std::mem::size_of::<ExtendedLimits>() as u32,
                )
            },
            0,
            "lab job setup failed"
        );
        job
    }
    fn contains(&self, pid: u32) -> bool {
        let process = unsafe { OpenProcess(0x1000, 0, pid) }; // limited read-only query
        if process.is_null() {
            return false;
        }
        let mut inside = 0;
        let ok = unsafe { IsProcessInJob(process, self.0, &mut inside) };
        unsafe {
            CloseHandle(process);
        }
        ok != 0 && inside != 0
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
struct Owned {
    job: Option<Job>,
    child: Child,
    drain: Option<std::thread::JoinHandle<Vec<u8>>>,
}
impl Drop for Owned {
    fn drop(&mut self) {
        // Only this test-owned job/tree is controlled. Scanner code cannot use it.
        self.job.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(drain) = self.drain.take() {
            let _ = drain.join();
        }
    }
}
fn consent() {
    assert_eq!(
        std::env::var("GUARD_LAUNCHER_LAB_CONSENT").as_deref(),
        Ok("yes")
    );
}
fn benign_case() -> bool {
    match std::env::var("GUARD_LAUNCHER_LAB_CASE").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("assistant") => false,
        Ok("benign-node") => true,
        _ => panic!("unknown launcher lab case"),
    }
}
fn row() -> serde_json::Value {
    let id = std::env::var("GUARD_LAUNCHER_LAB_TOOL").expect("explicit frozen tool required");
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../catalog/windows-launcher-lab.json")).unwrap();
    rows.into_iter()
        .find(|r| r["tool_id"] == id)
        .expect("unknown lab tool")
}
fn root(row: &serde_json::Value) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".tools/lab")
        .join(format!("npm-{}", row["tool_id"].as_str().unwrap()))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn verify_entry(row: &serde_json::Value) {
    let path = root(row).join(row["entrypoint"].as_str().unwrap());
    let mut image = crate::inventory::open_image(&path).expect("guarded launcher unavailable");
    let size = image.file.metadata().unwrap().len();
    assert!(size <= 512 * 1024 * 1024);
    let mut bytes = Vec::new();
    image.file.read_to_end(&mut bytes).unwrap();
    assert_eq!(hex(&artifact::lab_sha256(&bytes)), row["entry_sha256"]);
}
fn isolated(command: &mut Command) {
    let profile = Path::new(env!("CARGO_MANIFEST_DIR")).join(".tools/lab/launcher-profile");
    let system = std::env::var_os("SystemRoot").expect("Windows system root");
    command
        .env_clear()
        .env("SystemRoot", &system)
        .env(
            "ComSpec",
            Path::new(&system).join("System32").join("cmd.exe"),
        )
        .env(
            "PATH",
            std::env::join_paths([
                toolchain().0.parent().unwrap(),
                &Path::new(&system).join("System32"),
                &Path::new(&system)
                    .join("System32")
                    .join("WindowsPowerShell")
                    .join("v1.0"),
            ])
            .unwrap(),
        )
        .env("HOME", &profile)
        .env("USERPROFILE", &profile)
        .env("APPDATA", profile.join("AppData/Roaming"))
        .env("LOCALAPPDATA", profile.join("AppData/Local"))
        .env("TEMP", profile.join("tmp"))
        .env("TMP", profile.join("tmp"))
        .env("PATHEXT", ".COM;.EXE;.BAT;.CMD")
        .env("NPM_CONFIG_OFFLINE", "true")
        .env("NPM_CONFIG_USERCONFIG", profile.join("user.npmrc"))
        .env("NPM_CONFIG_GLOBALCONFIG", profile.join("global.npmrc"))
        .env("NPM_CONFIG_CACHE", profile.join("npm-cache"));
}
fn toolchain() -> (PathBuf, PathBuf) {
    let node = std::env::var_os("GUARD_LAUNCHER_LAB_NODE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files\nodejs\node.exe"));
    let npm = std::env::var_os("GUARD_LAUNCHER_LAB_NPM")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            node.parent()
                .unwrap()
                .join("node_modules/npm/bin/npm-cli.js")
        });
    assert!(
        node.is_absolute() && npm.is_absolute(),
        "trusted lab toolchain paths must be absolute"
    );
    (node, npm)
}
#[test]
#[ignore = "owned launcher child helper; requires explicit consent and parent handshake"]
fn launcher_child() {
    consent();
    let row = row();
    let mut go = [0; 1];
    std::io::stdin().read_exact(&mut go).unwrap();
    assert_eq!(go, [b'G']); // Parent assigns job before allowing any tool launch.
    verify_entry(&row);
    let mut cmd = Command::new(toolchain().0);
    isolated(&mut cmd);
    cmd.arg(toolchain().1).args([
        "exec",
        "--offline",
        "--yes=false",
        "--ignore-scripts",
        "--loglevel=silent",
    ]);
    if benign_case() {
        cmd.args(["--call", "node --version"]);
    } else {
        cmd.arg("--")
            .arg(row["command"].as_str().unwrap())
            .arg("--version");
    }
    let status = cmd
        .current_dir(root(&row))
        .stdin(Stdio::null())
        .stdout(Stdio::from(
            std::io::stderr().as_handle().try_clone_to_owned().unwrap(),
        ))
        .stderr(Stdio::from(
            std::io::stderr().as_handle().try_clone_to_owned().unwrap(),
        ))
        .creation_flags(0x08000000)
        .status()
        .unwrap();
    assert!(
        status.success(),
        "frozen npm launcher exited unsuccessfully"
    );
}
#[test]
#[ignore = "explicit real npm launcher lab with owned job and stdout backpressure"]
fn package_launcher_lab() {
    consent();
    let benign = benign_case();
    let row = row();
    verify_entry(&row);
    let catalog = catalog::get().unwrap();
    let pin = catalog
        .artifacts
        .iter()
        .find(|p| p.rule_id == row["rule_id"])
        .unwrap();
    let mut read = std::ptr::null_mut();
    let mut write = std::ptr::null_mut();
    assert_ne!(
        unsafe { CreatePipe(&mut read, &mut write, std::ptr::null_mut(), 4096) },
        0
    );
    let reader = unsafe { File::from_raw_handle(read) };
    let mut writer = unsafe { File::from_raw_handle(write) };
    let mut capacity = 0;
    assert_ne!(
        unsafe {
            GetNamedPipeInfo(
                write,
                std::ptr::null_mut(),
                &mut capacity,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        },
        0
    );
    assert!((1..=65536).contains(&capacity));
    // Empty private pipe, measured actual capacity, sole writer. Filling it
    // makes the real --version write wait while its live image is inspected.
    writer.write_all(&vec![0; capacity as usize]).unwrap();
    let job = Job::new();
    let mut command = Command::new(std::env::current_exe().unwrap());
    let (node, npm) = toolchain();
    command
        .env("GUARD_LAUNCHER_LAB_NODE", node)
        .env("GUARD_LAUNCHER_LAB_NPM", npm);
    command
        .args(["--ignored", "--exact", "lab::launcher_child", "--quiet"])
        .env("GUARD_LAUNCHER_LAB_CONSENT", "yes")
        .env("GUARD_LAUNCHER_LAB_TOOL", row["tool_id"].as_str().unwrap())
        .env(
            "GUARD_LAUNCHER_LAB_CASE",
            if benign { "benign-node" } else { "assistant" },
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::from(writer.try_clone().unwrap()))
        .creation_flags(0x08000000);
    let child = command.spawn().unwrap();
    drop(command);
    drop(writer);
    let mut owned = Owned {
        job: Some(job),
        child,
        drain: None,
    };
    assert_ne!(
        unsafe {
            AssignProcessToJobObject(owned.job.as_ref().unwrap().0, owned.child.as_raw_handle())
        },
        0,
        "cannot contain owned helper"
    );
    owned.child.stdin.take().unwrap().write_all(b"G").unwrap();
    let until = Instant::now() + Duration::from_secs(15);
    let mut observed = false;
    while Instant::now() < until {
        if benign && owned.job.as_ref().unwrap().active_node_count() < 2 {
            std::thread::sleep(Duration::from_millis(100));
            continue;
        }
        let request = processes::ProcessRequest::new(
            true,
            ScopeId(1),
            processes::Limits {
                max_elapsed: Duration::from_secs(10),
                ..Default::default()
            },
        )
        .unwrap();
        let result = processes::collect(&request, &AtomicBool::new(false), None);
        if benign {
            assert_eq!(
                result.execution,
                ScanExecution::Finished,
                "negative requires completed collection"
            );
            assert_eq!(
                result.coverage.state,
                CoverageState::Inspected,
                "negative requires inspected coverage"
            );
            assert!(
                result.coverage.gaps.is_empty(),
                "negative cannot pass with coverage gaps"
            );
            assert!(
                owned.job.as_ref().unwrap().active_node_count() >= 2,
                "Node/npm exited during negative snapshot"
            );
            assert!(result.findings.iter().all(|f| !matches!(&f.observation.kind,ObservationKind::Process{instance,..} if owned.job.as_ref().unwrap().contains(instance.pid))),"benign Node/npm classified as an assistant");
            observed = true;
            break;
        }
        for finding in result.findings {
            let ObservationKind::Process { instance, .. } = &finding.observation.kind else {
                continue;
            };
            if !owned.job.as_ref().unwrap().contains(instance.pid) {
                continue;
            }
            if finding
                .observation
                .identity
                .rule_match
                .as_ref()
                .is_some_and(|r| r.rule_id == pin.rule_id)
            {
                assert_eq!(
                    finding.observation.identity.class,
                    EvidenceClass::Corroborated
                );
                assert_eq!(
                    crate::policy::suggest_action(&finding.observation).action,
                    crate::policy::SuggestedAction::RequestClosureAndRescan
                );
                let diagnostic = crate::policy::evaluate(Snapshot {
                    scan_id: "scan-1-1".into(),
                    observed_at_ms: 0,
                    platform: crate::platform::current(),
                    scopes: vec![result.scope.clone()],
                    requested: vec![result.coverage.target],
                    execution: result.execution,
                    coverage: vec![result.coverage.clone()],
                    observations: vec![finding.observation.clone()],
                });
                let public = crate::report::project(&crate::scan::ScanResult {
                    diagnostic,
                    metrics: Default::default(),
                    location_links: vec![],
                });
                assert_eq!(public.value()["eligibility"], "NotIssued");
                assert_eq!(
                    public.value()["findings"][0]["artifact_rule_id"],
                    pin.rule_id
                );
                assert!(!public.json().contains(env!("CARGO_MANIFEST_DIR")));
                assert!(!public.json().contains(&crate::boot::current().unwrap()));
                observed = true;
            }
        }
        if observed {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    // Drain before asserting so even a failed observation releases real --version.
    owned.drain = Some(std::thread::spawn(move || {
        let mut bytes = Vec::new();
        reader.take(65536).read_to_end(&mut bytes).unwrap();
        bytes
    }));
    let until = Instant::now() + Duration::from_secs(15);
    let status = loop {
        if let Some(status) = owned.child.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < until,
            "owned launcher did not finish after draining stdout"
        );
        std::thread::sleep(Duration::from_millis(50));
    };
    drop(owned.job.take()); // Close any remaining owned writers before joining the drain.
    let bytes = owned.drain.take().unwrap().join().unwrap();
    if !status.success() {
        std::fs::write(
            Path::new(env!("CARGO_MANIFEST_DIR")).join(".tools/lab/launcher-failure.private.log"),
            &bytes,
        )
        .unwrap();
    }
    assert!(
        status.success(),
        "owned npm launcher failed; isolated private log retained"
    );
    let text = String::from_utf8_lossy(&bytes);
    if benign {
        let version = text.trim_start_matches('\0').trim();
        assert!(
            version.starts_with('v') && version.as_bytes().get(1).is_some_and(u8::is_ascii_digit),
            "Node version output missing"
        );
    } else {
        assert!(
            text.contains(&pin.tool_version),
            "expected pinned version output missing"
        );
    }
    assert!(
        observed,
        "no pinned image observed in the owned launcher job"
    );
    verify_entry(&row);
    if benign {
        println!("LAUNCHER_BENIGN result=PASS collection=Finished coverage=Inspected gaps=0 node_and_npm_alive=true owned_assistant_findings=0");
    } else {
        println!("LAUNCHER_LAB result=PASS tool={} version={} source=npm-exec native_image=Corroborated launcher_no_window_flag=true model_request=none", pin.tool_id, pin.tool_version);
    }
}

#[test]
fn frozen_launcher_entries_match_the_reviewed_tool_versions() {
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../catalog/windows-launcher-lab.json")).unwrap();
    assert_eq!(rows.len(), 4);
    let mut distinct = std::collections::HashSet::new();
    for row in rows {
        let pin = catalog::get()
            .unwrap()
            .artifacts
            .iter()
            .find(|p| p.rule_id == row["rule_id"])
            .unwrap();
        assert_eq!(row["tool_id"], pin.tool_id);
        assert!(distinct.insert(pin.tool_id.as_str()));
        assert_eq!(row["version"], pin.tool_version);
        let command = match pin.tool_id.as_str() {
            "codex-cli" => "codex",
            "claude-code" => "claude",
            "opencode-v1" | "opencode-v2" => "opencode",
            _ => panic!("unknown command family"),
        };
        assert_eq!(row["command"], command);
        let expected = match pin.tool_id.as_str() {
            "codex-cli" => "node_modules/@openai/codex/bin/codex.js",
            "claude-code" => "node_modules/@anthropic-ai/claude-code/bin/claude.exe",
            "opencode-v1" => "node_modules/opencode-ai/bin/opencode.exe",
            "opencode-v2" => "node_modules/@opencode/cli/bin/opencode.exe",
            _ => panic!("unknown frozen launcher"),
        };
        assert_eq!(row["entrypoint"], expected);
        let alternative = match pin.tool_id.as_str() {
            "opencode-v1" => Some("node_modules/opencode-windows-x64-baseline/bin/opencode.exe"),
            "opencode-v2" => {
                Some("node_modules/@opencode/cli-windows-x64-baseline/bin/opencode.exe")
            }
            _ => None,
        };
        assert_eq!(row["setup_alternative"]["file"].as_str(), alternative);
        if alternative.is_some() {
            assert_eq!(
                row["setup_alternative"]["sha256"].as_str().unwrap().len(),
                64
            );
        }
        if pin.tool_id != "codex-cli" {
            assert_eq!(row["entry_sha256"], pin.sha256);
        } else {
            assert_eq!(row["entry_sha256"].as_str().unwrap().len(), 64);
        }
    }
}
