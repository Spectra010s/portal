use crate::history::{HistoryItem, HistoryMode, HistoryStatus, TransferHistoryRecord};

pub fn build_receive_history_record(
    timestamp: u64,
    duration_ms: u64,
    status: HistoryStatus,
    peer_addr: Option<String>,
    peer_username: Option<String>,
    receiver_path: Option<String>,
    description: Option<String>,
    intended_count: u32,
    intended_bytes: u64,
    actual_count: u32,
    actual_bytes: u64,
    actual_items: Option<Vec<HistoryItem>>,
) -> TransferHistoryRecord {
    TransferHistoryRecord {
        timestamp,
        duration_ms,
        mode: HistoryMode::Receive,
        peer_addr,
        peer_username,
        receiver_path,
        description,
        status,
        error: None,
        intended_count,
        intended_bytes,
        intended_items: None,
        actual_count,
        actual_bytes,
        actual_items,
    }
}
