use std::path::PathBuf;

use crate::error::{Result, SkillzError};

#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub source_id: String,
    pub source_type: String,
    pub source_url: String,
    pub skill_path: Option<String>,
    pub github_owner_repo: Option<String>,
    pub github_branch: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Source {
    Local {
        path: PathBuf,
    },
    Git {
        url: String,
        subpath: Option<PathBuf>,
        info: SourceInfo,
    },
}

pub fn parse_source(source: &str) -> Result<Source> {
    if is_local_path(source) {
        let source_path = PathBuf::from(source);
        if !source_path.exists() {
            return Err(SkillzError::Message(format!(
                "Local path does not exist: {}",
                source
            )));
        }
        let path = std::fs::canonicalize(source_path)?;
        return Ok(Source::Local { path });
    }

    let source_path = PathBuf::from(source);
    if source_path.exists() {
        let path = std::fs::canonicalize(source_path)?;
        return Ok(Source::Local { path });
    }

    if looks_like_url(source) {
        return parse_git_url(source);
    }

    parse_owner_repo(source)
}

fn looks_like_url(source: &str) -> bool {
    source.contains("://") || source.starts_with("git@")
}

fn is_local_path(source: &str) -> bool {
    source.starts_with("./")
        || source.starts_with("../")
        || source == "."
        || source == ".."
        || PathBuf::from(source).is_absolute()
        || source
            .chars()
            .nth(1)
            .map(|c| {
                c == ':'
                    && source
                        .chars()
                        .nth(2)
                        .map(|s| s == '/' || s == '\\')
                        .unwrap_or(false)
            })
            .unwrap_or(false)
}

fn parse_owner_repo(source: &str) -> Result<Source> {
    let parts: Vec<&str> = source.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() < 2 {
        return Err(SkillzError::Message(
            "Invalid source: expected owner/repo or URL".to_string(),
        ));
    }
    let owner = parts[0];
    let repo = parts[1];
    let subpath = if parts.len() > 2 {
        Some(PathBuf::from(parts[2..].join("/")))
    } else {
        None
    };

    let url = format!("https://github.com/{}/{}.git", owner, repo);
    let source_id = format!("{}/{}", owner, repo);

    Ok(Source::Git {
        url: url.clone(),
        subpath,
        info: SourceInfo {
            source_id,
            source_type: "github".to_string(),
            source_url: url,
            skill_path: None,
            github_owner_repo: Some(format!("{}/{}", owner, repo)),
            github_branch: None,
        },
    })
}

fn parse_git_url(source: &str) -> Result<Source> {
    if let Some((url, subpath, owner_repo, branch, source_type)) = parse_hosted_git_url(source) {
        return Ok(Source::Git {
            url: url.clone(),
            subpath,
            info: SourceInfo {
                source_id: owner_repo.clone().unwrap_or_else(|| url.clone()),
                source_type,
                source_url: url,
                skill_path: None,
                github_owner_repo: owner_repo,
                github_branch: branch,
            },
        });
    }

    Ok(Source::Git {
        url: source.to_string(),
        subpath: None,
        info: SourceInfo {
            source_id: source.to_string(),
            source_type: "git".to_string(),
            source_url: source.to_string(),
            skill_path: None,
            github_owner_repo: parse_github_owner_repo(source),
            github_branch: None,
        },
    })
}

type ParsedGithubTreeUrl = (String, Option<PathBuf>, Option<String>, Option<String>);

pub fn parse_github_tree_url(source: &str) -> Option<ParsedGithubTreeUrl> {
    let source = source.trim_end_matches('/');

    let https_prefix = "https://github.com/";
    let http_prefix = "http://github.com/";
    let mut rest = None;
    if let Some(stripped) = source.strip_prefix(https_prefix) {
        rest = Some(stripped);
    } else if let Some(stripped) = source.strip_prefix(http_prefix) {
        rest = Some(stripped);
    }

    if let Some(rest) = rest {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            let owner = parts[0];
            let repo = parts[1].trim_end_matches(".git");
            let owner_repo = format!("{}/{}", owner, repo);
            let repo_url = format!("https://github.com/{}/{}.git", owner, repo);

            if parts.len() >= 4 && (parts[2] == "tree" || parts[2] == "blob") {
                let branch = parts[3].to_string();
                let subpath = parts[4..].join("/");
                let subpath = if subpath.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(subpath))
                };
                return Some((repo_url, subpath, Some(owner_repo), Some(branch)));
            }

            return Some((repo_url, None, Some(owner_repo), None));
        }
    }

    if source.starts_with("git@github.com:") {
        let rest = source.trim_start_matches("git@github.com:");
        let rest = rest.trim_end_matches(".git");
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            let owner_repo = format!("{}/{}", parts[0], parts[1]);
            return Some((source.to_string(), None, Some(owner_repo), None));
        }
    }

    None
}

