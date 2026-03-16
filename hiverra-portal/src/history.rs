use {
    crate::config::models::PortalConfig,
    anyhow::{Context, Result},
    serde::{Deserialize, Serialize},
    std::path::PathBuf,
    std::time::{SystemTime, UNIX_EPOCH},
    tokio::{
        fs::{create_dir_all, OpenOptions},
        io::AsyncWriteExt,
    },
    tracing::{debug, trace, warn},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryDirection {
    Send,
    Receive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryStatus {
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryItemKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub name: String,
    pub bytes: u64,
    pub kind: HistoryItemKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferHistoryRecord {
    pub ts_unix: u64,
    pub direction: HistoryDirection,
    pub items: u32,
    pub total_bytes: u64,
    pub peer: Option<String>,
    pub status: HistoryStatus,
    pub error: Option<String>,
    pub item_list: Option<Vec<HistoryItem>>,
}

impl TransferHistoryRecord {
    pub fn now_unix() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

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
        "Appending history record: direction={:?}, items={}, bytes={}, status={:?}",
        record.direction, record.items, record.total_bytes, record.status
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
