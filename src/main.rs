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
    match io::stdout().lock().write_all(output.as_bytes()) {
        Ok(()) => ExitCode::from(code),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(_) => {
            let _ = writeln!(io::stderr(), "error: unable to write diagnostic output");
            ExitCode::from(1)
        }
    }
}
