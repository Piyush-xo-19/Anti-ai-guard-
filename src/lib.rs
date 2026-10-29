//! CLI, shared observations and diagnostic Policy A contracts.
//!
//! Installation inventory requires explicit consent and bounded roots. There is
//! no process collection or competition verification. Inventory is not execution.

pub mod inventory;
pub mod model;
pub mod policy;

use std::ffi::OsString;
use std::fmt;

/// Version from the same package manifest used to build the binary.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Help for the commands implemented in this milestone chunk.
pub const HELP: &str = "Anti-AI Guard
Read-only competition preflight tooling (early development).

Usage: anti-ai-guard [OPTION]
       anti-ai-guard inventory --consent [--bin-root PATH] [--npm-prefix PATH] [--include-path]

Options:
  -h, --help       Show this help
  -V, --version    Show the program version

Inventory inspects only known executable names and npm manifests in approved roots.
--include-path opts into PATH directories; relative/empty/network roots are rejected.
No tool is executed. Findings are installation candidates, never execution violations.
Help/version do not inspect installations, processes, or files.
Process scanning is not implemented.
";

/// Actions available before collectors are introduced in later chunks.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Version,
    Inventory(InventoryOptions),
}

#[derive(Debug, PartialEq, Eq)]
pub struct InventoryOptions {
    pub roots: Vec<inventory::InventoryRoot>,
    pub include_path: bool,
}

/// Expands PATH only after the CLI consent flag has been validated. No filesystem
/// access occurs here. Missing/empty/relative PATH entries fail closed.
pub fn inventory_request(
    options: InventoryOptions,
    path: Option<&std::ffi::OsStr>,
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

/// Invalid CLI input. Deliberately excludes raw argument values, which may
/// contain private data accidentally pasted into the terminal.
#[derive(Debug, PartialEq, Eq)]
pub struct ArgumentError;

impl fmt::Display for ArgumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("unsupported arguments; run anti-ai-guard --help for usage")
    }
}

impl std::error::Error for ArgumentError {}

/// Parse arguments excluding the executable name.
///
/// No arguments show help. Exactly one supported option is accepted. Every
/// other input is rejected, including extra options following `--help`.
/// Operating-system strings avoid panicking on non-Unicode arguments.
pub fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Command, ArgumentError> {
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        return Ok(Command::Help);
    };

    if first == "inventory" {
        let mut consent = false;
        let mut include_path = false;
        let mut roots = Vec::new();
        let mut count = 0;
        while let Some(option) = args.next() {
            count += 1;
            if count > 66 {
                return Err(ArgumentError);
            }
            if option == "--help" && count == 1 && args.next().is_none() {
                return Ok(Command::Help);
            }
            if option == "--consent" && !consent {
                consent = true;
            } else if option == "--include-path" && !include_path {
                include_path = true;
            } else if option == "--bin-root" || option == "--npm-prefix" {
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
        if !consent || (roots.is_empty() && !include_path) {
            return Err(ArgumentError);
        }
        return Ok(Command::Inventory(InventoryOptions {
            roots,
            include_path,
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
