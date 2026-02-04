use std::path::Path;
use std::sync::atomic::AtomicBool;

use crate::error::Result;

/// Clones a git repository URL into the destination directory.
pub fn clone_repo(url: &str, dest: &Path) -> Result<()> {
    let mut prepare = gix::prepare_clone(url, dest)?;
    let (mut checkout, _) =
        prepare.fetch_then_checkout(gix::progress::Discard, &AtomicBool::new(false))?;
    let _ = checkout.main_worktree(gix::progress::Discard, &AtomicBool::new(false))?;
    Ok(())
}
