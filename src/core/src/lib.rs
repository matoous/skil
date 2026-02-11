#![allow(clippy::result_large_err)]

pub mod agent;
pub mod config;
pub mod error;
pub mod git;
pub mod install;
pub mod skills;
pub mod source;

pub use error::{Result, SkilError};
