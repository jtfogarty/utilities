//! Crate-level errors (`thiserror`). Operational code also uses [`anyhow`] at the boundary.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SaltMasterAgentError {
    #[error("configuration error: {0}")]
    Config(Box<figment::Error>),
}

impl From<figment::Error> for SaltMasterAgentError {
    fn from(value: figment::Error) -> Self {
        Self::Config(Box::new(value))
    }
}
