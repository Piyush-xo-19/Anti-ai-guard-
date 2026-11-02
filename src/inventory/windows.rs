//! Local Windows filesystem adapter. Open each ancestor without following
//! reparse points and retain non-write/non-delete-sharing handles through reads.
use super::*;

#[cfg(windows)]
pub(super) fn valid_root(path: &Path) -> bool {
    use std::path::{Component, Prefix};
    // Reject Win32 aliases before any CreateFile call. Extensions do not make
    // DOS device names ordinary files; superscript 1/2/3 are reserved too.
    // https://learn.microsoft.com/windows/win32/fileio/naming-a-file
    fn ordinary_component(value: &std::ffi::OsStr) -> bool {
        let Some(text) = value.to_str() else {
            return false;
        };
        if text.ends_with(['.', ' '])
            || text
                .chars()
                .any(|c| matches!(c, ':' | '<' | '>' | '"' | '|' | '?' | '*' | '\0'..='\u{1f}'))
        {
            return false;
        }
        let stem = text
            .split('.')
            .next()
            .unwrap_or_default()
            .trim_end_matches(' ')
            .to_ascii_uppercase();
        if matches!(
            stem.as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
        ) {
            return false;
        }
        let device_digit = stem
            .strip_prefix("COM")
            .or_else(|| stem.strip_prefix("LPT"));
        !device_digit.is_some_and(|d| {
            matches!(
                d,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    }
    let components: Vec<_> = path.components().collect();
    path.is_absolute()
        && components.len() >= 3
        && components.len() <= 32
        && matches!(components[0], Component::Prefix(p) if matches!(p.kind(), Prefix::Disk(_)))
        && matches!(components[1], Component::RootDir)
        && components[2..]
            .iter()
            .all(|c| matches!(c, Component::Normal(value) if ordinary_component(value)))
}

#[cfg(not(windows))]
pub(super) fn valid_root(path: &Path) -> bool {
    path.is_absolute()
}

pub(super) struct WindowsFs;

#[cfg(not(windows))]
impl InventoryFs for WindowsFs {
    fn open_root(&mut self, _: &Path) -> Result<Box<dyn RootReader>, GapReason> {
        Err(GapReason::Unsupported)
    }
}

#[cfg(windows)]
pub(super) mod native {
    use super::*;
    use std::fs::{File, OpenOptions};
    use std::io::{self, Read};
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    // Stable Win32 constants and one narrow FFI import, using the normal
    // kernel32 import library available to both MSVC and GNU toolchains.
    // https://learn.microsoft.com/windows/win32/api/fileapi/nf-fileapi-createfilew
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x02000000;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x00200000;
    const FILE_READ_ATTRIBUTES: u32 = 0x80;
    const FILE_SHARE_READ: u32 = 1;
    const DRIVE_FIXED: u32 = 3;
    #[link(name = "kernel32")]
    extern "system" {
        fn GetDriveTypeW(root_path: *const u16) -> u32;
    }

    struct ProtectedRoot {
        path: PathBuf,
        _guards: Vec<File>,
    }

    fn io_gap(error: io::Error) -> GapReason {
        if error.raw_os_error() == Some(32) {
            return GapReason::SharingConflict;
        }
        match error.kind() {
            io::ErrorKind::PermissionDenied => GapReason::AccessDenied,
            _ => GapReason::CollectorFailed,
        }
    }

    // Returns a handle before inspecting metadata. OPEN_REPARSE_POINT applies
    // to the final component; holding all previously checked ancestors keeps
    // intermediate components from being replaced while this path is used.
    fn open(path: &Path, directory: bool, content: bool) -> Result<Option<File>, GapReason> {
        open_with_execute(path, directory, content, false)
    }
    fn open_with_execute(
        path: &Path,
        directory: bool,
        content: bool,
        execute: bool,
    ) -> Result<Option<File>, GapReason> {
        let mut options = OpenOptions::new();
        options
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS);
        if execute {
            // READ_DATA | EXECUTE | READ_ATTRIBUTES | SYNCHRONIZE; never writes.
            options.access_mode(0x001000a1);
        } else if content {
            options.read(true);
        } else {
            // FILE_READ_ATTRIBUTES alone bypasses normal sharing checks on
            // Windows. Also request FILE_READ_DATA/LIST_DIRECTORY (bit 1) so
            // retaining the handle actually excludes concurrent writers.
            // Native/shim file contents are still never read.
            options.access_mode(FILE_READ_ATTRIBUTES | 1);
        }
        let file = match options.open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(io_gap(error)),
        };
        let metadata = file.metadata().map_err(io_gap)?;
        // Reparse points include symlinks, junctions and cloud placeholders.
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(GapReason::ScopeBoundary);
        }
        if metadata.is_dir() != directory || (!directory && !metadata.is_file()) {
            return Err(GapReason::InvalidMetadata);
        }
        Ok(Some(file))
    }

    pub(crate) struct GuardedImage {
        pub file: File,
        _guards: Vec<File>,
    }
    pub(crate) fn open_image(path: &Path) -> Result<GuardedImage, GapReason> {
        if !valid_root(path) {
            return Err(GapReason::InvalidScope);
        }
        let components: Vec<_> = path.components().collect();
        let mut current = PathBuf::new();
        let mut guards = vec![];
        for (index, component) in components.iter().enumerate() {
            current.push(component);
            if !current.is_absolute() {
                continue;
            }
            if guards.is_empty() {
                let drive: Vec<u16> = current.as_os_str().encode_wide().chain(Some(0)).collect();
                // SAFETY: NUL-terminated drive path remains alive for the call.
                if unsafe { GetDriveTypeW(drive.as_ptr()) } != DRIVE_FIXED {
                    return Err(GapReason::Unsupported);
                }
            }
            let final_file = index + 1 == components.len();
            let file = open_with_execute(&current, !final_file, final_file, final_file)?
                .ok_or(GapReason::MissingRoot)?;
            if final_file {
                return Ok(GuardedImage {
                    file,
                    _guards: guards,
                });
            }
            guards.push(file);
        }
        Err(GapReason::InvalidScope)
    }
    impl InventoryFs for WindowsFs {
        fn open_root(&mut self, path: &Path) -> Result<Box<dyn RootReader>, GapReason> {
            if !valid_root(path) {
                return Err(GapReason::InvalidScope);
            }
            let mut current = PathBuf::new();
            let mut guards = vec![];
            for component in path.components() {
                current.push(component);
                if !current.is_absolute() {
                    continue;
                }
                if guards.is_empty() {
                    let drive: Vec<u16> =
                        current.as_os_str().encode_wide().chain(Some(0)).collect();
                    // SAFETY: drive is a NUL-terminated Windows path, alive for this call.
                    if unsafe { GetDriveTypeW(drive.as_ptr()) } != DRIVE_FIXED {
                        return Err(GapReason::Unsupported);
                    }
                }
                guards.push(open(&current, true, false)?.ok_or(GapReason::MissingRoot)?);
            }
            Ok(Box::new(ProtectedRoot {
                path: path.into(),
                _guards: guards,
            }))
        }
    }

    impl RootReader for ProtectedRoot {
        fn probe(
            &mut self,
            selector: &Selector,
            max_bytes: usize,
            bytes_read: &mut usize,
        ) -> Result<Probe, GapReason> {
            let relative = Path::new(selector.relative);
            let mut current = self.path.clone();
            let mut guards = vec![];
            let components: Vec<_> = relative.components().collect();
            for (index, component) in components.iter().enumerate() {
                current.push(component);
                let directory = index + 1 != components.len();
                let Some(file) = open(
                    &current,
                    directory,
                    !directory && selector.package.is_some(),
                )?
                else {
                    return Ok(Probe::Absent);
                };
                if directory {
                    guards.push(file);
                    continue;
                }
                if selector.package.is_none() {
                    return Ok(Probe::Indicator);
                }
                if file.metadata().map_err(io_gap)?.len() > max_bytes as u64 {
                    return Err(GapReason::BudgetExceeded);
                }
                let mut bytes = Vec::new();
                let read_result = file.take(max_bytes as u64).read_to_end(&mut bytes);
                // Failed reads may still consume bytes. Charge them before
                // propagating errors so later files cannot reuse that budget.
                *bytes_read += bytes.len();
                read_result.map_err(io_gap)?;
                return Ok(Probe::Manifest(bytes));
            }
            Err(GapReason::InvalidMetadata)
        }
    }
}
