mod get_dir;
mod local_ip;
mod receive_item;
mod history;
mod handshake;
mod stream;

use {
    crate::{
        history::{append_record, HistoryStatus},
        progress::{ProgressManager, Side},
    },
    stream::receive_stream,
    history::build_receive_history_record,
    handshake::accept_and_read_manifest,
    anyhow::Result,
    get_dir::get_target_dir,
    std::{path::PathBuf,
    time::Instant, },
    tracing::{debug, error, info, trace, warn},
};

pub async fn start_receiver(port: Option<u16>, dir: &Option<PathBuf>) -> Result<()> {
    info!("Portal: Initializing receiver systems...");
    let mut peer_addr: Option<String> = None;
    let mut peer_username: Option<String> = None;
    let mut start_ts_unix = 0u64;
    let mut start_instant = Instant::now();
    let mut expected_items: Option<u32> = None;
    let mut expected_bytes: u64 = 0;

    let result: Result<()> = async {

    let handshake = accept_and_read_manifest(port).await?;
    let socket = handshake.socket;
    peer_addr = handshake.peer_addr;
    peer_username = handshake.peer_username.clone();
    start_ts_unix = handshake.start_ts_unix;
    start_instant = handshake.start_instant;
    let global_manifest = handshake.manifest;

    let total_directories = &global_manifest.total_directories;
    let total_files = global_manifest.total_files;
    let description = global_manifest.description.clone();
    if let Some(name) = &peer_username {
        info!("Sender username received in manifest: {}", name);
    } else {
        warn!("No sender username provided in manifest");
    }
    expected_bytes = global_manifest.total_bytes;
    let compressed = global_manifest.compressed;
    if compressed {
        info!("Incoming transfer is gzip-compressed");
    } else {
        info!("Incoming transfer is not compressed");
    }

    let total_items = total_files + total_directories;
    expected_items = Some(total_items);

    // Print basic info for the user
    println!("Portal: Incoming transfer - {} item(s)", total_items);

    if let Some(desc) = &description {
        println!("Portal: Sender left a note: \"{}\"", desc);
        info!("Transfer Note: {}", desc);
    } else {
        info!("Transfer has no description.");
    }

    // Determine the directory to save files
    let target_dir = get_target_dir(&dir).await?;
    info!("Target directory for saving: {:?}", target_dir);

    // progress manager for receiver UI
    let prog = ProgressManager::new_with_side(Side::Receiver);
    debug!("Progress UI created for receiver");
    prog.set_total_items(total_items as usize);
    trace!("Progress UI initialized with total_items={}", total_items);

    let summary = receive_stream(
        socket,
        compressed,
        &target_dir,
        total_items,
        Some(prog.clone()),
    )
    .await?;

    info!(
        "SUCCESS: Transfer completed. Saved to {}",
        target_dir.display()
    );
    prog.println(format!(
        "Portal: All item(s) have been received successfully! Saved to '{}'",
        target_dir.display()
    ));

    let duration_ms = start_instant.elapsed().as_millis() as u64;
    debug!("Preparing successful receive history record (duration: {}ms)", duration_ms);
    let record = build_receive_history_record(
        start_ts_unix,
        duration_ms,
        HistoryStatus::Success,
        peer_addr.clone(),
        peer_username.clone(),
        Some(target_dir.display().to_string()),
        description.clone(),
        expected_items.unwrap_or(summary.items.len() as u32),
        expected_bytes,
        summary.items.len() as u32,
        summary.total_bytes,
        Some(summary.items),
    );
    if let Err(e) = append_record(&record).await {
        error!("Failed to append history record: {:#}", e);
    } else {
        info!("Successfully appended receive history record.");
        trace!("Appended success record: {:?}", record);
    }

    Ok(())
    }
    .await;

    if let Err(ref e) = result {
        let duration_ms = start_instant.elapsed().as_millis() as u64;
        debug!("Preparing failed receive history record (duration: {}ms)", duration_ms);
    let mut record = build_receive_history_record(
        start_ts_unix,
        duration_ms,
        HistoryStatus::Failed,
        peer_addr,
        peer_username,
        None,
        None,
        expected_items.unwrap_or(0),
        expected_bytes,
        0,
        0,
        None,
    );
    record.error = Some(format!("{:#}", e));
        if let Err(err) = append_record(&record).await {
            error!("Failed to append failed history record: {:#}", err);
        } else {
            info!("Successfully appended failed receive history record.");
            trace!("Appended failed record details: {:?}", record);
        }
    }

    result
}
