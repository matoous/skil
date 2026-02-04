use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "skills",
    version,
    about = "The CLI for the open agent skills ecosystem"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(aliases = ["a", "install", "i"])]
    Add(AddArgs),
    #[command(aliases = ["rm", "r"])]
    Remove(RemoveArgs),
    #[command(aliases = ["ls"])]
    List(ListArgs),
    #[command(aliases = ["search", "f", "s"])]
    Find(FindArgs),
    Check,
    #[command(aliases = ["upgrade"])]
    Update,
    Init(InitArgs),
}

#[derive(Args, Clone)]
pub struct AddArgs {
    pub source: String,
    #[arg(short = 'g', long = "global")]
    pub global: bool,
    #[arg(long = "copy")]
    pub copy: bool,
    #[arg(short = 'a', long = "agent", num_args = 1..)]
    pub agent: Vec<String>,
    #[arg(short = 's', long = "skill", num_args = 1..)]
    pub skill: Vec<String>,
    #[arg(short = 'l', long = "list")]
    pub list: bool,
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,
    #[arg(long = "all")]
    pub all: bool,
    #[arg(long = "full-depth")]
    pub full_depth: bool,
}

#[derive(Args, Clone)]
pub struct RemoveArgs {
    pub skills: Vec<String>,
    #[arg(short = 'g', long = "global")]
    pub global: bool,
    #[arg(short = 'a', long = "agent", num_args = 1..)]
    pub agent: Vec<String>,
    #[arg(short = 's', long = "skill", num_args = 1..)]
    pub skill: Vec<String>,
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,
    #[arg(long = "all")]
    pub all: bool,
}

#[derive(Args, Clone)]
pub struct ListArgs {
    #[arg(short = 'g', long = "global")]
    pub global: bool,
    #[arg(short = 'a', long = "agent", num_args = 1..)]
    pub agent: Vec<String>,
}

#[derive(Args, Clone)]
pub struct FindArgs {
    pub query: Option<String>,
}

#[derive(Args, Clone)]
pub struct InitArgs {
    pub name: Option<String>,
}
