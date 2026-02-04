use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::error::{Result, SkillzError};

const CONFIG_DIR: &str = "skillz";
const CONFIG_FILE: &str = "config.toml";
const LOCAL_CONFIG_FILE: &str = ".skillz.toml";

/// Persistent configuration for installed sources and skills.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SkillzConfig {
    #[serde(rename = "source")]
    pub sources: BTreeMap<String, SkillzSource>,
}

/// A source entry tracked in config.toml.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillzSource {
    #[serde(rename = "source-type")]
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subpath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    pub skills: Vec<String>,
}

/// Resolved config location and whether it is global.
pub struct ConfigLocation {
    pub path: PathBuf,
    pub is_global: bool,
}

/// Returns the config location for local or global installs.
pub fn config_location(global: bool) -> Result<ConfigLocation> {
    if global {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_home = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".config"));
        return Ok(ConfigLocation {
            path: config_home.join(CONFIG_DIR).join(CONFIG_FILE),
            is_global: true,
        });
    }

    let cwd = std::env::current_dir()?;
    Ok(ConfigLocation {
        path: cwd.join(LOCAL_CONFIG_FILE),
        is_global: false,
    })
}

/// Uses the local config if present, otherwise falls back to global.
pub fn config_location_auto() -> Result<ConfigLocation> {
    let local = config_location(false)?;
    if local.path.exists() {
        return Ok(local);
    }
    config_location(true)
}

/// Reads config from disk, returning an empty config if missing.
pub fn read_config(path: &Path) -> Result<SkillzConfig> {
    if !path.exists() {
        return Ok(SkillzConfig::default());
    }
    let content = std::fs::read_to_string(path)?;
    let config: SkillzConfig =
        toml::from_str(&content).map_err(|err| SkillzError::Message(err.to_string()))?;
    Ok(config)
}

/// Writes config to disk, creating parent directories as needed.
pub fn write_config(path: &Path, config: &SkillzConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content =
        toml::to_string_pretty(config).map_err(|err| SkillzError::Message(err.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Updates a config entry with skills and optional revision.
pub fn update_config(
    path: &Path,
    source_key: &str,
    source: SkillzSource,
    skills: &[String],
    revision: Option<String>,
) -> Result<()> {
    let mut config = read_config(path)?;
    let entry = config
        .sources
        .entry(source_key.to_string())
        .or_insert(source);
    let mut combined: BTreeSet<String> = entry.skills.iter().cloned().collect();
    combined.extend(skills.iter().cloned());
    entry.skills = combined.into_iter().collect();
    entry.revision = revision.or(entry.revision.clone());
    write_config(path, &config)?;
    Ok(())
}
