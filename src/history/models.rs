use {
    chrono::Utc,
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum HistoryMode {
    Send,
    Receive,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
pub struct ReceiveSummary {
    pub items: Vec<HistoryItem>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferHistoryRecord {
    pub timestamp: u64,
    pub duration_ms: u64,
    pub mode: HistoryMode,
    pub peer_addr: Option<String>,
    pub peer_username: Option<String>,
    pub receiver_path: Option<String>,
    pub description: Option<String>,
    pub status: HistoryStatus,
    pub error: Option<String>,
    pub intended_count: u32,
    pub intended_bytes: u64,
    pub intended_items: Option<Vec<HistoryItem>>,
    pub actual_count: u32,
    pub actual_bytes: u64,
    pub actual_items: Option<Vec<HistoryItem>>,
}

impl TransferHistoryRecord {
    pub fn now_unix() -> u64 {
        Utc::now().timestamp().max(0) as u64
    }
}
