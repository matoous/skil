use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::agent::AgentConfig;
use crate::error::Result;
use crate::skill::Skill;

/// Installation strategy for agent skill directories.
#[derive(Clone, Copy)]
pub enum InstallMode {
    Symlink,
    Copy,
}

const AGENTS_DIR: &str = ".agents";
const SKILLS_SUBDIR: &str = "skills";

/// Installs a skill into the canonical store and agent directories.
pub fn install_skill(
    skill: &Skill,
    agent: &AgentConfig,
    global: bool,
    mode: InstallMode,
) -> Result<()> {
    let raw_name = if skill.name.is_empty() {
        "unnamed".to_string()
    } else {
        skill.name.clone()
    };
    let skill_name = sanitize_name(&raw_name);

    let canonical_dir = canonical_skills_dir(global)?.join(&skill_name);
    let agent_dir = agent_skills_base(agent, global)?.join(&skill_name);

    if canonical_dir.exists() {
        std::fs::remove_dir_all(&canonical_dir)?;
    }
    std::fs::create_dir_all(&canonical_dir)?;
    copy_dir(&skill.path, &canonical_dir)?;

    match mode {
        InstallMode::Symlink => {
            if create_symlink(&canonical_dir, &agent_dir).is_err() {
                if agent_dir.exists() {
                    std::fs::remove_dir_all(&agent_dir)?;
                }
                std::fs::create_dir_all(&agent_dir)?;
                copy_dir(&canonical_dir, &agent_dir)?;
            }
        }
        InstallMode::Copy => {
            if agent_dir.exists() {
                std::fs::remove_dir_all(&agent_dir)?;
            }
            std::fs::create_dir_all(&agent_dir)?;
            copy_dir(&canonical_dir, &agent_dir)?;
        }
    }

    Ok(())
}

/// Returns the canonical skill storage directory for a scope.
pub fn canonical_skills_dir(global: bool) -> Result<PathBuf> {
    if global {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Ok(home.join(AGENTS_DIR).join(SKILLS_SUBDIR))
    } else {
        let cwd = std::env::current_dir()?;
        Ok(cwd.join(AGENTS_DIR).join(SKILLS_SUBDIR))
    }
}

/// Returns the base skills directory for a given agent.
pub fn agent_skills_base(agent: &AgentConfig, global: bool) -> Result<PathBuf> {
    if global {
        Ok(PathBuf::from(agent.global_skills_dir.as_str()))
    } else {
        let cwd = std::env::current_dir()?;
        Ok(cwd.join(&agent.skills_dir))
    }
}

/// Recursively copies a directory, skipping ignored folders.
fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    for entry in WalkDir::new(from) {
        let entry = entry?;
        if should_skip_path(from, entry.path()) {
            continue;
        }
        let rel = entry.path().strip_prefix(from).unwrap_or(entry.path());
        let dest = to.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

/// Checks whether a path should be skipped during copy.
fn should_skip_path(root: &Path, path: &Path) -> bool {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let mut components = rel.components().filter_map(|c| c.as_os_str().to_str());
    let Some(first) = components.next() else {
        return false;
    };

    if should_skip_component(first) {
        return true;
    }
    for component in components {
        if should_skip_component(component) {
            return true;
        }
    }

    false
}

/// Returns true for directories that should never be copied.
fn should_skip_component(component: &str) -> bool {
    matches!(
        component,
        ".git" | "node_modules" | "target" | "dist" | "build" | ".next" | ".turbo" | ".cache"
    )
}

/// Creates a directory symlink, replacing any existing path.
fn create_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    if link.exists() {
        if link.is_dir() {
            std::fs::remove_dir_all(link)?;
        } else {
            std::fs::remove_file(link)?;
        }
    }

    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(target, link)
    }
}

/// Normalizes a skill name to a safe directory name.
pub fn sanitize_name(name: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;

    for ch in name.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    let trimmed = out.trim_matches(&['-', '.'][..]).to_string();
    if trimmed.is_empty() {
        "unnamed-skill".to_string()
    } else {
        trimmed.chars().take(255).collect()
    }
}
