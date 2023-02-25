/// Result type for the soliddb crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The Error type for the soliddb crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Returned if the given key was not found.
    #[error("not found")]
    NotFound,

    /// Returned if the given value would violate unique constraints.
    #[error("already exists")]
    AlreadyExists,

    /// Returned if encoding or decoding failed.
    #[error("encoding failed")]
    Encoding(#[from] ron::Error),

    /// Returned if the found key has the wrong format.
    #[error("malformed key")]
    MalformedKey,

    /// Returned if rocksdb returned an error.
    #[error("internal rocksdb error: {0}")]
    Internal(#[from] rocksdb::Error),

    /// Returned if the given directory could not be created.
    #[error("database creation failed: {0}")]
    CreateDirectory(std::io::Error),
}
