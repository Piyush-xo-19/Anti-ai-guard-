use super::*;

#[cfg(not(windows))]
#[derive(Default)]
pub(super) struct NativeProvider;
#[cfg(not(windows))]
impl Provider for NativeProvider {
    fn capture(&mut self) -> Result<Context, GapReason> {
        Err(GapReason::Unsupported)
    }
    fn next(&mut self) -> Result<Option<Row>, GapReason> {
        Err(GapReason::Unsupported)
    }
    fn session(&mut self, _: u32) -> Result<u32, GapReason> {
        Err(GapReason::Unsupported)
    }
    fn metadata(&mut self, _: u32) -> Result<Metadata, GapReason> {
        Err(GapReason::Unsupported)
    }
}

#[cfg(windows)]
pub(super) use native::NativeProvider;

#[cfg(windows)]
mod native {
    use super::*;
    use std::ffi::c_void;
    use std::mem::{size_of, zeroed};
    use std::os::windows::ffi::OsStringExt;
    type Handle = *mut c_void;
    #[repr(C)]
    struct Entry {
        size: u32,
        usage: u32,
        pid: u32,
        heap: usize,
        module: u32,
        threads: u32,
        parent: u32,
        priority: i32,
        flags: u32,
        exe: [u16; 260],
    }
    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    impl FileTime {
        fn value(self) -> u64 {
            ((self.high as u64) << 32) | self.low as u64
        }
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> Handle;
        fn Process32FirstW(snapshot: Handle, entry: *mut Entry) -> i32;
        fn Process32NextW(snapshot: Handle, entry: *mut Entry) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> Handle;
        fn ProcessIdToSessionId(pid: u32, session: *mut u32) -> i32;
        fn GetSystemTimeAsFileTime(time: *mut FileTime);
        fn GetProcessTimes(
            process: Handle,
            creation: *mut FileTime,
            exit: *mut FileTime,
            kernel: *mut FileTime,
            user: *mut FileTime,
        ) -> i32;
        fn QueryFullProcessImageNameW(
            process: Handle,
            flags: u32,
            image: *mut u16,
            size: *mut u32,
        ) -> i32;
        fn WaitForSingleObject(handle: Handle, millis: u32) -> u32;
    }
    struct Owned(Handle);
    impl Drop for Owned {
        fn drop(&mut self) {
            // SAFETY: this owns a valid, non-inherited handle.
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    fn error() -> GapReason {
        match std::io::Error::last_os_error().raw_os_error() {
            Some(5) => GapReason::AccessDenied,
            Some(87 | 1168) => GapReason::ProcessExited,
            _ => GapReason::CollectorFailed,
        }
    }
    fn session(pid: u32) -> Result<u32, GapReason> {
        let mut id = 0;
        // SAFETY: id points to writable storage for the documented DWORD output.
        if unsafe { ProcessIdToSessionId(pid, &mut id) } == 0 {
            Err(error())
        } else {
            Ok(id)
        }
    }
    #[derive(Default)]
    pub(crate) struct NativeProvider {
        snapshot: Option<Owned>,
        first: bool,
        session: Option<u32>,
    }
    impl Provider for NativeProvider {
        fn capture(&mut self) -> Result<Context, GapReason> {
            let current = session(std::process::id())?;
            let mut checkpoint = FileTime::default();
            // SAFETY: valid output pointer; process-only snapshot flags, no heaps/modules.
            let handle = unsafe {
                GetSystemTimeAsFileTime(&mut checkpoint);
                CreateToolhelp32Snapshot(2, 0)
            };
            if handle as isize == -1 {
                return Err(error());
            }
            self.snapshot = Some(Owned(handle));
            self.first = true;
            self.session = Some(current);
            Ok(Context {
                checkpoint: checkpoint.value(),
                session: current,
            })
        }
        fn next(&mut self) -> Result<Option<Row>, GapReason> {
            let handle = self.snapshot.as_ref().ok_or(GapReason::CollectorFailed)?.0;
            // SAFETY: Entry is repr(C), all-zero bit patterns are valid integers;
            // size is initialized before the API can write the complete structure.
            let mut entry: Entry = unsafe { zeroed() };
            entry.size = size_of::<Entry>() as u32;
            let ok = unsafe {
                if self.first {
                    Process32FirstW(handle, &mut entry)
                } else {
                    Process32NextW(handle, &mut entry)
                }
            };
            self.first = false;
            if ok == 0 {
                if std::io::Error::last_os_error().raw_os_error() == Some(18) {
                    return Ok(None);
                }
                return Err(error());
            }
            let len = entry
                .exe
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(entry.exe.len());
            Ok(Some(Row {
                pid: entry.pid,
                basename: OsString::from_wide(&entry.exe[..len]),
            }))
        }
        fn session(&mut self, pid: u32) -> Result<u32, GapReason> {
            session(pid)
        }
        fn metadata(&mut self, pid: u32) -> Result<Metadata, GapReason> {
            // PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE. No VM-read,
            // terminate, suspend, injection or write rights are ever requested.
            let handle = unsafe { OpenProcess(0x1000 | 0x100000, 0, pid) };
            if handle.is_null() {
                return Err(error());
            }
            let handle = Owned(handle);
            if session(pid)? != self.session.ok_or(GapReason::CollectorFailed)? {
                return Err(GapReason::ProcessIdentityChanged);
            }
            let (mut creation, mut exit, mut kernel, mut user) = (
                FileTime::default(),
                FileTime::default(),
                FileTime::default(),
                FileTime::default(),
            );
            let mut gaps = vec![];
            let start = if unsafe {
                GetProcessTimes(handle.0, &mut creation, &mut exit, &mut kernel, &mut user)
            } != 0
            {
                Some(creation.value())
            } else {
                gaps.push(error());
                None
            };
            let mut image = vec![0u16; 32768];
            let mut len = image.len() as u32;
            let image =
                if unsafe { QueryFullProcessImageNameW(handle.0, 0, image.as_mut_ptr(), &mut len) }
                    != 0
                {
                    if len as usize > image.len() {
                        gaps.push(GapReason::InvalidMetadata);
                        None
                    } else {
                        Some(PathBuf::from(OsString::from_wide(&image[..len as usize])))
                    }
                } else {
                    gaps.push(error());
                    None
                };
            let state = match unsafe { WaitForSingleObject(handle.0, 0) } {
                0 => ProcessState::ExitedDuringEnrichment,
                258 => ProcessState::AliveAtSnapshot,
                _ => {
                    gaps.push(error());
                    ProcessState::AliveAtSnapshot
                }
            };
            Ok(Metadata {
                start,
                image,
                state,
                gaps,
            })
        }
    }
}
