use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use clap_complete::{Shell, generate};
use dialoguer::theme::ColorfulTheme;

use crate::agent::{agent_configs, resolve_agents};
use crate::config::{
    SkillzSource, config_location, config_location_auto, read_config, update_config,
};
use crate::error::{Result, SkillzError};
use crate::git::{clone_repo, head_revision, remote_revision};
use crate::install::{
    InstallMode, agent_skills_base, canonical_skills_dir, install_skill, sanitize_name,
};
use crate::lock::{remove_lock_entry, update_lock_for_skill};
use crate::skill::{discover_skills, parse_skill_md, select_skills};
use crate::source::{Source, parse_source};
use crate::ui;

/// CLI argument parser definition.
#[derive(Parser)]
#[command(
    name = "skills",
    version,
    about = "The CLI for the agent skills",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level CLI commands.
#[derive(Subcommand)]
pub enum Command {
    #[command(aliases = ["a", "install", "i"], about = "Install skills from a source")]
    Add(AddArgs),
    #[command(aliases = ["rm", "r"], about = "Remove installed skills")]
    Remove(RemoveArgs),
    #[command(aliases = ["ls"], about = "List installed skills")]
    List(ListArgs),
    #[command(aliases = ["search", "f", "s"], about = "Search for skills by keyword")]
    Find(FindArgs),
    #[command(about = "Check for available skill updates")]
    Check,
    #[command(aliases = ["upgrade"], about = "Update all skills to latest versions")]
    Update,
    #[command(about = "Create a new SKILL.md template")]
    Init(InitArgs),
    #[command(aliases = ["completion"], about = "Generate shell completion scripts")]
    Completions(CompletionsArgs),
}

/// Arguments for `skills add`.
#[derive(Args, Clone)]
#[command(about = "Install skills from a repository or archive")]
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

/// Arguments for `skills remove`.
#[derive(Args, Clone)]
#[command(about = "Remove installed skills")]
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

/// Arguments for `skills list`.
#[derive(Args, Clone)]
#[command(about = "List installed skills")]
pub struct ListArgs {
    #[arg(short = 'g', long = "global")]
    pub global: bool,
    #[arg(short = 'a', long = "agent", num_args = 1..)]
    pub agent: Vec<String>,
}

/// Arguments for `skills find`.
#[derive(Args, Clone)]
#[command(about = "Search for skills by keyword")]
pub struct FindArgs {
    pub query: Option<String>,
}

/// Arguments for `skills init`.
#[derive(Args, Clone)]
#[command(about = "Initialize a new skill template")]
pub struct InitArgs {
    pub name: Option<String>,
}

/// Arguments for `skills completions`.
#[derive(Args, Clone)]
#[command(about = "Generate shell completion scripts")]
pub struct CompletionsArgs {
    #[arg(value_enum)]
    pub shell: Shell,
}

const SEARCH_API_BASE: &str = "https://skills.sh";

#[derive(Debug, serde::Deserialize)]
struct SearchApiResponse {
    skills: Vec<SearchApiSkill>,
}

#[derive(Debug, serde::Deserialize)]
struct SearchApiSkill {
    name: String,
    installs: Option<u64>,
    source: Option<String>,
}

#[derive(Debug)]
struct UpdateEntry {
    source_key: String,
    source: SkillzSource,
    latest_revision: String,
}

fn prompt_for_skills(skills: &[crate::skill::Skill]) -> Result<Vec<String>> {
    let max_width = console::Term::stdout().size().1 as usize;
    let items: Vec<String> = skills
        .iter()
        .map(|s| format_skill_line(&s.name, &s.description, max_width))
        .collect();
    if items.is_empty() {
        return Ok(vec![]);
    }

    let selection = dialoguer::MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select skills to install")
        .items(&items)
        .max_length(12)
        .interact()
        .map_err(|err| SkillzError::Message(err.to_string()))?;
    let selected = selection
        .into_iter()
        .map(|idx| skills[idx].name.clone())
        .collect();
    Ok(selected)
}

