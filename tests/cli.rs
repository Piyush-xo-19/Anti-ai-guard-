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
    assert!(text.contains("Names alone require review."));
    assert!(text.contains("Help/version do not inspect installations, processes, or files"));
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
fn scan_requires_explicit_consent_and_scope() {
    let output = run(&["scan"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}

#[test]
fn scan_rejects_missing_consent_empty_scope_and_ambiguous_options() {
    for args in [
        vec!["scan", "--consent"],
        vec!["scan", "--processes"],
        vec!["scan", "--consent", "--processes", "--json", "--json"],
        vec!["processes", "--consent", "--bin-root", "fake-private-token"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("fake-private-token"));
    }
}

#[test]
fn json_process_report_is_parseable_and_contains_no_raw_arguments() {
    let output = run(&["processes", "--consent", "--json"]);
    assert!(matches!(output.status.code(), Some(0 | 3)));
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["eligibility"], "NotIssued");
    assert!(value.get("collectors").is_some());
    assert!(value.get("limitations").is_some());
}

#[test]
fn inventory_requires_consent_and_explicit_scope() {
    for args in [
        vec!["inventory"],
        vec!["inventory", "--consent"],
        vec!["inventory", "--bin-root", "C:\\fixture-secret"],
        vec!["inventory", "--consent", "--bin-root"],
        vec!["inventory", "--consent", "--bin-root", "relative"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("fixture-secret"));
    }
}

#[test]
fn inventory_help_does_not_require_consent_or_inspect() {
    let output = run(&["inventory", "--help"]);
    assert!(output.status.success());
    assert_eq!(output.stdout, run(&["--help"]).stdout);
}

#[test]
fn processes_requires_consent_and_rejects_extra_scope_arguments() {
    for args in [
        vec!["processes"],
        vec!["processes", "--unknown"],
        vec!["processes", "--consent", "fake-private-argument"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("fake-private-argument"));
    }
    assert!(run(&["processes", "--help"]).status.success());
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

#[test]
fn help_explains_required_scope_options() {
    let text = String::from_utf8(run(&["--help"]).stdout).unwrap();
    assert!(text
        .contains("Inventory requires at least one of --bin-root, --npm-prefix or --include-path"));
    assert!(text.contains("Scan requires at least one of those scope options or --processes"));
}

#[test]
fn opted_in_path_errors_are_specific_and_never_echo_entries() {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let root = if cfg!(windows) {
        r"C:\fixture"
    } else {
        "/fixture"
    };
    for (value, message) in [
        (format!("{root}{separator}"), "invalid entry 2"),
        ("fake-secret-relative-entry".into(), "invalid entry 1"),
        (
            (0..33)
                .map(|i| format!("{root}{i}"))
                .collect::<Vec<_>>()
                .join(&separator.to_string()),
            "more than 32 distinct",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_anti-ai-guard"))
            .args(["scan", "--consent", "--include-path"])
            .env("PATH", value)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("PATH scope rejected"));
        assert!(error.contains(message), "{error}");
        assert!(!error.contains("fake-secret"));
        assert!(output.stdout.is_empty());
    }
}
