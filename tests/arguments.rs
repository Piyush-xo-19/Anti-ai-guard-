//! Scope validation uses synthetic paths only; no filesystem or process access.
use anti_ai_guard::{inventory_request, parse_args, ArgumentError, Command, InventoryOptions};
use std::ffi::OsString;

fn root(index: usize) -> String {
    if cfg!(windows) {
        format!(r"C:\synthetic-scope-{index}")
    } else {
        format!("/synthetic-scope-{index}")
    }
}

fn options(roots: impl IntoIterator<Item = (String, &'static str)>) -> anti_ai_guard::ScanOptions {
    let mut args = vec![OsString::from("scan"), OsString::from("--consent")];
    for (path, selector) in roots {
        args.extend([selector.into(), path.into()]);
    }
    let Command::Collect(options) = parse_args(args).unwrap() else {
        panic!("expected collection options")
    };
    options
}

fn request(
    options: anti_ai_guard::ScanOptions,
) -> Result<anti_ai_guard::inventory::InventoryRequest, ArgumentError> {
    inventory_request(
        InventoryOptions {
            roots: options.roots,
            include_path: options.include_path,
        },
        None,
    )
}

#[test]
fn cli_duplicates_are_deduplicated_before_unique_scope_limit() {
    let duplicates = (0..33).map(|_| (root(0), "--bin-root"));
    assert!(request(options(duplicates)).is_ok());
    assert!(request(options((0..32).map(|i| (root(i), "--bin-root")))).is_ok());
    assert!(matches!(
        request(options((0..33).map(|i| (root(i), "--bin-root")))),
        Err(ArgumentError::Unsupported)
    ));
    // Different selectors on the same path consume independent scope slots.
    let mixed = (0..17).flat_map(|i| [(root(i), "--bin-root"), (root(i), "--npm-prefix")]);
    assert!(matches!(
        request(options(mixed)),
        Err(ArgumentError::Unsupported)
    ));
}

#[test]
fn removing_raw_root_limit_preserves_bounded_argument_input() {
    let mut args: Vec<OsString> = vec!["scan".into(), "--consent".into(), "--json".into()];
    for _ in 0..64 {
        args.extend(["--bin-root".into(), root(0).into()]);
    }
    assert_eq!(args.len(), 131);
    let Command::Collect(parsed) = parse_args(args.clone()).unwrap() else {
        panic!("expected collection")
    };
    assert!(request(parsed).is_ok());
    args.push("--processes".into());
    assert!(matches!(parse_args(args), Err(ArgumentError::Unsupported)));
}

#[test]
fn combined_path_limit_error_explains_combined_scope_without_echoing_paths() {
    let roots = options((0..32).map(|i| (root(i), "--bin-root"))).roots;
    let path = OsString::from(root(32));
    let error = inventory_request(
        InventoryOptions {
            roots,
            include_path: true,
        },
        Some(&path),
    )
    .unwrap_err();
    assert_eq!(error, ArgumentError::TooManyPathRoots);
    let message = error.to_string();
    assert!(message.contains("combined explicit roots and PATH"));
    assert!(!message.contains("use explicit roots"));
    assert!(!message.contains("synthetic-scope"));
}
