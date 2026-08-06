use {
    crate::history::models::{HistoryItemKind, HistoryMode, HistoryStatus, TransferHistoryRecord},
    chrono::{TimeZone, Utc},
    tracing::debug,
};

pub fn output_history_table(records: &[TransferHistoryRecord]) {
    debug!("Outputting history table with {} records", records.len());
    for line in format_history_table(records) {
        println!("{}", line);
    }
}

pub fn format_history_table(records: &[TransferHistoryRecord]) -> Vec<String> {
    let mut lines = Vec::new();
    const W_ID: usize = 3;
    const W_DATE: usize = 10;
    const W_MODE: usize = 9;
    const W_STATUS: usize = 9;
    const W_ITEMS: usize = 6;
    const W_SIZE: usize = 9;

    let header_ansi = format!(
        "{} {} {} {} {} {} {}",
        pad_ansi_right("\x1b[4mID\x1b[0m", W_ID),
        pad_ansi_right("\x1b[4mDATE\x1b[0m", W_DATE),
        pad_ansi_right("\x1b[4mMODE\x1b[0m", W_MODE),
        pad_ansi_right("\x1b[4mSTATUS\x1b[0m", W_STATUS),
        pad_ansi_left("\x1b[4mITEMS\x1b[0m", W_ITEMS),
        pad_ansi_left("\x1b[4mSIZE\x1b[0m", W_SIZE),
        "\x1b[4mPEER\x1b[0m"
    );
    lines.push(header_ansi);
    for (idx, record) in records.iter().enumerate() {
        let date = format_date(record.timestamp);
        let mode = format_mode(record.mode);
        let status = format_status(record.status);
        let peer = format_peer(record);
        let items = record.actual_count.to_string();
        let bytes = format_bytes(record.actual_bytes);
        let id = pad_ansi_right(&format!("\x1b[36m{}\x1b[0m", idx + 1), W_ID);
        lines.push(format!(
            "{} {} {} {} {} {} {}",
            id,
            pad_right(&date, W_DATE),
            pad_right(&mode, W_MODE),
            pad_right(&status, W_STATUS),
            pad_center(&items, W_ITEMS),
            pad_left(&bytes, W_SIZE),
            peer
        ));
    }
    lines
}

pub fn format_date(timestamp: u64) -> String {
    Utc.timestamp_opt(timestamp as i64, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| timestamp.to_string())
}

pub fn format_duration(duration_ms: u64) -> String {
    format!("{:.2}s", (duration_ms as f64) / 1000.0)
}

pub fn format_mode(mode: HistoryMode) -> String {
    match mode {
        HistoryMode::Send => "send".to_string(),
        HistoryMode::Receive => "receive".to_string(),
    }
}

pub fn format_status(status: HistoryStatus) -> String {
    match status {
        HistoryStatus::Success => "success".to_string(),
        HistoryStatus::Failed => "failed".to_string(),
    }
}

pub fn format_peer(record: &TransferHistoryRecord) -> String {
    match (record.peer_username.as_deref(), record.peer_addr.as_deref()) {
        (Some(user), Some(addr)) => format!("{} ({})", user, addr),
        (Some(user), None) => user.to_string(),
        (None, Some(addr)) => addr.to_string(),
        (None, None) => "unknown".to_string(),
    }
}

pub fn items_label(mode: HistoryMode) -> &'static str {
    match mode {
        HistoryMode::Send => "sent",
        HistoryMode::Receive => "received",
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0usize;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else if size >= 100.0 {
        format!("{:.0} {}", size, UNITS[unit])
    } else if size >= 10.0 {
        format!("{:.1} {}", size, UNITS[unit])
    } else {
        format!("{:.2} {}", size, UNITS[unit])
    }
}

fn pad_left(text: &str, width: usize) -> String {
    if text.len() >= width {
        return text.to_string();
    }
    let mut s = String::with_capacity(width);
    for _ in 0..(width - text.len()) {
        s.push(' ');
    }
    s.push_str(text);
    s
}

fn pad_right(text: &str, width: usize) -> String {
    if text.len() >= width {
        return text.to_string();
    }
    let mut s = String::with_capacity(width);
    s.push_str(text);
    for _ in 0..(width - text.len()) {
        s.push(' ');
    }
    s
}

