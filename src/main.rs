use std::process::ExitCode;

use anti_ai_guard::{parse_args, Command, HELP, VERSION};
use std::sync::atomic::AtomicBool;

fn main() -> ExitCode {
    match parse_args(std::env::args_os().skip(1)) {
        Ok(Command::Help) => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("anti-ai-guard {VERSION}");
            ExitCode::SUCCESS
        }
        Ok(Command::Inventory(options)) => {
            let path = if options.include_path {
                std::env::var_os("PATH")
            } else {
                None
            };
            let request = match anti_ai_guard::inventory_request(options, path.as_deref()) {
                Ok(request) => request,
                Err(error) => {
                    eprintln!("error: {error}");
                    return ExitCode::from(2);
                }
            };
            let result = anti_ai_guard::inventory::collect(&request, &AtomicBool::new(false));
            println!("Installation inventory only; no process inspection or eligibility issued.");
            for finding in &result.findings {
                println!(
                    "Root {}: {} | {} | Candidate | InventoryOnly",
                    finding.observation.scope.0,
                    finding.selector,
                    finding
                        .package_version
                        .as_deref()
                        .unwrap_or("version unknown")
                );
            }
            for coverage in &result.coverage {
                println!(
                    "Root {} coverage: {:?} {:?}",
                    coverage.target.scope.0, coverage.state, coverage.gaps
                );
            }
            println!(
                "Execution: {:?}; findings: {}; probes: {}; metadata bytes: {}",
                result.execution,
                result.findings.len(),
                result.probes,
                result.bytes_read
            );
            println!("Scope is limited to documented selectors; an empty inventory does not mean a clean device.");
            if result.execution == anti_ai_guard::model::ScanExecution::Finished {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(3)
            }
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}
