use anti_ai_guard::{parse_args, Command, OutputFormat, HELP, VERSION};
use std::io::{self, Write};
use std::process::ExitCode;
fn main() -> ExitCode {
    let (output, code) = match parse_args(std::env::args_os().skip(1)) {
        Ok(Command::Help) => (HELP.to_owned(), 0),
        Ok(Command::Version) => (format!("anti-ai-guard {VERSION}\n"), 0),
        Ok(Command::Collect(options)) => {
            let format = options.format;
            let path = if options.include_path {
                std::env::var_os("PATH")
            } else {
                None
            };
            let cancellation = anti_ai_guard::cancellation::Session::start();
            let result =
                match anti_ai_guard::scan::run(options, path.as_deref(), cancellation.flag()) {
                    Ok(result) => result,
                    Err(error) => {
                        let _ = writeln!(io::stderr(), "error: {error}");
                        return ExitCode::from(2);
                    }
                };
            let public = anti_ai_guard::report::project(&result);
            (
                match format {
                    OutputFormat::Text => public.text(),
                    OutputFormat::Json => public.json(),
                },
                public.exit_code(),
            )
        }
        Err(error) => {
            let _ = writeln!(io::stderr(), "error: {error}");
            return ExitCode::from(2);
        }
    };
    let exit = write_output(&output, code, io::stdout().lock());
    if exit == 1 {
        let _ = writeln!(io::stderr(), "error: unable to write diagnostic output");
    }
    ExitCode::from(exit)
}

fn write_output(output: &str, report_code: u8, mut writer: impl Write) -> u8 {
    match writer.write_all(output.as_bytes()) {
        Ok(()) => report_code,
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct BrokenPipe;
    impl Write for BrokenPipe {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn closed_consumer_is_output_failure_for_complete_and_incomplete_reports() {
        for report_code in [0, 3] {
            assert_eq!(write_output("diagnostic", report_code, BrokenPipe), 1);
            assert_eq!(
                write_output("diagnostic", report_code, Vec::new()),
                report_code
            );
        }
    }
}
