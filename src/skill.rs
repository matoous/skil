use std::path::{Path, PathBuf};

use serde::Deserialize;
use walkdir::WalkDir;

use crate::error::Result;

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub raw_content: String,
}

#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub fn discover_skills(
    base: &Path,
    subpath: Option<&Path>,
    full_depth: bool,
) -> Result<Vec<Skill>> {
    let search_root = subpath
        .map(|p| base.join(p))
        .unwrap_or_else(|| base.to_path_buf());

    let mut skills = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if has_skill_md(&search_root) {
        if let Some(skill) = parse_skill_md(&search_root.join("SKILL.md"))? {
            seen.insert(skill.name.clone());
            skills.push(skill);
            if !full_depth {
                return Ok(skills);
            }
        }
    }

    let priority_dirs = priority_skill_dirs(&search_root);
    for dir in priority_dirs {
        if !dir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.path().is_dir() && has_skill_md(&entry.path()) {
                if let Some(skill) = parse_skill_md(&entry.path().join("SKILL.md"))? {
                    if seen.insert(skill.name.clone()) {
                        skills.push(skill);
                    }
                }
            }
        }
    }

    if skills.is_empty() {
        for entry in WalkDir::new(&search_root)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == "SKILL.md" {
                if let Some(skill) = parse_skill_md(entry.path())? {
                    if seen.insert(skill.name.clone()) {
                        skills.push(skill);
                    }
                }
            }
        }
    }

    Ok(skills)
}

pub fn select_skills(skills: &[Skill], requested: &[String]) -> Vec<Skill> {
    if requested.is_empty() || (requested.len() == 1 && requested[0] == "*") {
        return skills.to_vec();
    }

    let requested_lower: std::collections::HashSet<String> =
        requested.iter().map(|s| s.to_lowercase()).collect();
    let mut selected = Vec::new();

    for skill in skills {
        let name = skill.name.to_lowercase();
        if requested_lower.contains(&name) {
            selected.push(skill.clone());
        }
    }

    selected
}

fn priority_skill_dirs(base: &Path) -> Vec<PathBuf> {
    vec![
        base.to_path_buf(),
        base.join("skills"),
        base.join("skills/.curated"),
        base.join("skills/.experimental"),
        base.join("skills/.system"),
        base.join(".agent/skills"),
        base.join(".agents/skills"),
        base.join(".claude/skills"),
        base.join(".cline/skills"),
        base.join(".codebuddy/skills"),
        base.join(".codex/skills"),
        base.join(".commandcode/skills"),
        base.join(".continue/skills"),
        base.join(".cursor/skills"),
        base.join(".github/skills"),
        base.join(".goose/skills"),
        base.join(".junie/skills"),
        base.join(".kilocode/skills"),
        base.join(".kiro/skills"),
        base.join(".mux/skills"),
        base.join(".opencode/skills"),
        base.join(".openhands/skills"),
        base.join(".roo/skills"),
        base.join(".trae/skills"),
        base.join(".windsurf/skills"),
        base.join(".zencoder/skills"),
    ]
}

fn has_skill_md(dir: &Path) -> bool {
    dir.join("SKILL.md").is_file()
}

pub fn parse_skill_md(path: &Path) -> Result<Option<Skill>> {
    let content = std::fs::read_to_string(path)?;
    let frontmatter = parse_frontmatter(&content)?;
    let Some(frontmatter) = frontmatter else {
        return Ok(None);
    };

    let name = frontmatter.name.unwrap_or_default();
    let description = frontmatter.description.unwrap_or_default();
    if name.is_empty() || description.is_empty() {
        return Ok(None);
    }

    Ok(Some(Skill {
        name,
        description,
        path: path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        raw_content: content,
    }))
}

pub fn parse_frontmatter(content: &str) -> Result<Option<Frontmatter>> {
    let mut lines = content.lines();
    let first = lines.next().unwrap_or("");
    if first.trim() != "---" {
        return Ok(None);
    }

    let mut yaml = String::new();
    for line in &mut lines {
        if line.trim() == "---" {
            break;
        }
        yaml.push_str(line);
        yaml.push('\n');
    }

    if yaml.trim().is_empty() {
        return Ok(None);
    }

    let data: Frontmatter = serde_yaml::from_str(&yaml)?;
    Ok(Some(data))
}
