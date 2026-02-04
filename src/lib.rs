#![allow(clippy::result_large_err)]

pub mod agent;

mod cli;
mod commands;
mod error;
mod git;
pub mod install;
mod lock;
pub mod skill;
pub mod source;
pub mod ui;

pub use error::{Result, SkillzError};

pub fn run() -> Result<()> {
    use clap::Parser;
    let cli = cli::Cli::parse();

    match cli.command {
        None => {
            print_help();
            Ok(())
        }
        Some(cli::Command::Add(args)) => commands::run_add(args),
        Some(cli::Command::Remove(args)) => commands::run_remove(args),
        Some(cli::Command::List(args)) => commands::run_list(args),
        Some(cli::Command::Find(args)) => commands::run_find(args),
        Some(cli::Command::Check) => commands::run_check(),
        Some(cli::Command::Update) => commands::run_update(),
        Some(cli::Command::Init(args)) => commands::run_init(args),
    }
}

fn print_help() {
    ui::heading("skills");
    println!("The CLI for the open agent skills ecosystem\n");
    ui::info("Usage: skills <command> [options]\n");
    ui::heading("Commands");
    ui::list_item("add <package>     Add a skill package");
    ui::list_item("remove [skills]   Remove installed skills");
    ui::list_item("list, ls          List installed skills");
    ui::list_item("find [query]      Search for skills by keyword");
    ui::list_item("init [name]       Initialize a skill (creates <name>/SKILL.md or ./SKILL.md)");
    ui::list_item("check             Check for available skill updates");
    ui::list_item("update            Update all skills to latest versions");
}
