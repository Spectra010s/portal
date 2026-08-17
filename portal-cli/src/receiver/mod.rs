mod get_dir;
mod history;

use {
    crate::{
        config::models::PortalConfig,
        history::{
            HistoryItem, HistoryItemKind, HistoryStatus, TransferHistoryRecord, append_record,
        },
        progress::{ProgressManager, Side},
    },
    anyhow::{Context, Result, anyhow},
    get_dir::get_target_dir,
    history::build_receive_history_record,
    inquire::Select,
    pxp::{
        ConflictAction, ConflictResolver,
        metadata::ReceiveSummary,
    },
    std::{path::PathBuf, time::Instant},
    tracing::{debug, error, info, trace, warn},
};

/// CLI conflict resolver using inquire::Select.
///
/// If a file we are trying to receive already exists, the core library will call this method
/// to figure out what to do. Since the core doesn't know about TTYs or user prompts,
/// we handle the CLI interaction here and return the resolved action back to the core.
struct CliConflictResolver;

impl ConflictResolver for CliConflictResolver {
    fn resolve(&self, item_name: &str) -> std::result::Result<ConflictAction, pxp::PxpError> {
        let options = vec![
            "Overwrite",
            "Overwrite All",
            "Rename",
            "Rename All",
            "Skip",
            "Skip All",
        ];
        // We prompt the user interactively on the terminal using `inquire`.
        // If the prompt fails (e.g., TTY disconnected or Ctrl-C), we map the error to our typed PxpError.
        let ans = Select::new(&format!("Portal: '{}' exists. Action?", item_name), options)
            .prompt()
            .map_err(|e| pxp::PxpError::ConflictResolution(e.to_string()))?;

        // Translate the user's choice string into the corresponding core Action enum.
        match ans {
            "Overwrite" => Ok(ConflictAction::Overwrite),
            "Overwrite All" => Ok(ConflictAction::OverwriteAll),
            "Rename" => Ok(ConflictAction::Rename),
            "Rename All" => Ok(ConflictAction::RenameAll),
            "Skip" => Ok(ConflictAction::Skip),
            "Skip All" => Ok(ConflictAction::SkipAll),
            _ => unreachable!(),
        }
    }
}