fn format_skill_line(name: &str, description: &str, max_width: usize) -> String {
    let sep = " — ";
    if max_width == 0 {
        return format!(
            "{}{sep}{}",
            console::style(name).bold(),
            console::style(description).dim()
        );
    }

    let max_len = max_width.saturating_sub(4).max(20);
    let name_len = name.chars().count();
    let sep_len = sep.chars().count();
    let available = max_len.saturating_sub(name_len + sep_len);
    if available == 0 {
        return format!("{}", console::style(name).bold());
    }
    let mut desc = description.to_string();
    if desc.chars().count() > available {
        let take = available.saturating_sub(3);
        desc = desc.chars().take(take).collect();
        desc.push_str("...");
    }

    format!(
        "{}{}{}",
        console::style(name).bold(),
        console::style(sep).dim(),
        console::style(desc).dim()
    )
}

fn prompt_for_agents() -> Result<Vec<String>> {
    let agents = agent_configs();
    let items: Vec<String> = agents.iter().map(|a| a.display_name.to_string()).collect();
    if items.is_empty() {
        return Ok(vec![]);
    }

    let selection = dialoguer::MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select agents to install to")
        .items(&items)
        .interact()
        .map_err(|err| SkillzError::Message(err.to_string()))?;
    let selected = selection
        .into_iter()
        .map(|idx| agents[idx].name.to_string())
        .collect();
    Ok(selected)
}

/// Initializes a new SKILL.md file in the current or named directory.
pub fn run_init(args: InitArgs) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let has_name = args.name.is_some();
    let skill_name = args.name.clone().unwrap_or_else(|| {
        cwd.file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("skill")
            .to_string()
    });

    let skill_dir = if has_name {
        cwd.join(&skill_name)
    } else {
        cwd.clone()
    };
    let skill_file = skill_dir.join("SKILL.md");

    if skill_file.exists() {
        ui::warn(&format!(
            "Skill already exists at {}",
            display_path(&skill_file)
        ));
        return Ok(());
    }

    if has_name {
        std::fs::create_dir_all(&skill_dir)?;
    }

    let content = format!(
        "---\nname: {name}\ndescription: A brief description of what this skill does\n---\n\n# {name}\n\nInstructions for the agent to follow when this skill is activated.\n\n## When to use\n\nDescribe when this skill should be used.\n\n## Instructions\n\n1. First step\n2. Second step\n3. Additional steps as needed\n",
        name = skill_name
    );

    std::fs::write(&skill_file, content)?;

    ui::success(&format!("Initialized skill: {}", skill_name));
    ui::info(&format!("Created: {}", display_path(&skill_file)));
    Ok(())
}

/// Prints shell completion scripts to stdout.
pub fn run_completions(args: CompletionsArgs) -> Result<()> {
    use clap::CommandFactory;
    let mut cmd = crate::cli::Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(args.shell, &mut cmd, bin_name, &mut io::stdout());
    Ok(())
}