fn pad_center(text: &str, width: usize) -> String {
    let len = text.len();
    if len >= width {
        return text.to_string();
    }
    let total = width - len;
    let left = total / 2;
    let right = total - left;
    let mut s = String::with_capacity(width);
    for _ in 0..left {
        s.push(' ');
    }
    s.push_str(text);
    for _ in 0..right {
        s.push(' ');
    }
    s
}

fn pad_ansi_left(text: &str, width: usize) -> String {
    let visible = strip_ansi(text).len();
    if visible >= width {
        return text.to_string();
    }
    let mut s = String::with_capacity(text.len() + (width - visible));
    for _ in 0..(width - visible) {
        s.push(' ');
    }
    s.push_str(text);
    s
}

fn pad_ansi_right(text: &str, width: usize) -> String {
    let visible = strip_ansi(text).len();
    if visible >= width {
        return text.to_string();
    }
    let mut s = String::with_capacity(text.len() + (width - visible));
    s.push_str(text);
    for _ in 0..(width - visible) {
        s.push(' ');
    }
    s
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_escape = false;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
            continue;
        }
        if ch == '\x1b' {
            if let Some('[') = chars.peek() {
                in_escape = true;
                continue;
            }
        }
        out.push(ch);
    }
    out
}

pub fn format_history_detail(
    record: &TransferHistoryRecord,
    id: usize,
    items_all: bool,
) -> Vec<String> {
    debug!(
        "Formatting detail view for record #{} (items_all={})",
        id, items_all
    );
    let mut lines = Vec::new();
    lines.push(format!("Portal History • Record #{}\n", id));
    let date = format_date(record.timestamp);
    let mode = format_mode(record.mode);
    let status = format_status(record.status);
    lines.push(format!("Date: {}", date));
    lines.push(format!("Duration: {}", format_duration(record.duration_ms)));
    lines.push(format!("Mode: {}", mode));
    lines.push(format!("Status: {}", status));
    if matches!(record.mode, HistoryMode::Send) {
        lines.push(format!("Total Items Intended: {}", record.intended_count));
        lines.push(format!("Total Items Sent: {}", record.actual_count));
    } else {
        lines.push(format!("Total Items Expected: {}", record.intended_count));
        lines.push(format!("Total Items Received: {}", record.actual_count));
    }
    lines.push(format!(
        "Total Size Intended: {}",
        format_bytes(record.intended_bytes)
    ));
    if matches!(record.mode, HistoryMode::Send) {
        lines.push(format!(
            "Total Size Sent: {}",
            format_bytes(record.actual_bytes)
        ));
    } else {
        lines.push(format!(
            "Total Size Received: {}",
            format_bytes(record.actual_bytes)
        ));
    }
    lines.push(format!("Peer: {}", format_peer(record)));
    if let Some(path) = record.receiver_path.as_deref() {
        lines.push(format!("Receiver Path: {}", path));
    }
    lines.push(format!(
        "Transfer Description: {}",
        record.description.as_deref().unwrap_or("none")
    ));
    if let Some(err) = record.error.as_deref() {
        lines.push(format!("Error: {}", err));
    }

    // Item lists (capped unless --items-all)
    let cap = 5usize;

    if let Some(items) = record.actual_items.as_ref() {
        lines.push(String::new());
        let label = if matches!(record.mode, HistoryMode::Send) {
            "Items Sent:"
        } else {
            "Items Received:"
        };
        lines.push(label.to_string());
        let shown = if items_all {
            items.len()
        } else {
            items.len().min(cap)
        };
        for item in items.iter().take(shown) {
            lines.push(format!(
                "- {} — {} ({})",
                item.name,
                format_bytes(item.bytes),
                match item.kind {
                    HistoryItemKind::File => "File",
                    HistoryItemKind::Directory => "Directory",
                }
            ));
        }
        if !items_all && items.len() > cap {
            lines.push(format!("(+{} more)", items.len() - cap));
        }
    }

    if let Some(items) = record.intended_items.as_ref() {
        lines.push(String::new());
        lines.push("Intended Items:".to_string());
        let shown = if items_all {
            items.len()
        } else {
            items.len().min(cap)
        };
        for item in items.iter().take(shown) {
            lines.push(format!(
                "- {} — {} ({})",
                item.name,
                format_bytes(item.bytes),
                match item.kind {
                    HistoryItemKind::File => "File",
                    HistoryItemKind::Directory => "Directory",
                }
            ));
        }
        if !items_all && items.len() > cap {
            lines.push(format!("(+{} more)", items.len() - cap));
        }
    }
    lines
}
