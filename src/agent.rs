use std::path::PathBuf;

/// Configuration for a supported agent and its skills directories.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub name: &'static str,
    pub display_name: &'static str,
    pub skills_dir: String,
    pub global_skills_dir: String,
}

/// Returns the full list of known agents with resolved paths.
pub fn agent_configs() -> Vec<AgentConfig> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".config"));
    let codex_home = std::env::var("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".codex"));
    let claude_home = std::env::var("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".claude"));

    vec![
        AgentConfig {
            name: "codex",
            display_name: "Codex",
            skills_dir: ".codex/skills".to_string(),
            global_skills_dir: codex_home.join("skills").to_string_lossy().to_string(),
        },
        AgentConfig {
            name: "claude-code",
            display_name: "Claude Code",
            skills_dir: ".claude/skills".to_string(),
            global_skills_dir: claude_home.join("skills").to_string_lossy().to_string(),
        },
        AgentConfig {
            name: "opencode",
            display_name: "OpenCode",
            skills_dir: ".opencode/skills".to_string(),
            global_skills_dir: config_home
                .join("opencode/skills")
                .to_string_lossy()
                .to_string(),
        },
        AgentConfig {
            name: "cursor",
            display_name: "Cursor",
            skills_dir: ".cursor/skills".to_string(),
            global_skills_dir: home.join(".cursor/skills").to_string_lossy().to_string(),
        },
        AgentConfig {
            name: "continue",
            display_name: "Continue",
            skills_dir: ".continue/skills".to_string(),
            global_skills_dir: home.join(".continue/skills").to_string_lossy().to_string(),
        },
        AgentConfig {
            name: "github-copilot",
            display_name: "GitHub Copilot",
            skills_dir: ".github/skills".to_string(),
            global_skills_dir: home.join(".copilot/skills").to_string_lossy().to_string(),
        },
        AgentConfig {
            name: "goose",
            display_name: "Goose",
            skills_dir: ".goose/skills".to_string(),
            global_skills_dir: config_home
                .join("goose/skills")
                .to_string_lossy()
                .to_string(),
        },
        AgentConfig {
            name: "junie",
            display_name: "Junie",
            skills_dir: ".junie/skills".to_string(),
            global_skills_dir: home.join(".junie/skills").to_string_lossy().to_string(),
        },
        AgentConfig {
            name: "windsurf",
            display_name: "Windsurf",
            skills_dir: ".windsurf/skills".to_string(),
            global_skills_dir: home.join(".windsurf/skills").to_string_lossy().to_string(),
        },
    ]
}

/// Resolves requested agent names to configs, with defaults if empty.
pub fn resolve_agents(requested: &[String]) -> Vec<AgentConfig> {
    let all_agents = agent_configs();

    if requested.is_empty() {
        return detect_default_agents(&all_agents);
    }

    if requested.len() == 1 && requested[0] == "*" {
        return all_agents;
    }

    let mut selected = Vec::new();
    for name in requested {
        if let Some(agent) = all_agents.iter().find(|a| a.name == name) {
            selected.push(agent.clone());
        }
    }

    selected
}

/// Detects a reasonable default set of agents based on local config folders.
fn detect_default_agents(all_agents: &[AgentConfig]) -> Vec<AgentConfig> {
    let mut detected = Vec::new();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".config"));
    let codex_home = std::env::var("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".codex"));
    let claude_home = std::env::var("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home.join(".claude"));

    let default_candidates = vec![
        ("codex", codex_home),
        ("claude-code", claude_home),
        ("opencode", config_home.join("opencode")),
    ];

    for (name, path) in default_candidates {
        if path.exists() && let Some(agent) = all_agents.iter().find(|a| a.name == name) {
            detected.push(agent.clone());
        }
    }

    if detected.is_empty()
        && let Some(agent) = all_agents.iter().find(|a| a.name == "codex")
    {
        detected.push(agent.clone());
    }

    detected
}
