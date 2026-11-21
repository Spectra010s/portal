use serde::{Deserialize, Serialize};

/// A structured error frame the receiver sends to the sender when it must abort
/// the transfer early — for example on a disk-full error, a permission failure,
/// or a user-initiated cancel during conflict resolution.
///
/// Frame layout (same length-prefixed bincode envelope used throughout pxp):
/// ```text
/// [ length: u32 big-endian ][ bincode-encoded ReceiverError ]
/// ```
///
/// The sender reads this on its next write attempt (which will otherwise see an
/// opaque `BrokenPipe`). The frame is best-effort: if the TCP write fails the
/// sender still gets the connection error, just without the structured reason.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReceiverError {
    /// Machine-readable abort reason.
    pub kind: ReceiverAbortKind,
    /// Human-readable detail message.
    pub message: String,
}

/// Categorises why the receiver had to abort the transfer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ReceiverAbortKind {
    /// The target filesystem has no space left.
    DiskFull,
    /// A filesystem write failed for reasons other than disk space.
    WriteError,
    /// The user cancelled during conflict resolution.
    UserCancelled,
    /// A protocol invariant was violated (item count, metadata mismatch, etc.).
    ProtocolError,
    /// Catch-all for errors that do not fit a specific category.
    Other,
}
/// has been consumed and all staged items have been reconciled into the target
/// directory.  The sender blocks on this before recording its history entry,
/// so both sides converge on the same success/failure view of the transfer.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransferResult {
    /// `true` when every item was staged and reconciled without error.
    pub success: bool,
    /// Number of top-level items that were successfully moved into place.
    pub items_received: u32,
    /// Total bytes written to the target directory.
    pub bytes_received: u64,
    /// Human-readable error description when `success` is `false`.
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GlobalTransferManifest {
    pub total_files: u32,
    pub total_directories: u32,
    pub total_bytes: u64,
    pub description: Option<String>,
    pub sender_username: Option<String>,
    pub compressed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetadata {
    pub filename: String,
    pub file_size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirectoryMetadata {
    pub dirname: String,
    pub total_size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransferItem {
    File(FileMetadata),
    Directory(DirectoryMetadata),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PxpMeta {
    Item(TransferItem),
    NestedFile(FileMetadata),
}

/// A single item that was received during a transfer.
#[derive(Debug, Clone)]
pub struct ReceivedItem {
    pub name: String,
    pub bytes: u64,
    pub is_directory: bool,
}

/// Summary of items received during a transfer.
#[derive(Debug, Clone)]
pub struct ReceiveSummary {
    pub items: Vec<ReceivedItem>,
    pub total_bytes: u64,
}
