use std::fs::create_dir_all;

use anyhow::Result;
use clap::Parser;
use xtaskops::ops::{clean_files, cmd, remove_dir};

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    subcommand: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Coverage,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Args::parse();

    match cli.subcommand {
        Command::Coverage => coverage(),
    }
}

// Taken from https://github.com/jondot/xtaskops/blob/master/xtaskops/src/tasks.rs
pub fn coverage() -> Result<()> {
    remove_dir("coverage")?;
    create_dir_all("coverage")?;

    let max_threads = std::thread::available_parallelism()
        .map(|t| t.get())
        .unwrap_or(1);

    println!("=== running coverage ===");

    let cmd_test = cmd!(
        "cargo",
        "test",
        "--target-dir",
        "coverage-target",
        "--all-features"
    );

    cmd_test
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-Cinstrument-coverage")
        .env("LLVM_PROFILE_FILE", "cargo-test-%p-%m.profraw")
        .run()?;
    println!("ok.");

    println!("=== generating report ===");
    let result = cmd!(
        "grcov",
        ".",
        "-t",
        "cobertura,html",
        "--binary-path",
        "./coverage-target/debug/deps",
        "-s",
        ".",
        "--llvm",
        "--parallel",
        "--threads",
        max_threads.to_string(),
        "--branch",
        "--ignore-not-existing",
        "--keep-only",
        "src/*,axum-distributed-routing-macros/src/*",
        "-o",
        "coverage/"
    )
    .run();
    if result.is_err() {
        println!("failed.");
    } else {
        println!("ok.");
    }

    println!("=== cleaning up ===");
    clean_files("**/*.profraw")?;
    println!("ok.");

    result?;

    Ok(())
}
