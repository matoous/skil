use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkillzError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    GixClone(#[from] gix::clone::Error),
    #[error(transparent)]
    GixFetch(#[from] gix::clone::fetch::Error),
    #[error(transparent)]
    GixCheckout(#[from] gix::clone::checkout::main_worktree::Error),
    #[error(transparent)]
    Walkdir(#[from] walkdir::Error),
}

pub type Result<T> = std::result::Result<T, SkillzError>;
