use thiserror::Error;

/// Top-level error type for pxp operations.
#[derive(Debug, Error)]
pub enum PxpError {
    /// Network/IO errors
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors (bincode)
    #[error("Received data that could not be understood: {0}")]
    Bincode(#[from] bincode::Error),

    /// JSON serialization errors
    #[error("Discovery message was malformed: {0}")]
    Json(#[from] serde_json::Error),

    /// UTF-8 conversion errors
    #[error("Received text data was not valid: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// StripPrefix errors from path operations
    #[error("A file path could not be resolved: {0}")]
    StripPrefix(#[from] std::path::StripPrefixError),

    /// Discovery timed out
    #[error("Discovery timed out: {message}")]
    DiscoveryTimeout { message: String },

    /// Discovery beacon stopped unexpectedly
    #[error("Discovery beacon stopped unexpectedly")]
    BeaconStopped,

    /// Connection failed
    #[error("Failed to connect to receiver at {address}")]
    ConnectionFailed {
        address: String,
        #[source]
        source: std::io::Error,
    },

    /// Identity mismatch during handshake
    #[error("Security: ID mismatch — claimed '{claimed}', expected '{expected}'")]
    IdentityMismatch { claimed: String, expected: String },

    /// Port binding failed
    #[error("Failed to bind to port {port}")]
    BindFailed {
        port: u16,
        #[source]
        source: std::io::Error,
    },

    /// Protocol violation (metadata mismatch, item count mismatch, etc.)
    #[error("The other device sent unexpected data: {0}")]
    Protocol(String),

    /// Security violation (too many items, etc.)
    #[error("Blocked a potentially unsafe transfer: {0}")]
    Security(String),

    /// Conflict resolution error (from the consumer's resolver)
    #[error("Could not resolve a file naming conflict: {0}")]
    ConflictResolution(String),

    /// Compression error
    #[error("Could not compress or decompress the transfer data: {0}")]
    Compression(String),

    /// Tar archive error  
    #[error("Could not package the files for sending: {0}")]
    Archive(String),

    /// Walkdir error
    #[error("Could not read one of the files or folders you're trying to send: {0}")]
    WalkDir(String),
}

pub type Result<T> = std::result::Result<T, PxpError>;