/// Installs skills from a local path or git source.
pub fn run_add(mut args: AddArgs) -> Result<()> {
    if args.all {
        args.skill = vec!["*".to_string()];
        args.agent = vec!["*".to_string()];
        args.yes = true;
    }

    let source = parse_source(&args.source)?;

    let should_prompt_agents = !args.list;
    if should_prompt_agents && args.agent.is_empty() && !args.yes {
        args.agent = prompt_for_agents()?;
    }

    if should_prompt_agents
        && !(args.agent.is_empty() || (args.agent.len() == 1 && args.agent[0] == "*"))
    {
        let valid: HashSet<&str> = agent_configs().iter().map(|a| a.name).collect();
        let invalid: Vec<String> = args
            .agent
            .iter()
            .filter(|name| !valid.contains(name.as_str()))
            .cloned()
            .collect();
        if !invalid.is_empty() {
            let valid_list = agent_configs()
                .iter()
                .map(|a| a.name)
                .collect::<Vec<_>>()
                .join(", ");
            return Err(SkillzError::Message(format!(
                "Invalid agents: {}. Valid agents: {}",
                invalid.join(", "),
                valid_list
            )));
        }
    }

    let agents = if should_prompt_agents {
        let agents = resolve_agents(&args.agent);
        if agents.is_empty() {
            return Err(SkillzError::Message("No agents selected".to_string()));
        }
        agents
    } else {
        Vec::new()
    };

    let supports_global = agents
        .iter()
        .any(|agent| !agent.global_skills_dir.is_empty());
    let mut install_global = args.global;
    if should_prompt_agents && supports_global && !args.global && !args.yes {
        let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Installation scope")
            .items(["Project (current directory)", "Global (home directory)"])
            .default(0)
            .interact()
            .map_err(|err| SkillzError::Message(err.to_string()))?;
        install_global = selection == 1;
    }

    let mut install_mode = if args.copy {
        InstallMode::Copy
    } else {
        InstallMode::Symlink
    };
    if should_prompt_agents && !args.yes && !args.copy {
        let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Installation method")
            .items(["Symlink (recommended)", "Copy to each agent"])
            .default(0)
            .interact()
            .map_err(|err| SkillzError::Message(err.to_string()))?;
        if selection == 1 {
            install_mode = InstallMode::Copy;
        }
    }

    let (base_path, _temp): (PathBuf, Option<tempfile::TempDir>) = match &source {
        Source::Local { path } => (path.clone(), None),
        Source::Git { url, .. } => {
            let temp_dir = tempfile::tempdir()?;
            let spinner = ui::spinner("Cloning repository...");
            let result = clone_repo(url, temp_dir.path());
            match result {
                Ok(()) => spinner.finish_with_message("Repository cloned"),
                Err(err) => {
                    spinner.finish_with_message("Repository clone failed");
                    return Err(err);
                }
            }
            (temp_dir.path().to_path_buf(), Some(temp_dir))
        }
    };

    let (subpath, source_info) = match &source {
        Source::Local { .. } => (None, None),
        Source::Git { subpath, info, .. } => (subpath.clone(), Some(info.clone())),
    };

    let revision = match &source {
        Source::Local { .. } => None,
        Source::Git { .. } => head_revision(&base_path).ok(),
    };

    let skills = discover_skills(&base_path, subpath.as_deref(), args.full_depth)?;

    if skills.is_empty() {
        return Err(SkillzError::Message(
            "No skills found in source".to_string(),
        ));
    }

    if args.list {
        ui::heading("Available skills");
        for skill in &skills {
            ui::list_item(&format!("{}: {}", skill.name, skill.description));
        }
        return Ok(());
    }

    if args.skill.is_empty() && !args.yes {
        args.skill = prompt_for_skills(&skills)?;
    }

    let selected_skills = select_skills(&skills, &args.skill);
    if selected_skills.is_empty() {
        return Err(SkillzError::Message(
            "No matching skills selected".to_string(),
        ));
    }

    let install_spinner = ui::spinner("Installing skills...");
    for skill in &selected_skills {
        for agent in &agents {
            install_skill(skill, agent, install_global, install_mode)?;
        }

        if let Some(info) = source_info.clone() {
            update_lock_for_skill(skill, &info, &base_path)?;
        }
    }
    install_spinner.finish_with_message("Installation complete");

    let config_location = config_location(install_global)?;
    let source_key = match &source {
        Source::Local { path } => path.to_string_lossy().to_string(),
        Source::Git { url, .. } => url.clone(),
    };
    let source_entry = match &source {
        Source::Local { .. } => SkillzSource {
            source_type: "local".to_string(),
            branch: None,
            subpath: None,
            revision: None,
            skills: vec![],
        },
        Source::Git { subpath, info, .. } => SkillzSource {
            source_type: info.source_type.clone(),
            branch: info.github_branch.clone(),
            subpath: subpath.as_ref().map(|p| p.to_string_lossy().to_string()),
            revision: None,
            skills: vec![],
        },
    };
    let skill_names: Vec<String> = selected_skills.iter().map(|s| s.name.clone()).collect();
    update_config(
        &config_location.path,
        &source_key,
        source_entry,
        &skill_names,
        revision,
    )?;

    ui::success(&format!(
        "Installed {} skill(s) to {} agent(s)",
        selected_skills.len(),
        agents.len()
    ));
    Ok(())
}

