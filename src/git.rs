use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::process::Command;

use crate::error::{Result, SkillzError};

/// Clones a git repository URL into the destination directory.
pub fn clone_repo(url: &str, dest: &Path) -> Result<()> {
    let mut prepare = gix::prepare_clone(url, dest)?;
    let (mut checkout, _) =
        prepare.fetch_then_checkout(gix::progress::Discard, &AtomicBool::new(false))?;
    let _ = checkout.main_worktree(gix::progress::Discard, &AtomicBool::new(false))?;
    Ok(())
}

/// Returns the HEAD revision for a cloned repository.
pub fn head_revision(repo_path: &Path) -> Result<String> {
    let repo = gix::open(repo_path).map_err(|err| SkillzError::Message(err.to_string()))?;
    let head = repo
        .head_id()
        .map_err(|err| SkillzError::Message(err.to_string()))?;
    Ok(head.to_string())
}

/// Returns the latest revision for a remote URL and optional branch.
pub fn remote_revision(url: &str, branch: Option<&str>) -> Result<String> {
    let target = branch.unwrap_or("HEAD");
    let output = Command::new("git")
        .args(["ls-remote", url, target])
        .output()?;
    if !output.status.success() {
        return Err(crate::error::SkillzError::Message(
            "git ls-remote failed".to_string(),
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let rev = stdout.split_whitespace().next().unwrap_or("").to_string();
    if rev.is_empty() {
        return Err(crate::error::SkillzError::Message(
            "Could not resolve remote revision".to_string(),
        ));
    }
    Ok(rev)
}
