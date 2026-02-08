use skil::source::{parse_github_tree_url, parse_hosted_git_url};

#[test]
fn parses_github_tree_url() {
    let url = "https://github.com/vercel-labs/agent-skills/tree/main/skills/web-design";
    let (repo_url, subpath, owner_repo, branch) = parse_github_tree_url(url).expect("parsed");
    assert_eq!(repo_url, "https://github.com/vercel-labs/agent-skills.git");
    assert_eq!(subpath.unwrap().to_string_lossy(), "skills/web-design");
    assert_eq!(owner_repo.unwrap(), "vercel-labs/agent-skills");
    assert_eq!(branch.unwrap(), "main");
}

#[test]
fn parses_git_ssh_url() {
    let url = "git@github.com:vercel-labs/agent-skills.git";
    let (repo_url, subpath, owner_repo, branch) = parse_github_tree_url(url).expect("parsed");
    assert_eq!(repo_url, url);
    assert!(subpath.is_none());
    assert_eq!(owner_repo.unwrap(), "vercel-labs/agent-skills");
    assert!(branch.is_none());
}

#[test]
fn parses_gitlab_tree_url() {
    let url = "https://gitlab.com/example/skills/-/tree/main/skills/web-design";
    let (repo_url, subpath, owner_repo, branch, source_type) =
        parse_hosted_git_url(url).expect("parsed");
    assert_eq!(repo_url, "https://gitlab.com/example/skills.git");
    assert_eq!(subpath.unwrap().to_string_lossy(), "skills/web-design");
    assert_eq!(owner_repo.unwrap(), "example/skills");
    assert_eq!(branch.unwrap(), "main");
    assert_eq!(source_type, "gitlab");
}

#[test]
fn parses_codeberg_tree_url() {
    let url = "https://codeberg.org/example/skills/src/branch/main/skills/web-design";
    let (repo_url, subpath, owner_repo, branch, source_type) =
        parse_hosted_git_url(url).expect("parsed");
    assert_eq!(repo_url, "https://codeberg.org/example/skills.git");
    assert_eq!(subpath.unwrap().to_string_lossy(), "skills/web-design");
    assert_eq!(owner_repo.unwrap(), "example/skills");
    assert_eq!(branch.unwrap(), "main");
    assert_eq!(source_type, "codeberg");
}
