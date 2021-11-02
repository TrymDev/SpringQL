//! Error type.

pub mod foreign_info;
pub mod responsibility;

use thiserror::Error;

use crate::model::name::PumpName;

use self::{foreign_info::ForeignInfo, responsibility::SpringErrorResponsibility};

/// Result type
pub type Result<T> = std::result::Result<T, SpringError>;

/// Error type
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum SpringError {
    #[error("I/O error related to foreign system: {foreign_info:?}")]
    ForeignIo {
        foreign_info: ForeignInfo,
        source: anyhow::Error,
    },

    #[error("Timeout when getting an input from foreign source: {foreign_info:?}")]
    ForeignSourceTimeout {
        foreign_info: ForeignInfo,
        source: anyhow::Error,
    },

    #[error("Timeout when getting an input from a stream ({task_name})")]
    InputTimeout {
        task_name: String,
        source: anyhow::Error,
    },

    #[error("I/O error inside SpringQL-core")]
    SpringQlCoreIo(anyhow::Error),

    #[error("invalid option (key `{key:?}`, value `{value:?}`)")]
    InvalidOption {
        key: String,
        value: String,
        source: anyhow::Error,
    },

    #[error(r#"invalid format ("{s}")"#)]
    InvalidFormat { s: String, source: anyhow::Error },

    #[error("requested {resource} but its not available")]
    Unavailable {
        resource: String,
        source: anyhow::Error,
    },

    #[error("SQL error")]
    Sql(anyhow::Error),
}

impl SpringError {
    /// Get who is responsible for the error.
    /// Used for error handling and bug reports.
    pub fn responsibility(&self) -> SpringErrorResponsibility {
        match self {
            SpringError::ForeignIo { .. } => SpringErrorResponsibility::Foreign,

            SpringError::ForeignSourceTimeout { .. }
            | SpringError::InputTimeout { .. }
            | SpringError::SpringQlCoreIo(_) => SpringErrorResponsibility::SpringQlCore,

            SpringError::InvalidOption { .. }
            | SpringError::InvalidFormat { .. }
            | SpringError::Sql(_)
            | SpringError::Unavailable { .. } => SpringErrorResponsibility::Client,
        }
    }
}