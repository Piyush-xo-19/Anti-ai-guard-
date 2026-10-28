//! CLI, shared observations and diagnostic Policy A contracts.
//!
//! No module inspects the host, collects telemetry or grants competition
//! verification. Identity assessments are inputs, not detection proofs.

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

Options:
  -h, --help       Show this help
  -V, --version    Show the program version

M2.2 adds shared library contracts. Scanning is not implemented.
Running this command does not inspect installations, processes, or files.
";

/// Actions available before collectors are introduced in later chunks.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Version,
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
