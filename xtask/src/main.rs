//! Repository automation tasks for `rustitch`.

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{self, Command};

fn main() {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_help();
        process::exit(1);
    };
    let passthrough = normalize_passthrough_args(args);

    match command.as_str() {
        "help" | "--help" | "-h" => {
            print_help();
        }
        "check" => run_standard_command(["check", "--workspace"], &passthrough),
        "fmt" => run_standard_command(["fmt", "--all"], &passthrough),
        "lint" => run_standard_command(
            ["clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"],
            &passthrough,
        ),
        "test" => run_standard_command(["test", "--workspace"], &passthrough),
        "feature-matrix" => run_feature_matrix(),
        other => {
            eprintln!("Unknown xtask command: {other}");
            print_help();
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("cargo xtask <command>");
    println!();
    println!("Commands:");
    println!("  help            Show this message");
    println!("  check           Run `cargo check --workspace`");
    println!("  fmt             Run `cargo fmt --all`");
    println!("  lint            Run workspace clippy with warnings denied");
    println!("  test            Run `cargo test --workspace`");
    println!("  feature-matrix  Validate minimal root feature combinations");
}

fn normalize_passthrough_args(args: impl Iterator<Item = String>) -> Vec<String> {
    let mut args: Vec<String> = args.collect();
    if matches!(args.first(), Some(separator) if separator == "--") {
        args.remove(0);
    }
    args
}

fn run_standard_command<const N: usize>(base: [&str; N], passthrough: &[String]) {
    let mut args: Vec<OsString> = base.into_iter().map(OsString::from).collect();
    args.extend(passthrough.iter().map(OsString::from));
    run_cargo(args);
}

fn run_feature_matrix() {
    const FEATURE_MATRIX: &[(&str, &[&str])] = &[
        ("base", &["check", "-p", "rustitch", "--no-default-features"]),
        ("auth", &["check", "-p", "rustitch", "--no-default-features", "--features", "auth"]),
        ("helix", &["check", "-p", "rustitch", "--no-default-features", "--features", "helix"]),
        (
            "eventsub",
            &["check", "-p", "rustitch", "--no-default-features", "--features", "eventsub"],
        ),
        (
            "eventsub-manage",
            &["check", "-p", "rustitch", "--no-default-features", "--features", "eventsub-manage"],
        ),
        (
            "eventsub-webhook",
            &["check", "-p", "rustitch", "--no-default-features", "--features", "eventsub-webhook"],
        ),
        ("chat", &["check", "-p", "rustitch", "--no-default-features", "--features", "chat"]),
        (
            "chat-irc",
            &["check", "-p", "rustitch", "--no-default-features", "--features", "chat-irc"],
        ),
    ];

    for (label, args) in FEATURE_MATRIX {
        println!("==> validating feature set: {label}");
        let command_args: Vec<OsString> = args.iter().map(OsString::from).collect();
        run_cargo(command_args);
    }
}

fn run_cargo(args: Vec<OsString>) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(workspace_root) = manifest_dir.parent().map(PathBuf::from) else {
        eprintln!("xtask manifest directory does not have a workspace parent");
        process::exit(1);
    };

    let display_args = args.iter().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>().join(" ");
    println!("> cargo {display_args}");

    match Command::new("cargo").args(args).current_dir(workspace_root).status() {
        Ok(status) if status.success() => {}
        Ok(status) => process::exit(status.code().unwrap_or(1)),
        Err(error) => {
            eprintln!("failed to run cargo from xtask: {error}");
            process::exit(1);
        }
    }
}
