use {
    crate::{
        history::format::{
            format_bytes, format_date, format_duration, format_mode, format_peer, format_status,
            items_label,
        },
        history::models::{HistoryItem, HistoryItemKind, TransferHistoryRecord},
    },
    anyhow::Result,
    serde::Serialize,
    tracing::debug,
};

#[derive(Debug, Serialize)]
pub struct HistoryJsonItem {
    pub name: String,
    pub size: String,
    pub kind: String,
}

#[derive(Debug, Serialize)]
pub struct HistoryJsonSummary {
    pub id: usize,
    pub date: String,
    pub duration: String,
    pub mode: String,
    pub status: String,
    pub items: String,
    pub size: String,
    pub peer: String,
}

#[derive(Debug, Serialize)]
pub struct HistoryJsonDetail {
    pub id: usize,
    pub date: String,
    pub duration: String,
    pub mode: String,
    pub status: String,
    pub total_items_intended: u32,
    pub total_items_actual: u32,
    pub total_size_intended: String,
    pub total_size_actual: String,
    pub peer_addr: Option<String>,
    pub peer_username: Option<String>,
    pub receiver_path: Option<String>,
    pub transfer_description: String,
    pub error: Option<String>,
    pub items_actual: Vec<HistoryJsonItem>,
    pub items_intended: Vec<HistoryJsonItem>,
}

pub fn output_history_json_list(records: Vec<TransferHistoryRecord>) -> Result<()> {
    debug!("Outputting JSON list for {} history records", records.len());
    let out = build_history_json_list(records)?;
    println!("{out}");
    Ok(())
}

pub fn output_history_json_detail(record: &TransferHistoryRecord, id: usize) -> Result<()> {
    debug!("Outputting JSON detail for history record #{}", id);
    let out = build_history_json_detail(record, id);
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

pub fn build_history_json_list(records: Vec<TransferHistoryRecord>) -> Result<String> {
    let mut out = Vec::with_capacity(records.len());
    for (idx, record) in records.iter().enumerate() {
        out.push(build_history_json_summary(record, idx + 1));
    }
    Ok(serde_json::to_string(&out)?)
}

pub fn build_history_json_detail_list(records: Vec<TransferHistoryRecord>) -> Result<String> {
    let mut out = Vec::with_capacity(records.len());
    for (idx, record) in records.iter().enumerate() {
        out.push(build_history_json_detail(record, idx + 1));
    }
    Ok(serde_json::to_string(&out)?)
}

fn build_history_json_summary(record: &TransferHistoryRecord, id: usize) -> HistoryJsonSummary {
    let date = format_date(record.timestamp);
    let duration = format_duration(record.duration_ms);
    let mode = format_mode(record.mode);
    let status = format_status(record.status);
    let items = format!("{} {}", record.actual_count, items_label(record.mode));
    let size = format_bytes(record.actual_bytes);
    let peer = format_peer(record);
    HistoryJsonSummary {
        id,
        date,
        duration,
        mode,
        status,
        items,
        size,
        peer,
    }
}

fn build_history_json_detail(record: &TransferHistoryRecord, id: usize) -> HistoryJsonDetail {
    let date = format_date(record.timestamp);
    let duration = format_duration(record.duration_ms);
    let mode = format_mode(record.mode);
    let status = format_status(record.status);
    let total_size_intended = format_bytes(record.intended_bytes);
    let total_size_actual = format_bytes(record.actual_bytes);
    let items_actual = record
        .actual_items
        .as_ref()
        .map(|items| items.iter().map(to_json_item).collect())
        .unwrap_or_default();
    let items_intended = record
        .intended_items
        .as_ref()
        .map(|items| items.iter().map(to_json_item).collect())
        .unwrap_or_default();
    HistoryJsonDetail {
        id,
        date,
        duration,
        mode,
        status,
        total_items_intended: record.intended_count,
        total_items_actual: record.actual_count,
        total_size_intended,
        total_size_actual,
        peer_addr: record.peer_addr.clone(),
        peer_username: record.peer_username.clone(),
        receiver_path: record.receiver_path.clone(),
        transfer_description: record
            .description
            .clone()
            .unwrap_or_else(|| "none".to_string()),
        error: record.error.clone(),
        items_actual,
        items_intended,
    }
}

fn to_json_item(item: &HistoryItem) -> HistoryJsonItem {
    HistoryJsonItem {
        name: item.name.clone(),
        size: format_bytes(item.bytes),
        kind: match item.kind {
            HistoryItemKind::File => "File".to_string(),
            HistoryItemKind::Directory => "Directory".to_string(),
        },
    }
}
