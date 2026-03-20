use {
    crate::history::models::{HistoryMode, TransferHistoryRecord},
    anyhow::{Context, Result},
    chrono::{DateTime, NaiveDate, Utc},
    tracing::{debug, trace},
};

pub fn parse_since_unix(value: &str) -> Result<u64> {
    trace!("Parsing history since filter: {}", value);
    if let Ok(ts) = value.parse::<u64>() {
        return Ok(ts);
    }
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .with_context(|| format!("Invalid date format '{}', expected YYYY-MM-DD", value))?;
    let dt: DateTime<Utc> =
        DateTime::<Utc>::from_naive_utc_and_offset(date.and_hms_opt(0, 0, 0).unwrap(), Utc);
    Ok(dt.timestamp().max(0) as u64)
}

pub fn filter_history(
    records: Vec<TransferHistoryRecord>,
    mode: Option<HistoryMode>,
    since_unix: Option<u64>,
    limit: usize,
) -> Vec<TransferHistoryRecord> {
    trace!(
        "Filtering history: mode={:?}, since={:?}, limit={}",
        mode, since_unix, limit
    );
    let mut filtered: Vec<TransferHistoryRecord> = records
        .into_iter()
        .filter(|r| {
            let dir_ok = mode.as_ref().map_or(true, |d| d == &r.mode);
            let since_ok = since_unix.map_or(true, |s| r.timestamp >= s);
            dir_ok && since_ok
        })
        .collect();
    if filtered.is_empty() {
        debug!("No history records after filter");
        return filtered;
    }
    let len = filtered.len();
    let take_from = if limit == 0 {
        0
    } else {
        len.saturating_sub(limit)
    };
    filtered = filtered.split_off(take_from);
    filtered.reverse(); // newest first
    debug!("Filtered history count: {}", filtered.len());
    filtered
}
