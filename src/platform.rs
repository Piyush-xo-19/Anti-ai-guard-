//! Read-only OS/build and current-token metadata. Unknown values remain unknown.
use crate::model::*;

pub fn current() -> Platform {
    let architecture = if cfg!(target_arch = "x86_64") {
        Architecture::X64
    } else if cfg!(target_arch = "aarch64") {
        Architecture::Arm64
    } else if cfg!(target_arch = "x86") {
        Architecture::X86
    } else {
        Architecture::Unknown
    };
    let (os, build, inspection_mode) = native();
    Platform {
        os,
        build,
        architecture,
        inspection_mode,
    }
}
#[cfg(not(windows))]
fn native() -> (OperatingSystem, Option<String>, InspectionMode) {
    (OperatingSystem::Unknown, None, InspectionMode::Unknown)
}

#[cfg(windows)]
fn native() -> (OperatingSystem, Option<String>, InspectionMode) {
    use std::ffi::c_void;
    #[repr(C)]
    struct Version {
        size: u32,
        major: u32,
        minor: u32,
        build: u32,
        platform: u32,
        csd: [u16; 128],
        sp_major: u16,
        sp_minor: u16,
        suite: u16,
        product: u8,
        reserved: u8,
    }
    #[link(name = "advapi32")]
    extern "system" {
        fn OpenProcessToken(process: *mut c_void, access: u32, token: *mut *mut c_void) -> i32;
        fn GetTokenInformation(
            token: *mut c_void,
            class: u32,
            info: *mut c_void,
            len: u32,
            returned: *mut u32,
        ) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn CloseHandle(handle: *mut c_void) -> i32;
        fn GetModuleHandleW(name: *const u16) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
    }
    // SAFETY: repr(C) version structure has only integer fields; its size is
    // initialized for RtlGetVersion before the API writes it.
    let mut version: Version = unsafe { std::mem::zeroed() };
    version.size = std::mem::size_of::<Version>() as u32;
    let module_name: Vec<u16> = "ntdll.dll".encode_utf16().chain(Some(0)).collect();
    // SAFETY: only the already-loaded system module is consulted; no DLL search
    // or load occurs. The function is optional and the module is never unloaded.
    let module = unsafe { GetModuleHandleW(module_name.as_ptr()) };
    let address = if module.is_null() {
        std::ptr::null_mut()
    } else {
        unsafe { GetProcAddress(module, c"RtlGetVersion".as_ptr().cast()) }
    };
    let status = if address.is_null() {
        -1
    } else {
        // SAFETY: this exact export uses the documented NTSTATUS/version-info
        // system ABI; Version is the matching initialized repr(C) structure.
        let get_version: unsafe extern "system" fn(*mut Version) -> i32 =
            unsafe { std::mem::transmute(address) };
        unsafe { get_version(&mut version) }
    };
    let (os, build) = if status == 0 {
        (
            if version.product != 1 {
                OperatingSystem::WindowsServer
            } else if version.major == 10 && version.build >= 22000 {
                OperatingSystem::Windows11
            } else if version.major == 10 {
                OperatingSystem::Windows10
            } else {
                OperatingSystem::Unknown
            },
            Some(version.build.to_string()),
        )
    } else {
        (OperatingSystem::Unknown, None)
    };
    let mut token = std::ptr::null_mut();
    // SAFETY: -1 is the documented current-process pseudo handle; TOKEN_QUERY
    // is the only requested access. Output pointers refer to live local storage.
    let mode = if unsafe { OpenProcessToken(-1isize as *mut c_void, 8, &mut token) } != 0 {
        let mut elevation = 0u32;
        let mut returned = 0u32;
        let ok = unsafe {
            GetTokenInformation(
                token,
                20,
                (&mut elevation as *mut u32).cast(),
                4,
                &mut returned,
            )
        };
        unsafe {
            CloseHandle(token);
        }
        if ok == 0 || returned != 4 {
            InspectionMode::Unknown
        } else if elevation != 0 {
            InspectionMode::ElevatedDiagnostic
        } else {
            InspectionMode::StandardUser
        }
    } else {
        InspectionMode::Unknown
    };
    (os, build, mode)
}
