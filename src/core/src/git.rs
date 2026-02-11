use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;

use crate::error::{Result, SkilError};

/// Clones a git repository URL into the destination directory.
pub fn clone_repo(url: &str, dest: &Path) -> Result<()> {
    let mut prepare = gix::prepare_clone(url, dest)?;
    let (mut checkout, _) =
        prepare.fetch_then_checkout(gix::progress::Discard, &AtomicBool::new(false))?;
    let _ = checkout.main_worktree(gix::progress::Discard, &AtomicBool::new(false))?;
    Ok(())
}

/// Checks out a specific revision in a cloned repository.
pub fn checkout_revision(repo_path: &Path, revision: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo_path)
        .args(["checkout", "--detach", revision])
        .output()?;
    if !output.status.success() {
        return Err(crate::error::SkilError::Message(format!(
            "git checkout failed for revision {}",
            revision
        )));
    }
    Ok(())
}

/// Returns the latest remote tag name if any tags are available.
pub fn latest_tag(url: &str) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["ls-remote", "--tags", "--refs", "--sort=-v:refname", url])
        .output()?;
    if !output.status.success() {
        return Err(crate::error::SkilError::Message(
            "git ls-remote --tags failed".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(first) = stdout.lines().next() else {
        return Ok(None);
    };
    let mut parts = first.split_whitespace();
    let _hash = parts.next();
    let reference = parts.next().unwrap_or("");
    let Some(tag) = reference.strip_prefix("refs/tags/") else {
        return Ok(None);
    };
    Ok(Some(tag.to_string()))
}

/// Returns the HEAD revision for a cloned repository.
pub fn head_revision(repo_path: &Path) -> Result<String> {
    let repo = gix::open(repo_path).map_err(|err| SkilError::Message(err.to_string()))?;
    let head = repo
        .head_id()
        .map_err(|err| SkilError::Message(err.to_string()))?;
    Ok(head.to_string())
}

/// Returns the latest revision for a remote URL and optional branch.
pub fn remote_revision(url: &str, branch: Option<&str>) -> Result<String> {
    let target = branch.unwrap_or("HEAD");
    let output = Command::new("git")
        .args(["ls-remote", url, target])
        .output()?;
    if !output.status.success() {
        return Err(crate::error::SkilError::Message(
            "git ls-remote failed".to_string(),
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let rev = stdout.split_whitespace().next().unwrap_or("").to_string();
    if rev.is_empty() {
        return Err(crate::error::SkilError::Message(
            "Could not resolve remote revision".to_string(),
        ));
    }
    Ok(rev)
}
