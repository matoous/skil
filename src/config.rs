use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::error::{Result, SkilError};

const CONFIG_DIR: &str = "skil";
const CONFIG_FILE: &str = "config.toml";
const LOCAL_CONFIG_FILE: &str = ".skil.toml";

/// Persistent configuration for installed sources and skills.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SkilConfig {
    #[serde(rename = "source")]
    pub sources: BTreeMap<String, SkilSource>,
}

/// A source entry tracked in config.toml.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkilSource {
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
pub fn read_config(path: &Path) -> Result<SkilConfig> {
    if !path.exists() {
        return Ok(SkilConfig::default());
    }
    let content = std::fs::read_to_string(path)?;
    let config: SkilConfig =
        toml::from_str(&content).map_err(|err| SkilError::Message(err.to_string()))?;
    Ok(config)
}

/// Writes config to disk, creating parent directories as needed.
pub fn write_config(path: &Path, config: &SkilConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content =
        toml::to_string_pretty(config).map_err(|err| SkilError::Message(err.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Updates a config entry with skills and optional revision.
pub fn update_config(
    path: &Path,
    source_key: &str,
    source: SkilSource,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_config_returns_default_when_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("missing.toml");

        let config = read_config(&missing).expect("read");
        assert!(config.sources.is_empty());
    }

    #[test]
    fn write_and_read_config_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");

        let mut config = SkilConfig::default();
        config.sources.insert(
            "repo".to_string(),
            SkilSource {
                source_type: "github".to_string(),
                branch: Some("main".to_string()),
                subpath: Some("skills".to_string()),
                revision: Some("abc123".to_string()),
                skills: vec!["one".to_string()],
            },
        );

        write_config(&path, &config).expect("write");
        let loaded = read_config(&path).expect("read");

        let source = loaded.sources.get("repo").expect("repo source");
        assert_eq!(source.source_type, "github");
        assert_eq!(source.branch.as_deref(), Some("main"));
        assert_eq!(source.subpath.as_deref(), Some("skills"));
        assert_eq!(source.revision.as_deref(), Some("abc123"));
        assert_eq!(source.skills, vec!["one"]);
    }

    #[test]
    fn update_config_merges_skills_and_preserves_existing_revision() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let source_key = "https://github.com/example/repo.git";
        let source = SkilSource {
            source_type: "github".to_string(),
            branch: Some("main".to_string()),
            subpath: None,
            revision: Some("rev-1".to_string()),
            skills: vec!["alpha".to_string()],
        };

        update_config(
            &path,
            source_key,
            source.clone(),
            &[String::from("beta"), String::from("alpha")],
            Some("rev-2".to_string()),
        )
        .expect("first update");

        update_config(&path, source_key, source, &[String::from("gamma")], None)
            .expect("second update");

        let loaded = read_config(&path).expect("read");
        let entry = loaded.sources.get(source_key).expect("source entry");

        assert_eq!(entry.skills, vec!["alpha", "beta", "gamma"]);
        assert_eq!(entry.revision.as_deref(), Some("rev-2"));
        assert_eq!(entry.branch.as_deref(), Some("main"));
    }

    #[test]
    fn local_config_location_uses_current_directory() {
        let cwd = std::env::current_dir().expect("cwd");
        let location = config_location(false).expect("location");
        assert!(!location.is_global);
        assert_eq!(location.path, cwd.join(".skil.toml"));
    }
}
