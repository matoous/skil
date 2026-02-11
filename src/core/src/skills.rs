use std::path::{Path, PathBuf};

use serde::Deserialize;
use walkdir::WalkDir;

use crate::error::Result;

/// Parsed skill metadata and file location.
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub raw_content: String,
}

/// Frontmatter structure for SKILL.md.
#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Discovers skills in a repository or directory tree.
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

    if has_skill_md(&search_root)
        && let Some(skill) = parse_skill_md(&search_root.join("SKILL.md"))?
    {
        seen.insert(skill.name.clone());
        skills.push(skill);
        if !full_depth {
            return Ok(skills);
        }
    }

    let priority_dirs = priority_skill_dirs(&search_root);
    for dir in priority_dirs {
        if !dir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.path().is_dir()
                && has_skill_md(&entry.path())
                && let Some(skill) = parse_skill_md(&entry.path().join("SKILL.md"))?
                && seen.insert(skill.name.clone())
            {
                skills.push(skill);
            }
        }
    }

    if skills.is_empty() {
        for entry in WalkDir::new(&search_root)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == "SKILL.md"
                && let Some(skill) = parse_skill_md(entry.path())?
                && seen.insert(skill.name.clone())
            {
                skills.push(skill);
            }
        }
    }

    Ok(skills)
}

/// Filters skills by requested names (case-insensitive).
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

/// Returns a prioritized list of directories to scan for skills.
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

/// Checks if a directory contains a SKILL.md file.
fn has_skill_md(dir: &Path) -> bool {
    dir.join("SKILL.md").is_file()
}

/// Parses a SKILL.md file into a Skill if valid.
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

/// Parses YAML frontmatter from SKILL.md content.
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parses_frontmatter() {
        let content = "---\nname: Test Skill\ndescription: Does stuff\n---\n\n# Test";
        let frontmatter = parse_frontmatter(content).expect("ok").expect("some");
        assert_eq!(frontmatter.name.expect("name"), "Test Skill");
        assert_eq!(frontmatter.description.expect("description"), "Does stuff");
    }

    #[test]
    fn ignores_missing_frontmatter() {
        let content = "# No frontmatter";
        let frontmatter = parse_frontmatter(content).expect("ok");
        assert!(frontmatter.is_none());
    }

    #[test]
    fn rejects_invalid_frontmatter_yaml() {
        let content = "---\nname: [\n---\n# Broken";
        let err = parse_frontmatter(content).expect_err("invalid yaml should fail");
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn selects_skills_case_insensitively() {
        let skills = vec![
            Skill {
                name: "Web-Design".to_string(),
                description: "One".to_string(),
                path: Path::new("one").to_path_buf(),
                raw_content: String::new(),
            },
            Skill {
                name: "go-style".to_string(),
                description: "Two".to_string(),
                path: Path::new("two").to_path_buf(),
                raw_content: String::new(),
            },
        ];

        let selected = select_skills(&skills, &[String::from("WEB-DESIGN")]);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].name, "Web-Design");
    }

    #[test]
    fn discovers_skills_in_priority_locations() {
        let dir = tempdir().expect("tempdir");
        let skill_dir = dir.path().join("skills").join("my-skill");
        std::fs::create_dir_all(&skill_dir).expect("create skill dir");
        std::fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: MySkill\ndescription: Desc\n---\n# Title",
        )
        .expect("write skill");

        let discovered = discover_skills(dir.path(), None, true).expect("discover");
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].name, "MySkill");
        assert_eq!(discovered[0].description, "Desc");
    }

    #[test]
    fn select_skills_wildcard_returns_all() {
        let skills = vec![
            Skill {
                name: "a".to_string(),
                description: "A".to_string(),
                path: Path::new("a").to_path_buf(),
                raw_content: String::new(),
            },
            Skill {
                name: "b".to_string(),
                description: "B".to_string(),
                path: Path::new("b").to_path_buf(),
                raw_content: String::new(),
            },
        ];

        let selected = select_skills(&skills, &[String::from("*")]);
        assert_eq!(selected.len(), 2);
    }

    #[test]
    fn parse_skill_md_requires_name_and_description() {
        let dir = tempdir().expect("tempdir");
        let missing_name = dir.path().join("missing-name.md");
        std::fs::write(
            &missing_name,
            "---\ndescription: Desc only\n---\n# Missing name",
        )
        .expect("write");
        assert!(parse_skill_md(&missing_name).expect("parsed").is_none());

        let missing_description = dir.path().join("missing-description.md");
        std::fs::write(
            &missing_description,
            "---\nname: Name only\n---\n# Missing description",
        )
        .expect("write");
        assert!(
            parse_skill_md(&missing_description)
                .expect("parsed")
                .is_none()
        );
    }

    #[test]
    fn discover_skills_deduplicates_by_name() {
        let dir = tempdir().expect("tempdir");
        let root_skill = dir.path().join("root-skill");
        let nested_skill = dir.path().join("skills").join("nested-skill");
        std::fs::create_dir_all(&root_skill).expect("create root");
        std::fs::create_dir_all(&nested_skill).expect("create nested");

        let content = "---\nname: SameName\ndescription: Desc\n---\n# Title";
        std::fs::write(root_skill.join("SKILL.md"), content).expect("write root");
        std::fs::write(nested_skill.join("SKILL.md"), content).expect("write nested");

        let discovered = discover_skills(dir.path(), None, true).expect("discover");
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].name, "SameName");
    }

    #[test]
    fn discover_skills_stops_early_when_root_has_skill_and_full_depth_is_false() {
        let dir = tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("SKILL.md"),
            "---\nname: RootSkill\ndescription: Root\n---\n# Root",
        )
        .expect("write root skill");

        let nested_skill = dir.path().join("skills").join("nested-skill");
        std::fs::create_dir_all(&nested_skill).expect("create nested");
        std::fs::write(
            nested_skill.join("SKILL.md"),
            "---\nname: NestedSkill\ndescription: Nested\n---\n# Nested",
        )
        .expect("write nested skill");

        let discovered = discover_skills(dir.path(), None, false).expect("discover");
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].name, "RootSkill");
    }
}
