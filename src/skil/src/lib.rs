#![allow(clippy::result_large_err)]

mod cli;
pub mod ui;

pub use skil_core::{Result, SkilError};

/// Entry point for the CLI command dispatch.
pub fn run() -> Result<()> {
    use clap::Parser;
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Add(args) => cli::run_add(args),
        cli::Command::Install(args) => cli::run_install(args),
        cli::Command::Remove(args) => cli::run_remove(args),
        cli::Command::List(args) => cli::run_list(args),
        cli::Command::Find(args) => cli::run_find(args),
        cli::Command::Check => cli::run_check(),
        cli::Command::Update => cli::run_update(),
        cli::Command::Init(args) => cli::run_init(args),
        cli::Command::Completions(args) => cli::run_completions(args),
        cli::Command::Docs(args) => skil_docs::run_docs(args),
    }
}
