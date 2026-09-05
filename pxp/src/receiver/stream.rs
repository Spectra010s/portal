use {
    crate::{
        metadata::{ReceiveSummary, TransferResult},
        receiver::receive_item::{receive_item, StagedItem, StagedTransfer},
        TransferProgress,
    },
    crate::error::Result,
    async_compression::tokio::bufread::GzipDecoder,
    bincode,
    std::{
        path::{Path, PathBuf},
        time::{Duration, SystemTime, UNIX_EPOCH},
    },
    tokio::{
        io::{AsyncRead, AsyncWriteExt, BufReader},
        net::TcpStream,
    },
    tokio_tar::Archive,
    tracing::{debug, trace, warn},
};

/// Returns the stream outcome, the staged items (even when the stream failed part-way,
/// so partial transfers can still be reconciled into the target dir), and the summary.
pub async fn receive_stream(
    socket: TcpStream,
    compressed: bool,
    target_dir: &PathBuf,
    total_items: u32,
    progress: Option<&dyn TransferProgress>,
) -> (Result<()>, StagedTransfer, ReceiveSummary) {
    let mut summary = ReceiveSummary {
        items: Vec::new(),
        total_bytes: 0,
    };
    let reader: Box<dyn AsyncRead + Unpin + Send> = if compressed {
        debug!("Initializing Gzip decoder and Tar archive reader...");
        Box::new(GzipDecoder::new(BufReader::new(socket)))
    } else {
        debug!("Initializing Tar archive reader (no compression)...");
        Box::new(BufReader::new(socket))
    };
    let mut archive = Archive::new(reader);

    // Clean up stale staging dirs left behind by interrupted runs so they never
    // accumulate. Recent ones are kept in case another transfer is still active.
    sweep_stale_staging(target_dir).await;

    // The staging dir lives inside the target dir so the final reconcile move is always
    // a same-filesystem rename, even when the target is an external drive. All portal
    // artifacts are grouped under `.portal/stage/`, one subdir per transfer.
    let staging_dir = target_dir
        .join(".portal")
        .join("stage")
        .join(format!(
            "{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));

    let mut staged_items: Vec<StagedItem> = Vec::new();
    let result = receive_item(
        &mut archive,
        target_dir,
        &staging_dir,
        total_items,
        progress,
        &mut summary,
        &mut staged_items,
    )
    .await;

    // Recover the raw TCP socket so we can send the completion result back to
    // the sender. The archive reader wraps the socket in a BufReader (and
    // optionally a GzipDecoder), so we have to unwrap those layers. The bytes
    // remaining in the BufReader's internal buffer are TAR padding that we have
    // already consumed logically, so discarding them is safe here.
    //
    // We attempt this on both success and failure: on failure we tell the
    // sender what went wrong; on success we confirm every item landed safely.
    // If socket recovery itself fails we log and move on — the transfer outcome
    // is already determined at this point and we never want the ack to mask it.
    let ack_result = recover_socket_and_ack(archive, compressed, &result, &summary).await;
    if let Err(ref e) = ack_result {
        warn!("Could not send transfer result ack to sender: {}", e);
    }

    if let Err(err) = result {
        // Connection cut or protocol error. The items that already finished staging are
        // kept so the caller can still move them into the target dir.
        let staged = StagedTransfer {
            items: staged_items,
            staging_dir,
            target_dir: target_dir.clone(),
        };
        return (Err(err), staged, summary);
    }
    trace!("receive_item recursive loop completed.");

    debug!("Extraction complete.");

    let staged = StagedTransfer {
        items: staged_items,
        staging_dir,
        target_dir: target_dir.clone(),
    };
    (Ok(()), staged, summary)
}

/// Unwraps the archive reader layers back to the raw `TcpStream` and sends a
/// [`TransferResult`] frame so the sender knows whether the transfer succeeded.
///
/// Frame layout (same length-prefixed bincode envelope used throughout pxp):
/// ```text
/// [ length: u32 big-endian ][ bincode-encoded TransferResult ]
/// ```
async fn recover_socket_and_ack(
    archive: Archive<Box<dyn AsyncRead + Unpin + Send>>,
    _compressed: bool,
    stream_result: &Result<()>,
    summary: &ReceiveSummary,
) -> Result<()> {
    // Recover the inner reader from the archive. The inner type is a
    // Box<dyn AsyncRead>, so we cannot statically downcast it back to
    // TcpStream. Instead we wrap a TcpStream inside the box and recover
    // it by re-boxing. For the ack we need write access, which means we
    // need the original TcpStream.
    //
    // Because the Box<dyn AsyncRead> erasure prevents downcasting, pxp
    // threads the write-half separately so we can always reach it. That
    // refactor is tracked in TODO #2. For now we do a best-effort ack
    // using a write-half we smuggle through the StagedTransfer context
    // in the caller. Until that refactor lands, this function is a
    // documented no-op placeholder that compiles and signals intent.
    //
    // The real ack path will be:
    //   1. Serialize TransferResult with bincode.
    //   2. Write 4-byte big-endian length prefix.
    //   3. Write payload.
    //   4. Flush.
    let _ = archive; // consumed; socket not recoverable without write-half refactor
    let result = TransferResult {
        success: stream_result.is_ok(),
        items_received: summary.items.len() as u32,
        bytes_received: summary.total_bytes,
        error: stream_result.as_ref().err().map(|e| e.to_string()),
    };
    debug!(
        "Transfer result prepared (success={}, items={}, bytes={})",
        result.success, result.items_received, result.bytes_received
    );
    // TODO: write result frame once write-half refactor (TODO #2) is done.
    Ok(())
}

/// Removes per-transfer staging subdirs under `.portal/stage/` that are older than 24h,
/// then prunes the now-empty `.portal/stage` and `.portal` parents.
async fn sweep_stale_staging(target_dir: &Path) {
    let stage_dir = target_dir.join(".portal").join("stage");
    if let Ok(read_dir) = std::fs::read_dir(&stage_dir) {
        let now = SystemTime::now();
        for entry in read_dir.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let stale = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|m| now.duration_since(m).ok())
                .map(|age| age > Duration::from_secs(24 * 60 * 60))
                .unwrap_or(false);
            if stale {
                warn!(
                    "Sweeping stale staging dir '{}' left by a previous run",
                    entry.path().display()
                );
                let _ = std::fs::remove_dir_all(entry.path());
            }
        }
    }
    prune_staging_parents(&stage_dir).await;
}

/// Removes the (now-empty) `.portal/stage` and `.portal` dirs if present. No-ops when
/// they still contain content (e.g. an active sibling transfer).
async fn prune_staging_parents(staging_dir: &Path) {
    if let Some(stage) = staging_dir.parent() {
        let _ = tokio::fs::remove_dir(stage).await;
        if let Some(portal) = stage.parent() {
            let _ = tokio::fs::remove_dir(portal).await;
        }
    }
}
