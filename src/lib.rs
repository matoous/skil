#![allow(clippy::result_large_err)]

pub mod agent;

mod cli;
mod config;
mod error;
mod git;
pub mod install;
mod lock;
pub mod skill;
pub mod source;
pub mod ui;

pub use error::{Result, SkilError};

/// Entry point for the CLI command dispatch.
pub fn run() -> Result<()> {
    use clap::Parser;
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Add(args) => cli::run_add(args),
        cli::Command::Remove(args) => cli::run_remove(args),
        cli::Command::List(args) => cli::run_list(args),
        cli::Command::Find(args) => cli::run_find(args),
        cli::Command::Check => cli::run_check(),
        cli::Command::Update => cli::run_update(),
        cli::Command::Init(args) => cli::run_init(args),
        cli::Command::Completions(args) => cli::run_completions(args),
    }
}
