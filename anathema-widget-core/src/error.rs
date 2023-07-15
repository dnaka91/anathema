use crate::Path;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to lookup id
    #[error("failed to lookup path")]
    IdNotFound(Path),

    /// Failed to lookup widget by the ident
    #[error("unregistered widget: {0}")]
    UnregisteredWidget(String),

    /// Reserved widget name
    #[error("reserved name: {0}")]
    ReservedName(String),

    /// Existing name
    #[error("existing name: {0} is already registered")]
    ExistingName(String),

    #[error("insufficient layout space available")]
    InsufficientSpaceAvailble,

    /// IO error
    #[error("{0}")]
    Io(#[from] std::io::Error),
}