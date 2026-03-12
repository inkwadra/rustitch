//! Repository automation tasks for `rustitch`.

use std::env;
use std::path::PathBuf;
use std::process::{self, Command};

fn main() {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_help();
        process::exit(1);
    };

    match command.as_str() {
        "help" | "--help" | "-h" => {
            print_help();
        }
        "check" => run_cargo(["check", "--workspace"]),
        "fmt" => run_cargo(["fmt", "--all"]),
        "lint" => run_cargo([
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ]),
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
    println!("  help   Show this message");
    println!("  check  Run `cargo check --workspace`");
    println!("  fmt    Run `cargo fmt --all`");
    println!("  lint   Run workspace clippy with warnings denied");
}

fn run_cargo<const N: usize>(args: [&str; N]) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(workspace_root) = manifest_dir.parent().map(PathBuf::from) else {
        eprintln!("xtask manifest directory does not have a workspace parent");
        process::exit(1);
    };

    match Command::new("cargo").args(args).current_dir(workspace_root).status() {
        Ok(status) if status.success() => {}
        Ok(status) => process::exit(status.code().unwrap_or(1)),
        Err(error) => {
            eprintln!("failed to run cargo from xtask: {error}");
            process::exit(1);
        }
    }
}