/// Removes installed skills from agent directories.
pub fn run_remove(mut args: RemoveArgs) -> Result<()> {
    if args.all {
        args.skill = vec!["*".to_string()];
        args.agent = vec!["*".to_string()];
        args.yes = true;
    }

    let mut requested_skills = args.skills.clone();
    requested_skills.extend(args.skill.clone());

    let agents = resolve_agents(&args.agent);
    if agents.is_empty() {
        return Err(SkillzError::Message("No agents selected".to_string()));
    }

    let skill_names = if requested_skills.is_empty() {
        if !console::Term::stdout().is_term() {
            return Err(SkillzError::Message(
                "No skills provided (interactive remove requires a TTY)".to_string(),
            ));
        }

        let mut names = std::collections::BTreeSet::new();
        for agent in &agents {
            let base = agent_skills_base(agent, args.global)?;
            if !base.exists() {
                continue;
            }
            for entry in std::fs::read_dir(&base)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    if let Some(skill) = parse_skill_md(&entry.path().join("SKILL.md"))? {
                        names.insert(skill.name);
                    } else if let Some(name) = entry.file_name().to_str() {
                        names.insert(name.to_string());
                    }
                }
            }
        }

        if names.is_empty() {
            return Err(SkillzError::Message(
                "No skills available to remove".to_string(),
            ));
        }

        let items: Vec<String> = names.into_iter().collect();
        let selection = dialoguer::MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select skills to remove")
            .items(&items)
            .max_length(12)
            .interact()
            .map_err(|err| SkillzError::Message(err.to_string()))?;
        if selection.is_empty() {
            return Err(SkillzError::Message("No skills selected".to_string()));
        }
        selection
            .into_iter()
            .map(|idx| items[idx].clone())
            .collect()
    } else {
        requested_skills
    };

    let mut removed = 0usize;

    for agent in &agents {
        let base = agent_skills_base(agent, args.global)?;
        if !base.exists() {
            continue;
        }

        if skill_names.len() == 1 && skill_names[0] == "*" {
            for entry in std::fs::read_dir(&base)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    std::fs::remove_dir_all(entry.path())?;
                    removed += 1;
                }
            }
            continue;
        }

        for name in &skill_names {
            let sanitized = sanitize_name(name);
            let target = base.join(&sanitized);
            if target.exists() {
                std::fs::remove_dir_all(&target)?;
                removed += 1;
                remove_lock_entry(name).ok();
            }
        }
    }

    ui::success(&format!("Removed {} skill(s)", removed));
    Ok(())
}

/// Lists installed skills for agents or the canonical store.
pub fn run_list(args: ListArgs) -> Result<()> {
    if args.agent.is_empty() {
        let canonical = canonical_skills_dir(args.global)?;
        if canonical.exists() {
            let mut names = Vec::new();
            for entry in std::fs::read_dir(&canonical)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    if let Some(skill) = parse_skill_md(&entry.path().join("SKILL.md"))? {
                        names.push(skill.name);
                    } else if let Some(name) = entry.file_name().to_str() {
                        names.push(name.to_string());
                    }
                }
            }

            if !names.is_empty() {
                ui::heading("Skills");
                names.sort();
                for name in names {
                    ui::list_item(&name);
                }
                return Ok(());
            }
        }

        if !args.global {
            let global_canonical = canonical_skills_dir(true)?;
            if global_canonical.exists() {
                let mut names = Vec::new();
                for entry in std::fs::read_dir(&global_canonical)? {
                    let entry = entry?;
                    if entry.path().is_dir() {
                        if let Some(skill) = parse_skill_md(&entry.path().join("SKILL.md"))? {
                            names.push(skill.name);
                        } else if let Some(name) = entry.file_name().to_str() {
                            names.push(name.to_string());
                        }
                    }
                }

                if !names.is_empty() {
                    ui::heading("Global skills (use -g to list directly)");
                    names.sort();
                    for name in names {
                        ui::list_item(&name);
                    }
                    return Ok(());
                }
            }
        }
    }

    let agents = resolve_agents(&args.agent);
    if agents.is_empty() {
        return Err(SkillzError::Message("No agents selected".to_string()));
    }

    for agent in agents {
        let base = agent_skills_base(&agent, args.global)?;
        ui::heading(&format!("{}:", agent.display_name));
        if !base.exists() {
            ui::info("  (no skills installed)");
            continue;
        }

        let mut names = Vec::new();
        for entry in std::fs::read_dir(base)? {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(skill) = parse_skill_md(&entry.path().join("SKILL.md"))? {
                    names.push(skill.name);
                } else if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }

        if names.is_empty() {
            ui::info("  (no skills installed)");
        } else {
            names.sort();
            for name in names {
                ui::list_item(&name);
            }
        }
    }

    Ok(())
}

