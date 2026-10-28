use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_anti-ai-guard"))
        .args(args)
        .output()
        .expect("CLI executable must run")
}

#[test]
fn no_arguments_show_help_without_scanning() {
    let output = run(&[]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Usage: anti-ai-guard [OPTION]"));
    assert!(text.contains("Scanning is not implemented."));
    assert!(text.contains("does not inspect installations, processes, or files"));
}

#[test]
fn help_aliases_have_the_same_successful_output() {
    let long = run(&["--help"]);
    let short = run(&["-h"]);
    assert!(long.status.success());
    assert!(short.status.success());
    assert!(long.stderr.is_empty());
    assert!(short.stderr.is_empty());
    assert_eq!(long.stdout, short.stdout);
    assert_eq!(long.stdout, run(&[]).stdout);
}

#[test]
fn version_matches_the_built_package() {
    for option in ["--version", "-V"] {
        let output = run(&[option]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().trim(),
            format!("anti-ai-guard {}", env!("CARGO_PKG_VERSION"))
        );
    }
}

#[test]
fn unknown_option_has_exit_code_two_and_stderr_only() {
    let output = run(&["--unknown"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("--help"));
}

#[test]
fn scan_command_is_rejected_until_it_is_implemented() {
    let output = run(&["scan"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}

#[test]
fn extra_arguments_are_rejected_even_after_help_or_version() {
    for args in [
        vec!["--help", "--unknown"],
        vec!["--version", "scan"],
        vec!["--help", "--version"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn errors_do_not_echo_potentially_sensitive_arguments() {
    let secret = "fake-test-secret-do-not-log-7c63";
    let output = run(&[secret]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!String::from_utf8(output.stderr).unwrap().contains(secret));
}
