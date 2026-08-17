use {
    crate::{
        metadata::ReceiveSummary,
        receiver::receive_item::{receive_item, StagedItem, StagedTransfer},
        TransferProgress,
    },
    crate::error::Result,
    async_compression::tokio::bufread::GzipDecoder,
    std::{
        path::{Path, PathBuf},
        time::{Duration, SystemTime, UNIX_EPOCH},
    },
    tokio::{
        io::{AsyncRead, BufReader},
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

    debug!("Extraction complete. Recovering stream...");
    let _reader = archive.into_inner();
    trace!("Archive reader recovered.");

    let staged = StagedTransfer {
        items: staged_items,
        staging_dir,
        target_dir: target_dir.clone(),
    };
    (Ok(()), staged, summary)
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
