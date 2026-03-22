use crate::history::{HistoryItem, HistoryMode, HistoryStatus, TransferHistoryRecord};

pub fn build_history_record(
    timestamp: u64,
    duration_ms: u64,
    status: HistoryStatus,
    peer_addr: Option<String>,
    peer_username: Option<String>,
    description: Option<String>,
    intended_items: Vec<HistoryItem>,
    intended_bytes: u64,
    actual_items: Vec<HistoryItem>,
    actual_bytes: u64,
) -> TransferHistoryRecord {
    TransferHistoryRecord {
        timestamp,
        duration_ms,
        mode: HistoryMode::Send,
        peer_addr,
        peer_username,
        receiver_path: None,
        description,
        status,
        error: None,
        intended_count: intended_items.len() as u32,
        intended_bytes,
        intended_items: if intended_items.is_empty() {
            None
        } else {
            Some(intended_items)
        },
        actual_count: actual_items.len() as u32,
        actual_bytes,
        actual_items: if actual_items.is_empty() {
            None
        } else {
            Some(actual_items)
        },
    }
}