/// Searches for skills using the remote registry API.
pub fn run_find(args: FindArgs) -> Result<()> {
    let Some(query) = args.query else {
        ui::info("Usage: skills find <query>");
        ui::info("Tip: use `skills find typescript`");
        return Ok(());
    };

    let url = format!(
        "{}/api/search?q={}&limit=10",
        SEARCH_API_BASE,
        urlencoding::encode(&query)
    );
    let res = reqwest::blocking::get(url)?;
    if !res.status().is_success() {
        ui::warn(&format!("Search failed: {}", res.status()));
        return Ok(());
    }

    let data: SearchApiResponse = res.json()?;
    if data.skills.is_empty() {
        ui::info("No skills found");
        return Ok(());
    }

    ui::heading("Results");
    for skill in data.skills {
        let source = skill.source.clone().unwrap_or_default();
        let installs = skill.installs.unwrap_or(0);
        ui::list_item(&format!(
            "{} ({}) - {} installs",
            skill.name, source, installs
        ));
        if !source.is_empty() {
            ui::info(&format!(
                "  add: skills add {} --skill {}",
                source, skill.name
            ));
        }
    }

    Ok(())
}

/// Checks for updates for skills tracked in config.
pub fn run_check() -> Result<()> {
    ui::info("Checking for skill updates...");
    let location = config_location_auto()?;
    let config = read_config(&location.path)?;
    if config.sources.is_empty() {
        ui::info("No skills tracked in config.");
        return Ok(());
    }

    let mut updates = Vec::new();
    for (source_key, source) in &config.sources {
        if source.source_type == "local" {
            continue;
        }
        let latest = remote_revision(source_key, source.branch.as_deref())?;
        let current = source.revision.clone().unwrap_or_default();
        if current.is_empty() || current != latest {
            updates.push(UpdateEntry {
                source_key: source_key.clone(),
                source: source.clone(),
                latest_revision: latest,
            });
        }
    }

    if updates.is_empty() {
        ui::success("All skills are up to date");
        return Ok(());
    }

    ui::heading(&format!("{} update(s) available", updates.len()));
    for update in updates {
        ui::list_item(&format!(
            "{} ({})",
            update.source_key, update.latest_revision
        ));
    }

    Ok(())
}

/// Updates all skills that have updates available.
pub fn run_update() -> Result<()> {
    ui::info("Checking for skill updates...");
    let location = config_location_auto()?;
    let config = read_config(&location.path)?;
    if config.sources.is_empty() {
        ui::info("No skills tracked in config.");
        return Ok(());
    }

    let mut updates = Vec::new();
    for (source_key, source) in &config.sources {
        if source.source_type == "local" {
            continue;
        }
        let latest = remote_revision(source_key, source.branch.as_deref())?;
        let current = source.revision.clone().unwrap_or_default();
        if current.is_empty() || current != latest {
            updates.push(UpdateEntry {
                source_key: source_key.clone(),
                source: source.clone(),
                latest_revision: latest,
            });
        }
    }

    if updates.is_empty() {
        ui::success("All skills are up to date");
        return Ok(());
    }

    ui::heading(&format!("Found {} update(s)", updates.len()));

    let mut success = 0usize;
    let mut failed = 0usize;

    for update in updates {
        ui::info(&format!("Updating {}...", update.source_key));

        let args = AddArgs {
            source: update.source_key.clone(),
            global: location.is_global,
            copy: false,
            agent: vec![],
            skill: update.source.skills.clone(),
            list: false,
            yes: true,
            all: false,
            full_depth: false,
        };

        match run_add(args) {
            Ok(_) => {
                success += 1;
                ui::info(&format!("  Updated {}", update.source_key));
            }
            Err(err) => {
                failed += 1;
                ui::warn(&format!(
                    "  Failed to update {}: {}",
                    update.source_key, err
                ));
            }
        }
    }

    ui::success(&format!("Updated {} source(s), {} failed", success, failed));
    Ok(())
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
