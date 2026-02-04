use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::skill::Skill;
use crate::source::SourceInfo;
use crate::ui;

const AGENTS_DIR: &str = ".agents";
const LOCK_FILE: &str = ".skill-lock.json";
const LOCK_VERSION: u32 = 3;

/// Entry for a single installed skill in the lock file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillLockEntry {
    pub source: String,
    #[serde(rename = "sourceType")]
    pub source_type: String,
    #[serde(rename = "sourceUrl")]
    pub source_url: String,
    #[serde(rename = "skillPath", skip_serializing_if = "Option::is_none")]
    pub skill_path: Option<String>,
    #[serde(rename = "sourceBranch", skip_serializing_if = "Option::is_none")]
    pub source_branch: Option<String>,
    #[serde(rename = "skillFolderHash")]
    pub skill_folder_hash: String,
    #[serde(rename = "installedAt")]
    pub installed_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// Lock file structure tracking installed skills.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillLockFile {
    pub version: u32,
    pub skills: BTreeMap<String, SkillLockEntry>,
}

/// Adds or updates a skill entry in the lock file.
pub fn update_lock_for_skill(skill: &Skill, info: &SourceInfo, base_path: &Path) -> Result<()> {
    let mut lock = read_lock()?;
    let now = Timestamp::now().to_string();

    let skill_path = skill
        .path
        .strip_prefix(base_path)
        .ok()
        .map(|p| p.join("SKILL.md"));
    let skill_path_str = skill_path.as_ref().map(|p| p.to_string_lossy().to_string());

    let folder_hash = if let Some(owner_repo) = &info.github_owner_repo {
        fetch_skill_folder_hash(
            owner_repo,
            info.github_branch.as_deref(),
            skill_path_str.clone().unwrap_or_default(),
        )
        .unwrap_or_default()
    } else {
        String::new()
    };

    let entry = SkillLockEntry {
        source: info.source_id.clone(),
        source_type: info.source_type.clone(),
        source_url: info.source_url.clone(),
        skill_path: skill_path_str.clone(),
        source_branch: info.github_branch.clone(),
        skill_folder_hash: folder_hash,
        installed_at: lock
            .skills
            .get(&skill.name)
            .map(|e| e.installed_at.clone())
            .unwrap_or_else(|| now.clone()),
        updated_at: now,
    };

    lock.skills.insert(skill.name.clone(), entry);
    write_lock(&lock)?;
    Ok(())
}

/// Reads the lock file, resetting on incompatible versions.
pub fn read_lock() -> Result<SkillLockFile> {
    let path = lock_path()?;
    if !path.exists() {
        return Ok(SkillLockFile {
            version: LOCK_VERSION,
            skills: BTreeMap::new(),
        });
    }

    let content = std::fs::read_to_string(path)?;
    let mut lock: SkillLockFile = serde_json::from_str(&content)?;
    if lock.version < LOCK_VERSION {
        ui::warn(&format!(
            "Lock file version {} is older than {}, resetting lock file.",
            lock.version, LOCK_VERSION
        ));
        lock = SkillLockFile {
            version: LOCK_VERSION,
            skills: BTreeMap::new(),
        };
    }
    Ok(lock)
}

/// Writes the lock file to disk.
fn write_lock(lock: &SkillLockFile) -> Result<()> {
    let path = lock_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(lock)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Returns the full path to the lock file.
fn lock_path() -> Result<PathBuf> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    Ok(home.join(AGENTS_DIR).join(LOCK_FILE))
}

/// Removes a skill entry from the lock file.
pub fn remove_lock_entry(skill_name: &str) -> Result<()> {
    let mut lock = read_lock()?;
    if lock.skills.remove(skill_name).is_some() {
        write_lock(&lock)?;
    }
    Ok(())
}

/// Fetches a remote folder tree hash for update checks.
pub fn fetch_skill_folder_hash(
    owner_repo: &str,
    branch: Option<&str>,
    skill_path: String,
) -> Option<String> {
    let mut folder_path = skill_path.replace('\\', "/");
    if folder_path.ends_with("/SKILL.md") {
        folder_path.truncate(folder_path.len() - 9);
    } else if folder_path.ends_with("SKILL.md") {
        folder_path.truncate(folder_path.len() - 8);
    }
    if folder_path.ends_with('/') {
        folder_path.pop();
    }

    let mut branches = Vec::new();
    if let Some(branch) = branch {
        branches.push(branch.to_string());
    } else {
        branches.push("main".to_string());
        branches.push("master".to_string());
    }
    let client = reqwest::blocking::Client::new();

    for branch in branches {
        let url = format!(
            "https://api.github.com/repos/{}/git/trees/{}?recursive=1",
            owner_repo, branch
        );
        let res = client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "skills-cli")
            .send();
        let Ok(res) = res else { continue };
        if !res.status().is_success() {
            continue;
        }

        let json: serde_json::Value = match res.json() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let tree_sha = json.get("sha").and_then(|v| v.as_str()).unwrap_or("");
        if folder_path.is_empty() && !tree_sha.is_empty() {
            return Some(tree_sha.to_string());
        }

        if let Some(tree) = json.get("tree").and_then(|v| v.as_array()) {
            for entry in tree {
                let path = entry.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let typ = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let sha = entry.get("sha").and_then(|v| v.as_str()).unwrap_or("");
                if typ == "tree" && path == folder_path && !sha.is_empty() {
                    return Some(sha.to_string());
                }
            }
        }
    }

    None
}
