//! Configuration errors.

use std::fmt;
use std::path::PathBuf;

use thiserror::Error;

/// An out-of-range or inconsistent value, with its path (`regulator.budget_fraction`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigIssue {
    pub path: String,
    pub message: String,
}

impl ConfigIssue {
    pub(crate) fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ConfigIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

fn join(issues: &[ConfigIssue]) -> String {
    issues
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not access {}: {source}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("could not write the TOML: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("invalid configuration: {}", join(.0))]
    Invalid(Vec<ConfigIssue>),
    #[error("environment variable {var}: {message}")]
    Env { var: &'static str, message: String },
}
