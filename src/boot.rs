//! Device-local boot correlation from a fixed self-process telemetry header.
//! Never requests variable payload, a foreign process, argv, or memory access.
use crate::model::GapReason;

#[repr(C)]
#[derive(Default)]
struct Header {
    size: u32,
    pid: u32,
    start_key: u64,
    create: u64,
    interrupt: u64,
    unbiased: u64,
    sequence: u64,
    session_create: u64,
    session: u32,
    boot: u32,
    checksum: u32,
    timestamp: u32,
    sid: u32,
    image: u32,
    package: u32,
    relative: u32,
    command: u32,
}

fn validate(
    header: &Header,
    status: i32,
    returned: u32,
    pid: u32,
    create: u64,
) -> Result<String, GapReason> {
    let size = std::mem::size_of::<Header>() as u32;
    // On the tested Windows builds, BUFFER_OVERFLOW returns a complete header
    // and no variable payload. Never retry with a larger buffer. Reject unknown
    // layouts, partially initialized headers, identity mismatches and offsets.
    let accepted_status = status == 0 || status as u32 == 0x80000005;
    if !accepted_status
        || returned < size
        || header.size != size
        || header.pid != pid
        || header.create != create
        || create == 0
        || [
            header.sid,
            header.image,
            header.package,
            header.relative,
            header.command,
        ]
        .iter()
        .any(|offset| *offset != 0)
    {
        return Err(GapReason::MissingBootIdentity);
    }
    Ok(format!("windows-local-telemetry-boot-v1:{}", header.boot))
}

#[cfg(windows)]
pub(crate) fn current() -> Result<String, GapReason> {
    use std::ffi::c_void;
    type Handle = *mut c_void;
    type Query = unsafe extern "system" fn(Handle, u32, *mut c_void, u32, *mut u32) -> i32;
    #[repr(C)]
    #[derive(Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleW(name: *const u16) -> Handle;
        fn GetProcAddress(module: Handle, name: *const u8) -> *mut c_void;
        fn GetCurrentProcess() -> Handle;
        fn GetProcessTimes(
            process: Handle,
            creation: *mut FileTime,
            exit: *mut FileTime,
            kernel: *mut FileTime,
            user: *mut FileTime,
        ) -> i32;
    }
    let name: Vec<u16> = "ntdll.dll\0".encode_utf16().collect();
    // SAFETY: kernel32 APIs receive valid NUL-terminated strings. ntdll is an
    // already loaded OS module; no user-controlled DLL search/loading occurs.
    let module = unsafe { GetModuleHandleW(name.as_ptr()) };
    if module.is_null() {
        return Err(GapReason::MissingBootIdentity);
    }
    let address = unsafe { GetProcAddress(module, c"NtQueryInformationProcess".as_ptr().cast()) };
    if address.is_null() {
        return Err(GapReason::MissingBootIdentity);
    }
    // SAFETY: the OS export has the documented NtQueryInformationProcess ABI.
    let query: Query = unsafe { std::mem::transmute(address) };
    let process = unsafe { GetCurrentProcess() };
    let (mut creation, mut exit, mut kernel, mut user) = (
        FileTime::default(),
        FileTime::default(),
        FileTime::default(),
        FileTime::default(),
    );
    if unsafe { GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user) } == 0 {
        return Err(GapReason::MissingBootIdentity);
    }
    let create = ((creation.high as u64) << 32) | creation.low as u64;
    let mut header = Header::default();
    let mut returned = 0;
    // SAFETY: fixed repr(C) output buffer, actual length, self pseudo-handle.
    // Exactly one bounded query; no variable payload or foreign argv is requested.
    let status = unsafe {
        query(
            process,
            64,
            (&mut header as *mut Header).cast(),
            std::mem::size_of::<Header>() as u32,
            &mut returned,
        )
    };
    validate(&header, status, returned, std::process::id(), create)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn header() -> Header {
        Header {
            size: 96,
            pid: 7,
            create: 123,
            boot: 4,
            ..Header::default()
        }
    }
    #[test]
    fn complete_header_is_required_even_for_the_known_overflow_status() {
        for status in [0, 0x80000005_u32 as i32] {
            assert!(validate(&header(), status, 96, 7, 123).is_ok());
            assert!(validate(&header(), status, 95, 7, 123).is_err());
            assert!(validate(
                &Header {
                    size: 112,
                    ..header()
                },
                status,
                112,
                7,
                123
            )
            .is_err());
            assert!(validate(&header(), status, 96, 8, 123).is_err());
            assert!(validate(&header(), status, 96, 7, 124).is_err());
        }
        assert!(validate(&header(), 0xc0000022_u32 as i32, 96, 7, 123).is_err());
        for field in 0..5 {
            let mut h = header();
            match field {
                0 => h.sid = 96,
                1 => h.image = 96,
                2 => h.package = 96,
                3 => h.relative = 96,
                _ => h.command = 96,
            }
            assert!(validate(&h, 0, 96, 7, 123).is_err());
        }
        let a = validate(&header(), 0, 96, 7, 123).unwrap();
        let b = validate(
            &Header {
                boot: 5,
                ..header()
            },
            0,
            96,
            7,
            123,
        )
        .unwrap();
        assert_ne!(a, b);
    }
    #[cfg(windows)]
    #[test]
    fn native_self_header_is_stable_without_payload_or_boot_token_output() {
        let a = current();
        let b = current();
        assert!(
            a.is_ok(),
            "fixed self header unavailable on this fixture configuration"
        );
        assert!(
            a == b,
            "boot identity changed between adjacent self queries"
        );
    }
}
