use std::path::PathBuf;

use crate::error::{Result, SkilError};

/// Source metadata used for installs and updates.
#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub source_id: String,
    pub source_url: String,
    pub github_owner_repo: Option<String>,
    pub github_branch: Option<String>,
}

/// A parsed source, either local or git-based.
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

/// Parses a user-provided source string into a concrete source.
pub fn parse_source(source: &str) -> Result<Source> {
    if is_local_path(source) {
        let source_path = PathBuf::from(source);
        if !source_path.exists() {
            return Err(SkilError::Message(format!(
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

/// Heuristic for URL-like sources (http/ssh git).
fn looks_like_url(source: &str) -> bool {
    source.contains("://") || source.starts_with("git@")
}

/// Returns true if the input is clearly a local filesystem path.
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

/// Parses GitHub-style owner/repo and optional subpath.
fn parse_owner_repo(source: &str) -> Result<Source> {
    let parts: Vec<&str> = source.split('/').filter(|s| !s.is_empty()).collect();
    if parts.len() < 2 {
        return Err(SkilError::Message(
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
            source_url: url,
            github_owner_repo: Some(format!("{}/{}", owner, repo)),
            github_branch: None,
        },
    })
}

/// Parses supported hosted git URLs into a source.
fn parse_git_url(source: &str) -> Result<Source> {
    if let Some((url, subpath, owner_repo, branch)) = parse_hosted_git_url(source) {
        return Ok(Source::Git {
            url: url.clone(),
            subpath,
            info: SourceInfo {
                source_id: owner_repo.clone().unwrap_or_else(|| url.clone()),
                source_url: url,
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
            source_url: source.to_string(),
            github_owner_repo: parse_github_owner_repo(source),
            github_branch: None,
        },
    })
}

/// Parsed GitHub URL tuple: repo URL, subpath, owner/repo, branch.
type ParsedGithubTreeUrl = (String, Option<PathBuf>, Option<String>, Option<String>);

/// Parses GitHub URLs with optional tree/blob paths.
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

/// Extracts owner/repo for GitHub URLs when possible.
fn parse_github_owner_repo(source: &str) -> Option<String> {
    if let Some((_, _, owner_repo, _)) = parse_github_tree_url(source) {
        return owner_repo;
    }

    None
}

/// Parsed hosted git tuple: repo URL, subpath, owner/repo, branch.
type ParsedHostedGitUrl = (String, Option<PathBuf>, Option<String>, Option<String>);

/// Parses supported hosted git providers (GitHub, GitLab, Codeberg).
pub fn parse_hosted_git_url(source: &str) -> Option<ParsedHostedGitUrl> {
    if let Some((url, subpath, owner_repo, branch)) = parse_github_tree_url(source) {
        return Some((url, subpath, owner_repo, branch));
    }

    if let Some((url, subpath, owner_repo, branch)) = parse_gitlab_tree_url(source) {
        return Some((url, subpath, owner_repo, branch));
    }

    if let Some((url, subpath, owner_repo, branch)) = parse_codeberg_tree_url(source) {
        return Some((url, subpath, owner_repo, branch));
    }

    None
}

/// Parses GitLab URLs with optional tree paths.
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

/// Parses Codeberg URLs with optional branch paths.
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parses_github_tree_url() {
        let url = "https://github.com/vercel-labs/agent-skills/tree/main/skills/web-design";
        let (repo_url, subpath, owner_repo, branch) = parse_github_tree_url(url).expect("parsed");
        assert_eq!(repo_url, "https://github.com/vercel-labs/agent-skills.git");
        assert_eq!(
            subpath.expect("subpath").to_string_lossy(),
            "skills/web-design"
        );
        assert_eq!(owner_repo.expect("owner/repo"), "vercel-labs/agent-skills");
        assert_eq!(branch.expect("branch"), "main");
    }

    #[test]
    fn parses_git_ssh_url() {
        let url = "git@github.com:vercel-labs/agent-skills.git";
        let (repo_url, subpath, owner_repo, branch) = parse_github_tree_url(url).expect("parsed");
        assert_eq!(repo_url, url);
        assert!(subpath.is_none());
        assert_eq!(owner_repo.expect("owner/repo"), "vercel-labs/agent-skills");
        assert!(branch.is_none());
    }

    #[test]
    fn parses_gitlab_tree_url() {
        let url = "https://gitlab.com/example/skills/-/tree/main/skills/web-design";
        let (repo_url, subpath, owner_repo, branch) = parse_hosted_git_url(url).expect("parsed");
        assert_eq!(repo_url, "https://gitlab.com/example/skills.git");
        assert_eq!(
            subpath.expect("subpath").to_string_lossy(),
            "skills/web-design"
        );
        assert_eq!(owner_repo.expect("owner/repo"), "example/skills");
        assert_eq!(branch.expect("branch"), "main");
    }

    #[test]
    fn parses_codeberg_tree_url() {
        let url = "https://codeberg.org/example/skills/src/branch/main/skills/web-design";
        let (repo_url, subpath, owner_repo, branch) = parse_hosted_git_url(url).expect("parsed");
        assert_eq!(repo_url, "https://codeberg.org/example/skills.git");
        assert_eq!(
            subpath.expect("subpath").to_string_lossy(),
            "skills/web-design"
        );
        assert_eq!(owner_repo.expect("owner/repo"), "example/skills");
        assert_eq!(branch.expect("branch"), "main");
    }

    #[test]
    fn parses_owner_repo_source_with_subpath() {
        let parsed = parse_source("vercel-labs/agent-skills/skills/web-design").expect("parsed");
        let Source::Git { url, subpath, info } = parsed else {
            panic!("expected git source");
        };

        assert_eq!(url, "https://github.com/vercel-labs/agent-skills.git");
        assert_eq!(
            subpath.expect("subpath"),
            PathBuf::from("skills/web-design")
        );
        assert_eq!(
            info.github_owner_repo.expect("owner/repo"),
            "vercel-labs/agent-skills"
        );
    }

    #[test]
    fn parses_existing_local_source_path() {
        let dir = tempdir().expect("tempdir");
        let parsed = parse_source(dir.path().to_str().expect("utf8 path")).expect("parsed");

        let Source::Local { path } = parsed else {
            panic!("expected local source");
        };
        assert_eq!(
            path,
            std::fs::canonicalize(dir.path()).expect("canonical path")
        );
    }

    #[test]
    fn rejects_invalid_short_source() {
        let err = parse_source("invalid").expect_err("invalid source should fail");
        assert!(err.to_string().contains("Invalid source"));
    }

    #[test]
    fn parses_unknown_url_as_generic_git_source() {
        let parsed = parse_source("https://example.com/custom/repo.git").expect("parsed");
        let Source::Git { url, subpath, info } = parsed else {
            panic!("expected git source");
        };

        assert_eq!(url, "https://example.com/custom/repo.git");
        assert!(subpath.is_none());
        assert_eq!(info.source_id, "https://example.com/custom/repo.git");
    }

    #[test]
    fn parses_github_blob_url_with_trailing_slash() {
        let url = "https://github.com/vercel-labs/agent-skills/blob/main/skills/web-design/";
        let (repo_url, subpath, owner_repo, branch) = parse_github_tree_url(url).expect("parsed");
        assert_eq!(repo_url, "https://github.com/vercel-labs/agent-skills.git");
        assert_eq!(
            subpath.expect("subpath").to_string_lossy(),
            "skills/web-design"
        );
        assert_eq!(owner_repo.expect("owner/repo"), "vercel-labs/agent-skills");
        assert_eq!(branch.expect("branch"), "main");
    }

    #[test]
    fn parses_plain_github_repo_url_without_branch_or_subpath() {
        let url = "https://github.com/vercel-labs/agent-skills";
        let (repo_url, subpath, owner_repo, branch) = parse_github_tree_url(url).expect("parsed");
        assert_eq!(repo_url, "https://github.com/vercel-labs/agent-skills.git");
        assert!(subpath.is_none());
        assert_eq!(owner_repo.expect("owner/repo"), "vercel-labs/agent-skills");
        assert!(branch.is_none());
    }

    #[test]
    fn parses_gitlab_and_codeberg_ssh_urls() {
        let gitlab = "git@gitlab.com:example/repo.git";
        let (repo_url, subpath, owner_repo, branch) =
            parse_hosted_git_url(gitlab).expect("gitlab parsed");
        assert_eq!(repo_url, gitlab);
        assert!(subpath.is_none());
        assert_eq!(owner_repo.expect("owner/repo"), "example/repo");
        assert!(branch.is_none());

        let codeberg = "git@codeberg.org:example/repo.git";
        let (repo_url, subpath, owner_repo, branch) =
            parse_hosted_git_url(codeberg).expect("codeberg parsed");
        assert_eq!(repo_url, codeberg);
        assert!(subpath.is_none());
        assert_eq!(owner_repo.expect("owner/repo"), "example/repo");
        assert!(branch.is_none());
    }

    #[test]
    fn hosted_git_url_returns_none_for_non_supported_hosts() {
        assert!(parse_hosted_git_url("https://example.com/org/repo.git").is_none());
    }

    #[test]
    fn parse_source_rejects_missing_explicit_local_path() {
        let missing_path = "./__skil_missing_path_for_test__";
        let err = parse_source(missing_path).expect_err("missing explicit local path should fail");
        assert!(err.to_string().contains("Local path does not exist"));
    }
}
