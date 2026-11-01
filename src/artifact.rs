//! Native read-only SHA-256 and a positive/negative image-file binding check.
use crate::{catalog, inventory, model::*};
use std::{
    ffi::c_void,
    io::Read,
    os::windows::io::AsRawHandle,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
type Handle = *mut c_void;
type Query = unsafe extern "system" fn(Handle, u32, *mut c_void, u32, *mut u32) -> i32;
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
    fn GetProcAddress(module: Handle, name: *const u8) -> *mut c_void;
    fn GetCurrentProcess() -> Handle;
    fn WaitForSingleObject(handle: Handle, millis: u32) -> u32;
}
#[link(name = "bcrypt")]
unsafe extern "system" {
    fn BCryptOpenAlgorithmProvider(
        algorithm: *mut Handle,
        name: *const u16,
        implementation: *const u16,
        flags: u32,
    ) -> i32;
    fn BCryptCloseAlgorithmProvider(algorithm: Handle, flags: u32) -> i32;
    fn BCryptCreateHash(
        algorithm: Handle,
        hash: *mut Handle,
        object: *mut u8,
        object_len: u32,
        secret: *mut u8,
        secret_len: u32,
        flags: u32,
    ) -> i32;
    fn BCryptHashData(hash: Handle, input: *const u8, length: u32, flags: u32) -> i32;
    fn BCryptFinishHash(hash: Handle, output: *mut u8, length: u32, flags: u32) -> i32;
    fn BCryptDestroyHash(hash: Handle) -> i32;
}
struct Algorithm(Handle);
impl Drop for Algorithm {
    fn drop(&mut self) {
        unsafe {
            BCryptCloseAlgorithmProvider(self.0, 0);
        }
    }
}
struct Hash {
    handle: Handle,
    _algorithm: Algorithm,
}
impl Drop for Hash {
    fn drop(&mut self) {
        unsafe {
            BCryptDestroyHash(self.handle);
        }
    }
}
impl Hash {
    fn new() -> Result<Self, GapReason> {
        let name: Vec<u16> = "SHA256\0".encode_utf16().collect();
        let mut algorithm = std::ptr::null_mut();
        // SAFETY: valid output and constant NUL-terminated algorithm; no secret.
        if unsafe {
            BCryptOpenAlgorithmProvider(&mut algorithm, name.as_ptr(), std::ptr::null(), 0)
        } != 0
        {
            return Err(GapReason::CollectorFailed);
        }
        let algorithm = Algorithm(algorithm);
        let mut hash = std::ptr::null_mut();
        // CNG owns the hash-object storage (supported since Windows 7).
        if unsafe {
            BCryptCreateHash(
                algorithm.0,
                &mut hash,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                0,
                0,
            )
        } != 0
        {
            return Err(GapReason::CollectorFailed);
        }
        Ok(Self {
            handle: hash,
            _algorithm: algorithm,
        })
    }
    fn update(&mut self, bytes: &[u8]) -> Result<(), GapReason> {
        let len = u32::try_from(bytes.len()).map_err(|_| GapReason::BudgetExceeded)?;
        // SAFETY: immutable live buffer, checked length, owned hash handle.
        if unsafe { BCryptHashData(self.handle, bytes.as_ptr(), len, 0) } != 0 {
            return Err(GapReason::CollectorFailed);
        }
        Ok(())
    }
    fn finish(self) -> Result<[u8; 32], GapReason> {
        let mut digest = [0; 32];
        // SAFETY: SHA256 provider and exact digest-sized writable output.
        if unsafe { BCryptFinishHash(self.handle, digest.as_mut_ptr(), 32, 0) } != 0 {
            return Err(GapReason::CollectorFailed);
        }
        Ok(digest)
    }
}
fn query() -> Result<Query, GapReason> {
    let name: Vec<u16> = "ntdll.dll\0".encode_utf16().collect();
    // SAFETY: already loaded OS module, fixed export; no DLL search path.
    let module = unsafe { GetModuleHandleW(name.as_ptr()) };
    if module.is_null() {
        return Err(GapReason::Unsupported);
    }
    let address = unsafe { GetProcAddress(module, c"NtQueryInformationProcess".as_ptr().cast()) };
    if address.is_null() {
        return Err(GapReason::Unsupported);
    }
    Ok(unsafe { std::mem::transmute::<*mut c_void, Query>(address) })
}
fn mapped(query: Query, process: Handle, file: Handle) -> Result<bool, GapReason> {
    let mut file = file;
    let mut returned = 0;
    // Class 44 is a native compatibility capability, not a documented Win32
    // identity guarantee. It consumes the opened file handle as input. Only
    // exact success is positive; all unknown statuses reject strong identity.
    let status = unsafe {
        query(
            process,
            44,
            (&mut file as *mut Handle).cast(),
            std::mem::size_of::<Handle>() as u32,
            &mut returned,
        )
    };
    match status as u32 {
        0 => Ok(true),
        0xc0000001 => Ok(false),
        0xc0000022 => Err(GapReason::AccessDenied),
        _ => Err(GapReason::ArtifactIdentityUnavailable),
    }
}
pub(crate) fn inspect(
    process: Handle,
    path: &Path,
    cancelled: &AtomicBool,
    deadline: Instant,
    remaining: &mut u64,
) -> Result<Option<IdentityAssessment>, GapReason> {
    let stop = || {
        if cancelled.load(Ordering::Relaxed) {
            Err(GapReason::Cancelled)
        } else if Instant::now() >= deadline {
            Err(GapReason::BudgetExceeded)
        } else {
            Ok(())
        }
    };
    stop()?;
    let catalog = catalog::get()?;
    if !cfg!(target_arch = "x86_64") {
        return Ok(None);
    }
    let mut image = inventory::open_image(path)?;
    let bytes = image
        .file
        .metadata()
        .map_err(|_| GapReason::CollectorFailed)?
        .len();
    // Avoid reading any content when no reviewed pin has this exact size.
    if !catalog.artifacts.iter().any(|p| p.bytes == bytes) {
        return Ok(None);
    }
    if bytes > *remaining {
        return Err(GapReason::BudgetExceeded);
    }
    let query = query()?;
    let file = image.file.as_raw_handle();
    // A native class that succeeds for both target and this unrelated scanner
    // is not an identity test. Require the negative control as well as positive.
    if !mapped(query, process, file)? || mapped(query, unsafe { GetCurrentProcess() }, file)? {
        return Err(GapReason::ArtifactIdentityUnavailable);
    }
    let mut hash = Hash::new()?;
    let mut buffer = [0u8; 65536];
    let mut read = 0u64;
    loop {
        stop()?;
        let length = image
            .file
            .read(&mut buffer)
            .map_err(|_| GapReason::CollectorFailed)?;
        if length == 0 {
            break;
        }
        *remaining = remaining
            .checked_sub(length as u64)
            .ok_or(GapReason::BudgetExceeded)?;
        read = read
            .checked_add(length as u64)
            .ok_or(GapReason::BudgetExceeded)?;
        if read > bytes {
            return Err(GapReason::InvalidMetadata);
        }
        hash.update(&buffer[..length])?;
    }
    stop()?;
    if read != bytes
        || !mapped(query, process, file)?
        || unsafe { WaitForSingleObject(process, 0) } != 258
    {
        return Err(GapReason::ProcessIdentityChanged);
    }
    let digest = hash
        .finish()?
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    Ok(catalog
        .artifacts
        .iter()
        .find(|p| p.bytes == bytes && p.sha256 == digest)
        .map(|p| catalog::assessment(p, catalog)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_and_expired_deadline_precede_any_artifact_io() {
        let mut remaining = 1024;
        let path = Path::new("not-an-approved-or-existing-path");
        assert_eq!(
            inspect(
                std::ptr::null_mut(),
                path,
                &AtomicBool::new(true),
                Instant::now() + std::time::Duration::from_secs(1),
                &mut remaining
            )
            .unwrap_err(),
            GapReason::Cancelled
        );
        assert_eq!(
            inspect(
                std::ptr::null_mut(),
                path,
                &AtomicBool::new(false),
                Instant::now(),
                &mut remaining
            )
            .unwrap_err(),
            GapReason::BudgetExceeded
        );
        assert_eq!(remaining, 1024);
    }
    #[test]
    fn cng_sha256_matches_standard_vectors_and_chunking() {
        for (input, expected) in [
            (
                b"".as_slice(),
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                b"abc".as_slice(),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
        ] {
            let mut hash = Hash::new().unwrap();
            for byte in input {
                hash.update(&[*byte]).unwrap();
            }
            assert_eq!(
                hash.finish()
                    .unwrap()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>(),
                expected
            );
        }
    }
    #[test]
    fn native_binding_rejects_an_identical_file_copy() {
        use std::os::windows::fs::OpenOptionsExt;
        let current = std::env::current_exe().unwrap();
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/image-binding-fixture");
        std::fs::create_dir_all(&base).unwrap();
        let copy = base.join(format!("copy-{}.exe", std::process::id()));
        std::fs::copy(&current, &copy).unwrap();
        let open = |p: &Path| {
            std::fs::OpenOptions::new()
                .access_mode(0x001000a1)
                .share_mode(1)
                .open(p)
                .unwrap()
        };
        let original = open(&current);
        let duplicate = open(&copy);
        let q = query().unwrap();
        let process = unsafe { GetCurrentProcess() };
        assert!(mapped(q, process, original.as_raw_handle()).unwrap());
        assert!(!mapped(q, process, duplicate.as_raw_handle()).unwrap());
        drop(duplicate);
        std::fs::remove_file(copy).unwrap();
    }
}

#[cfg(windows)]
#[test]
#[ignore = "explicit opt-in pinned official Windows artifact lab; never a default fixture"]
fn pinned_codex_native_lab() {
    use std::os::windows::ffi::OsStrExt;
    assert_eq!(
        std::env::var("GUARD_PINNED_LAB_CONSENT").as_deref(),
        Ok("yes"),
        "explicit lab consent required"
    );
    let path = std::path::PathBuf::from(
        std::env::var_os("GUARD_PINNED_LAB_ARTIFACT").expect("pinned lab path required"),
    );
    let catalog = catalog::get().unwrap();
    let pin = &catalog.artifacts[0];
    let mut artifact = inventory::open_image(&path).expect("guarded lab artifact unavailable");
    assert_eq!(
        artifact.file.metadata().unwrap().len(),
        pin.bytes,
        "lab artifact size mismatch"
    );
    let mut hash = Hash::new().unwrap();
    let mut buffer = [0; 65536];
    loop {
        let n = artifact.file.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]).unwrap();
    }
    let digest = hash
        .finish()
        .unwrap()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert!(
        digest == pin.sha256,
        "lab artifact digest mismatch; refusing launch"
    );
    #[repr(C)]
    struct Startup {
        size: u32,
        reserved: *mut u16,
        desktop: *mut u16,
        title: *mut u16,
        x: u32,
        y: u32,
        xsize: u32,
        ysize: u32,
        xchars: u32,
        ychars: u32,
        fill: u32,
        flags: u32,
        show: u16,
        reserved2: u16,
        bytes: *mut u8,
        input: Handle,
        output: Handle,
        error: Handle,
    }
    #[repr(C)]
    struct Process {
        process: Handle,
        thread: Handle,
        pid: u32,
        tid: u32,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateProcessW(
            application: *const u16,
            command: *mut u16,
            process_attributes: *mut c_void,
            thread_attributes: *mut c_void,
            inherit: i32,
            flags: u32,
            environment: *mut c_void,
            cwd: *const u16,
            startup: *mut Startup,
            process: *mut Process,
        ) -> i32;
        fn ResumeThread(thread: Handle) -> u32;
        fn TerminateProcess(process: Handle, code: u32) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
    }
    struct OwnedLab(Process);
    impl Drop for OwnedLab {
        fn drop(&mut self) {
            unsafe {
                // Cleanup affects ONLY the exact process created by this opt-in test.
                ResumeThread(self.0.thread);
                if WaitForSingleObject(self.0.process, 10000) != 0 {
                    TerminateProcess(self.0.process, 1);
                    WaitForSingleObject(self.0.process, 5000);
                }
                CloseHandle(self.0.thread);
                CloseHandle(self.0.process);
            }
        }
    }
    let application: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut command: Vec<u16> = std::iter::once('"' as u16)
        .chain(path.as_os_str().encode_wide())
        .chain("\" --version\0".encode_utf16())
        .collect();
    let mut startup: Startup = unsafe { std::mem::zeroed() };
    startup.size = std::mem::size_of::<Startup>() as u32;
    let mut process: Process = unsafe { std::mem::zeroed() };
    // Start only the already pinned artifact, suspended for deterministic image
    // observation. It makes no paid/model request. Normal scanner never launches.
    assert_ne!(
        unsafe {
            CreateProcessW(
                application.as_ptr(),
                command.as_mut_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                0x08000004,
                std::ptr::null_mut(),
                std::ptr::null(),
                &mut startup,
                &mut process,
            )
        },
        0,
        "owned lab process creation failed"
    );
    let owned = OwnedLab(process);
    let request = crate::processes::ProcessRequest::new(
        true,
        ScopeId(1),
        crate::processes::Limits {
            max_elapsed: std::time::Duration::from_secs(10),
            ..Default::default()
        },
    )
    .unwrap();
    let result = crate::processes::collect(&request, &AtomicBool::new(false), None);
    let finding=result.findings.iter().find(|f|matches!(&f.observation.kind,ObservationKind::Process{instance,..} if instance.pid==owned.0.pid)).expect("owned pinned process was not observed");
    assert_eq!(
        finding.observation.identity.class,
        EvidenceClass::Corroborated
    );
    let action = crate::policy::suggest_action(&finding.observation);
    assert_eq!(
        action.action,
        crate::policy::SuggestedAction::RequestClosureAndRescan
    );
    assert!(catalog::public_match(&finding.observation.identity).is_some());
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
    assert_eq!(
        diagnostic.eligibility,
        crate::policy::Eligibility::NotIssued
    );
    let public = crate::report::project(&crate::scan::ScanResult {
        diagnostic,
        metrics: Default::default(),
        location_links: vec![],
    });
    assert_eq!(
        public.value()["findings"][0]["artifact_rule_id"],
        pin.rule_id
    );
    assert!(public.text().contains(&pin.rule_id));
    println!("PINNED_LAB_CASE result=PASS tool=codex-cli version=0.166.0 evidence=Corroborated action=RequestClosureAndRescan eligibility=NotIssued process_started_suspended=true");
    drop(owned);
    drop(artifact);
}
