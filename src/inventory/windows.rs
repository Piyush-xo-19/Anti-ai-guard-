//! Local Windows filesystem adapter. Open each ancestor without following
//! reparse points and retain non-write/non-delete-sharing handles through reads.
use super::*;

#[cfg(windows)]
pub(super) fn valid_root(path: &Path) -> bool {
    use std::path::{Component, Prefix};
    let components: Vec<_> = path.components().collect();
    path.is_absolute() && components.len() >= 3 && components.len() <= 32
        && matches!(components[0], Component::Prefix(p) if matches!(p.kind(), Prefix::Disk(_)))
        && matches!(components[1], Component::RootDir)
        && components[2..].iter().all(|c| matches!(c, Component::Normal(value) if !value.to_string_lossy().contains(':') && !value.to_string_lossy().ends_with(['.', ' '])))
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
mod native {
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
        let mut options = OpenOptions::new();
        options
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS);
        if content {
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
