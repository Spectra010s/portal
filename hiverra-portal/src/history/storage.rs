use {
    crate::{
        config::models::PortalConfig,
        history::models::TransferHistoryRecord,
    },
    anyhow::{Context, Result},
    std::path::PathBuf,
    tokio::{
        fs::create_dir_all,
        fs::OpenOptions,
        io::AsyncWriteExt,
    },
    tracing::{debug, trace, warn},
};

pub async fn history_path() -> Result<PathBuf> {
    let home_dir = PortalConfig::get_dir()
        .await
        .context("Could not determine portal directory")?;
    let path = home_dir.join("history.jsonl");
    trace!("History path resolved: {}", path.display());
    Ok(path)
}

pub async fn append_record(record: &TransferHistoryRecord) -> Result<()> {
    let path = history_path().await?;
    if let Some(parent) = path.parent() {
        create_dir_all(parent)
            .await
            .context("Failed to create history directory")?;
    }
    debug!(
        "Appending history record: mode={:?}, intended_count={}, actual_count={}, status={:?}",
        record.mode, record.intended_count, record.actual_count, record.status
    );
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .with_context(|| format!("Failed to open history file: {}", path.display()))?;
    let line = serde_json::to_string(record).context("Failed to serialize history record")?;
    file.write_all(line.as_bytes()).await?;
    file.write_all(b"\n").await?;
    if file.flush().await.is_err() {
        warn!("History write flush failed for {}", path.display());
    }
    Ok(())
}

pub async fn load_history() -> Result<Vec<TransferHistoryRecord>> {
    trace!("Loading transfer history records");
    let path = history_path().await?;
    let content = match tokio::fs::read_to_string(&path).await {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).with_context(|| format!("Failed to read {}", path.display())),
    };
    let mut records = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<TransferHistoryRecord>(line) {
            Ok(r) => records.push(r),
            Err(e) => warn!("Skipping invalid history line {}: {}", idx + 1, e),
        }
    }
    debug!("Loaded {} history records", records.len());
    Ok(records)
}
