//! Consented, bounded diagnostic preflight. No competition verification.
pub mod cancellation;
pub mod inventory;
pub mod model;
pub mod platform;
pub mod policy;
pub mod processes;
pub mod report;
pub mod scan;
use std::ffi::{OsStr, OsString};
use std::fmt;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const HELP:&str="Anti-AI Guard
Read-only competition preflight tooling (early development).

Usage: anti-ai-guard [OPTION]
       anti-ai-guard inventory --consent [--bin-root PATH] [--npm-prefix PATH] [--include-path] [--json]
       anti-ai-guard processes --consent [--json]
       anti-ai-guard scan --consent [--bin-root PATH] [--npm-prefix PATH] [--include-path] [--processes] [--json]

Options:
  -h, --help       Show this help
  -V, --version    Show the program version

Inventory checks fixed executable names and npm manifests in approved roots.
Processes checks current-session known native name indicators. Names alone require review.
Scan combines only the scopes explicitly selected; no default disk/PATH/process inspection.
--include-path opts into PATH directories; relative/empty/network roots are rejected.
No tool is executed, no command lines or memory are read, and no eligibility is issued.
Help/version do not inspect installations, processes, or files.
JSON and text derive from the same minimized, bounded diagnostic report.
";
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Version,
    Collect(ScanOptions),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}
#[derive(Debug, PartialEq, Eq)]
pub struct ScanOptions {
    pub roots: Vec<inventory::InventoryRoot>,
    pub include_path: bool,
    pub processes: bool,
    pub format: OutputFormat,
    pub(crate) consent: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub struct InventoryOptions {
    pub roots: Vec<inventory::InventoryRoot>,
    pub include_path: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub struct ArgumentError;
impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unsupported arguments; run anti-ai-guard --help for usage")
    }
}
impl std::error::Error for ArgumentError {}
/// Expand only explicitly opted-in PATH. No filesystem access occurs here.
pub fn inventory_request(
    options: InventoryOptions,
    path: Option<&OsStr>,
) -> Result<inventory::InventoryRequest, ArgumentError> {
    let mut roots = options.roots;
    if options.include_path {
        let path = path
            .filter(|p| !p.is_empty() && p.len() <= 32768)
            .ok_or(ArgumentError)?;
        roots.extend(
            std::env::split_paths(path).map(|path| inventory::InventoryRoot {
                path,
                kind: inventory::RootKind::WindowsNpmPrefix,
            }),
        );
    }
    inventory::InventoryRequest::new(true, roots, inventory::Limits::default())
        .map_err(|_| ArgumentError)
}
/// Bounded argument parsing; raw input never enters error messages.
pub fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Command, ArgumentError> {
    let bounded: Vec<_> = args.into_iter().take(132).collect();
    if bounded.len() > 131 {
        return Err(ArgumentError);
    }
    let mut args = bounded.into_iter();
    let Some(first) = args.next() else {
        return Ok(Command::Help);
    };
    if first == "inventory" || first == "processes" || first == "scan" {
        let rest: Vec<_> = args.collect();
        if rest.len() == 1 && rest[0] == "--help" {
            return Ok(Command::Help);
        }
        let mut args = rest.into_iter();
        let mut consent = false;
        let mut include_path = false;
        let mut processes = first == "processes";
        let mut format = OutputFormat::Text;
        let mut roots = vec![];
        while let Some(option) = args.next() {
            if option == "--consent" && !consent {
                consent = true;
            } else if option == "--json" && format == OutputFormat::Text {
                format = OutputFormat::Json;
            } else if option == "--processes" && first == "scan" && !processes {
                processes = true;
            } else if option == "--include-path" && first != "processes" && !include_path {
                include_path = true;
            } else if first != "processes" && (option == "--bin-root" || option == "--npm-prefix") {
                let path = args.next().filter(|p| !p.is_empty()).ok_or(ArgumentError)?;
                roots.push(inventory::InventoryRoot {
                    path: path.into(),
                    kind: if option == "--bin-root" {
                        inventory::RootKind::BinDirectory
                    } else {
                        inventory::RootKind::WindowsNpmPrefix
                    },
                });
                if roots.len() > 32 {
                    return Err(ArgumentError);
                }
            } else {
                return Err(ArgumentError);
            }
        }
        if !consent || (roots.is_empty() && !include_path && !processes) {
            return Err(ArgumentError);
        }
        return Ok(Command::Collect(ScanOptions {
            roots,
            include_path,
            processes,
            format,
            consent,
        }));
    }
    if args.next().is_some() {
        return Err(ArgumentError);
    }
    if first == "--help" || first == "-h" {
        Ok(Command::Help)
    } else if first == "--version" || first == "-V" {
        Ok(Command::Version)
    } else {
        Err(ArgumentError)
    }
}
