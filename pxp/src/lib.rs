//! # PXP Crate
//!
//! This crate implements the core protocol logic for the **Portal Transfer Protocol (PXP)**,
//! a lightweight, transport-neutral, streaming protocol optimized for zero-configuration
//! local area network file delivery.
//!
//! The protocol design and specifications are documented under the `/spec` directory in the repository:
//! - [Overview](https://github.com/Spectra010s/portal/blob/main/spec/draft-pxp-overview-00.md)
//! - [PXP-DISCOVERY](https://github.com/Spectra010s/portal/blob/main/spec/draft-pxp-discovery-00.md)
//! - [PXP-HANDSHAKE](https://github.com/Spectra010s/portal/blob/main/spec/draft-pxp-handshake-00.md)
//! - [PXP-MANIFEST](https://github.com/Spectra010s/portal/blob/main/spec/draft-pxp-manifest-00.md)
//! - [PXP-STREAMING](https://github.com/Spectra010s/portal/blob/main/spec/draft-pxp-streaming-00.md)

pub mod error;
pub mod discovery;
pub mod metadata;
pub mod receiver;
pub mod sender;

pub use error::{PxpError, Result};

use tokio::io::{AsyncRead, AsyncWrite};

/// Trait for tracking progress of individual file/directory items during transfer.
pub trait ItemProgress: Send + Sync {
    /// Wrap an async reader with progress tracking.
    fn wrap_read(
        &self,
        reader: Box<dyn AsyncRead + Unpin + Send>,
    ) -> Box<dyn AsyncRead + Unpin + Send>;
    /// Wrap an async writer with progress tracking.
    fn wrap_write(
        &self,
        writer: Box<dyn AsyncWrite + Unpin + Send>,
    ) -> Box<dyn AsyncWrite + Unpin + Send>;
    /// Signal completion of this item's progress tracking.
    fn finish_and_clear(&self);
}

/// Trait for managing overall transfer progress across multiple items.
pub trait TransferProgress: Send + Sync {
    fn set_total_items(&self, total: usize);
    fn set_current_item(&self, current: usize, total: usize);
    fn create_item_progress(&self, name: &str, total_bytes: u64) -> Box<dyn ItemProgress>;
    fn println(&self, msg: &str);
}

/// Action to take when a file conflict occurs during receive.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConflictAction {
    Overwrite,
    OverwriteAll,
    Rename,
    RenameAll,
    Skip,
    SkipAll,
}

/// Trait for resolving file name conflicts during receive.
pub trait ConflictResolver: Send + Sync {
    fn resolve(&self, item_name: &str) -> crate::error::Result<ConflictAction>;
}