pub async fn start_receiver(port: Option<u16>, dir: &Option<PathBuf>) -> Result<()> {
    info!("Portal: Initializing receiver systems...");
    let mut peer_addr: Option<String> = None;
    let mut peer_username: Option<String> = None;
    let mut start_ts_unix = 0u64;
    let mut start_instant = Instant::now();
    let mut expected_items: Option<u32> = None;
    let mut expected_bytes: u64 = 0;

    let mut partial_summary: Option<ReceiveSummary> = None;
    let result: Result<()> = async {
        // --- Resolve port ---
        let n_port = if let Some(port) = port {
            trace!("Port source: CLI argument");
            debug!("Portal: Overriding config port with CLI port: {}", port);
            port
        } else if let Some(cfg) = PortalConfig::load_or_return().await? {
            if let Some(p) = cfg.network.default_port {
                trace!("Port source: User Configuration");
                debug!("Portal: Port not given, using config port: {}", p);
                p
            } else {
                error!("Port missing in both CLI and config");
                return Err(anyhow!("No port provided and config has no port set"));
            }
        } else {
            trace!("Port source: No configuration found");
            error!("No port configuration found");
            return Err(anyhow!("No port provided and no config found"));
        };

        // --- Display local IP ---
        let my_ip = pxp::receiver::local_ip::get_local_ip().await;
        if let Some(ip) = &my_ip {
            debug!("Local IP detected: {}", ip);
        } else {
            warn!(
                "Could not detect a friendly local IP; receiver is still listening on all interfaces at port {}",
                n_port
            );
        }

        // --- Load username ---
        let full_cfg = PortalConfig::load_all()
            .await
            .context("Failed to load user config")?;

        let username = full_cfg.user.username.ok_or_else(|| {
            error!("Attempted to receive without a username set");
            anyhow!("No username found. Please run 'portal config set user.username <name>' first.")
        })?;

        // --- Print listening info ---
        if let Some(ip) = &my_ip {
            println!("Portal: Creating wormhole at {}", ip);
        } else {
            println!("Portal: Creating wormhole on port {}.", n_port);
            println!("Portal: Tip: To connect manually, find this device's local IP:");
            println!("Portal:   Windows: ipconfig");
            println!("Portal:   macOS/Linux/Android: ifconfig or ip addr");
            println!("Portal: Then run from the sender:");
            println!(
                "Portal:   portal send --address <receiver-ip> --port {} <file-or-folder>",
                n_port
            );
        }
        println!("Portal: Wormhole open for {:?}", username);

        // --- Core handshake ---
        let handshake = pxp::receiver::handshake::accept_and_read_manifest(n_port, username).await?;
        let socket = handshake.socket;
        peer_addr = handshake.peer_addr;
        peer_username = handshake.peer_username.clone();

        println!("Portal: Connection established with {}!", peer_addr.as_deref().unwrap_or("unknown"));
        println!("Portal: Connected to sender");
        println!("Portal: Waiting for incoming files...");

        start_ts_unix = TransferHistoryRecord::now_unix();
        start_instant = Instant::now();

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

        println!("Portal: Incoming transfer - {} item(s)", total_items);

        if let Some(desc) = &description {
            println!("Portal: Sender left a note: \"{}\"", desc);
            info!("Transfer Note: {}", desc);
        } else {
            info!("Transfer has no description.");
        }

        let target_dir = get_target_dir(&dir).await?;
        info!("Target directory for saving: {:?}", target_dir);

        let prog = ProgressManager::new_with_side(Side::Receiver);
        debug!("Progress UI created for receiver");
        prog.set_total_items(total_items as usize);
        trace!("Progress UI initialized with total_items={}", total_items);

        let (stream_result, staged, summary) = pxp::receiver::stream::receive_stream(
            socket,
            compressed,
            &target_dir,
            total_items,
            Some(&prog as &dyn pxp::TransferProgress),
        )
        .await;
        // Stop the progress UI before any conflict prompts so the terminal stays clean.
        prog.finish();

        // Resolve any filename collisions now that the stream is done. This runs on success
        // AND on a cut connection, so whatever was already staged still lands in the target
        // dir (same crash-safety as the old per-item finalize behavior).
        let conflict_resolver = CliConflictResolver;
        if let Err(e) =
            pxp::receiver::receive_item::reconcile(&staged, Some(&conflict_resolver as &dyn ConflictResolver))
                .await
        {
            partial_summary = Some(summary);
            return Err(e.into());
        }

        if let Err(e) = stream_result {
            println!(
                "Portal: Transfer interrupted; recovered {} item(s) to '{}'",
                staged.items.len(),
                target_dir.display()
            );
            partial_summary = Some(summary);
            return Err(e.into());
        }

        info!(
            "SUCCESS: Transfer completed. Saved to {}",
            target_dir.display()
        );
        println!(
            "Portal: All item(s) have been received successfully! Saved to '{}'",
            target_dir.display()
        );

        // Convert core summary items to CLI history items
        let history_items: Vec<HistoryItem> = summary
            .items
            .iter()
            .map(|item| HistoryItem {
                name: item.name.clone(),
                bytes: item.bytes,
                kind: if item.is_directory {
                    HistoryItemKind::Directory
                } else {
                    HistoryItemKind::File
                },
            })
            .collect();

        let duration_ms = start_instant.elapsed().as_millis() as u64;
        debug!(
            "Preparing successful receive history record (duration: {}ms)",
            duration_ms
        );
        let record = build_receive_history_record(
            start_ts_unix,
            duration_ms,
            HistoryStatus::Success,
            peer_addr.clone(),
            peer_username.clone(),
            Some(target_dir.display().to_string()),
            description.clone(),
            expected_items.unwrap_or(history_items.len() as u32),
            expected_bytes,
            history_items.len() as u32,
            summary.total_bytes,
            Some(history_items),
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
        debug!(
            "Preparing failed receive history record (duration: {}ms)",
            duration_ms
        );
        let summary = partial_summary.unwrap_or(ReceiveSummary {
            items: Vec::new(),
            total_bytes: 0,
        });
        let history_items: Vec<HistoryItem> = summary
            .items
            .iter()
            .map(|item| HistoryItem {
                name: item.name.clone(),
                bytes: item.bytes,
                kind: if item.is_directory {
                    HistoryItemKind::Directory
                } else {
                    HistoryItemKind::File
                },
            })
            .collect();
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
            history_items.len() as u32,
            summary.total_bytes,
            if history_items.is_empty() {
                None
            } else {
                Some(history_items)
            },
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