fn parse_github_owner_repo(source: &str) -> Option<String> {
    if let Some((_, _, owner_repo, _)) = parse_github_tree_url(source) {
        return owner_repo;
    }

    None
}

type ParsedHostedGitUrl = (String, Option<PathBuf>, Option<String>, Option<String>, String);

pub fn parse_hosted_git_url(source: &str) -> Option<ParsedHostedGitUrl> {
    if let Some((url, subpath, owner_repo, branch)) = parse_github_tree_url(source) {
        return Some((
            url,
            subpath,
            owner_repo,
            branch,
            "github".to_string(),
        ));
    }

    if let Some((url, subpath, owner_repo, branch)) = parse_gitlab_tree_url(source) {
        return Some((
            url,
            subpath,
            owner_repo,
            branch,
            "gitlab".to_string(),
        ));
    }

    if let Some((url, subpath, owner_repo, branch)) = parse_codeberg_tree_url(source) {
        return Some((
            url,
            subpath,
            owner_repo,
            branch,
            "codeberg".to_string(),
        ));
    }

    None
}

fn parse_gitlab_tree_url(source: &str) -> Option<ParsedGithubTreeUrl> {
    let source = source.trim_end_matches('/');

    let https_prefix = "https://gitlab.com/";
    let http_prefix = "http://gitlab.com/";
    let mut rest = None;
    if let Some(stripped) = source.strip_prefix(https_prefix) {
        rest = Some(stripped);
    } else if let Some(stripped) = source.strip_prefix(http_prefix) {
        rest = Some(stripped);
    }

    if let Some(rest) = rest {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            let owner = parts[0];
            let repo = parts[1].trim_end_matches(".git");
            let owner_repo = format!("{}/{}", owner, repo);
            let repo_url = format!("https://gitlab.com/{}/{}.git", owner, repo);

            if parts.len() >= 5 && parts[2] == "-" && parts[3] == "tree" {
                let branch = parts[4].to_string();
                let subpath = parts[5..].join("/");
                let subpath = if subpath.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(subpath))
                };
                return Some((repo_url, subpath, Some(owner_repo), Some(branch)));
            }

            return Some((repo_url, None, Some(owner_repo), None));
        }
    }

    if source.starts_with("git@gitlab.com:") {
        let rest = source.trim_start_matches("git@gitlab.com:");
        let rest = rest.trim_end_matches(".git");
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            let owner_repo = format!("{}/{}", parts[0], parts[1]);
            return Some((source.to_string(), None, Some(owner_repo), None));
        }
    }

    None
}

fn parse_codeberg_tree_url(source: &str) -> Option<ParsedGithubTreeUrl> {
    let source = source.trim_end_matches('/');

    let https_prefix = "https://codeberg.org/";
    let http_prefix = "http://codeberg.org/";
    let mut rest = None;
    if let Some(stripped) = source.strip_prefix(https_prefix) {
        rest = Some(stripped);
    } else if let Some(stripped) = source.strip_prefix(http_prefix) {
        rest = Some(stripped);
    }

    if let Some(rest) = rest {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            let owner = parts[0];
            let repo = parts[1].trim_end_matches(".git");
            let owner_repo = format!("{}/{}", owner, repo);
            let repo_url = format!("https://codeberg.org/{}/{}.git", owner, repo);

            if parts.len() >= 5 && parts[2] == "src" && parts[3] == "branch" {
                let branch = parts[4].to_string();
                let subpath = parts[5..].join("/");
                let subpath = if subpath.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(subpath))
                };
                return Some((repo_url, subpath, Some(owner_repo), Some(branch)));
            }

            return Some((repo_url, None, Some(owner_repo), None));
        }
    }

    if source.starts_with("git@codeberg.org:") {
        let rest = source.trim_start_matches("git@codeberg.org:");
        let rest = rest.trim_end_matches(".git");
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            let owner_repo = format!("{}/{}", parts[0], parts[1]);
            return Some((source.to_string(), None, Some(owner_repo), None));
        }
    }

    None
}
