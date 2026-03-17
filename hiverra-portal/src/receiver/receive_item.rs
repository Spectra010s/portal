use {
    crate::{
        history::{HistoryItem, HistoryItemKind, ReceiveSummary},
        progress::ProgressManager,
        metadata::{PortalMeta, TransferItem},
    },
    indicatif::ProgressBar,
    anyhow::{Context, Result, anyhow},
    bincode::deserialize,
    inquire::Select,
    std::path::PathBuf,
    tokio::{
        fs::{File, create_dir_all, remove_dir_all, remove_file, rename, try_exists},
        io::AsyncRead,
    },
    tokio_stream::StreamExt,
    tokio_tar::Archive,
    tracing::{debug, error, info, trace, warn},
};

#[derive(Clone, Copy, PartialEq)]
enum ConflictStrategy {
    Prompt,
    OverwriteAll,
    RenameAll,
    SkipAll,
}

/// Receives a single item (file or directory) from the tar archive
/// including all nested files for directories. Uses metadata to validate.
pub async fn receive_item<R>(
    archive: &mut Archive<R>,
    target_dir: &PathBuf,
    total_items: u32,
    prog: Option<ProgressManager>,
) -> Result<ReceiveSummary>
where
    R: AsyncRead + Unpin + Send,
{
    let mut contract: Option<PortalMeta> = None;
    let mut global_strategy = ConflictStrategy::Prompt;
    let mut items_processed: u32 = 0;
    let mut active_dir_pb: Option<ProgressBar> = None;
    let mut pending_dir_success: Option<String> = None;
    let mut history_items: Vec<HistoryItem> = Vec::new();
    let mut history_total_bytes: u64 = 0;

    let mut entries = archive.entries()?;
    while let Some(entry_result) = entries.next().await {
        let mut entry = entry_result.context("Failed to read tar entry")?;
        let path = entry.path()?.to_path_buf();
        let entry_size = entry.header().size()?;
        trace!("--- Processing archive entry {} ---", path.display());

        // Catch metadata
        if path.to_string_lossy().replace('\\', "/") == ".portal.meta" {
            debug!("Caught metadata block (.portal.meta)");
            let mut meta_bytes = Vec::new();
            tokio::io::copy(&mut entry, &mut meta_bytes)
                .await
                .context("Failed to read metadata")?;
            let deserialized: PortalMeta = deserialize(&meta_bytes)?;
            trace!("Deserialized metadata content: {:?}", deserialized);
            contract = Some(deserialized);
            continue;
        }

        let meta = contract.take().ok_or_else(|| {
            error!(
                "Protocol error: {} arrived without preceding metadata",
                path.display()
            );
            anyhow!(
                "Protocol error: data entry '{}' arrived without metadata",
                path.display()
            )
        })?;
        trace!(
            "Matched entry '{}' with its metadata contract.",
            path.display()
        );

        let mut entry_pb: Option<ProgressBar> = None;
        if let PortalMeta::Item(item) = &meta {
            items_processed += 1;

            if let Some(pm) = &prog {
                pm.set_current_item(items_processed as usize, total_items as usize);
            }

            match item {
                TransferItem::File(f) => {
                    trace!(
                        "Progress UI: starting file item '{}' ({} bytes)",
                        f.filename,
                        f.file_size
                    );
                    history_items.push(HistoryItem {
                        name: f.filename.clone(),
                        bytes: f.file_size,
                        kind: HistoryItemKind::File,
                    });
                    history_total_bytes = history_total_bytes.saturating_add(f.file_size);
                    trace!("History tracker: recorded received file '{}'", f.filename);
                    info!(
                        "Incoming top-level file: {} ({} bytes)",
                        f.filename, f.file_size
                    );
                }
                TransferItem::Directory(d) => {
                    trace!(
                        "Progress UI: starting directory item '{}' ({} bytes)",
                        d.dirname,
                        d.total_size
                    );
                    history_items.push(HistoryItem {
                        name: d.dirname.clone(),
                        bytes: d.total_size,
                        kind: HistoryItemKind::Directory,
                    });
                    history_total_bytes = history_total_bytes.saturating_add(d.total_size);
                    trace!("History tracker: recorded received directory '{}'", d.dirname);
                    info!(
                        "Incoming top-level directory: {} ({} bytes)",
                        d.dirname, d.total_size
                    );
                }
            }

            // Hard Stop: reject anything beyond the manifest
            if items_processed > total_items {
                error!(
                    "SECURITY ALERT: Sender attempted to send more items than manifest allowed ({} > {})",
                    items_processed, total_items
                );
                return Err(anyhow!(
                    "Security Alert: Sender sent more items than manifest allowed"
                ));
            }

            // Close any active directory progress bar before starting a new top-level item
            if let Some(pb) = active_dir_pb.take() {
                pb.finish_and_clear();
                if let Some(dir_name) = pending_dir_success.take() {
                    if let Some(pm) = &prog {
                        pm.println(format!(
                            "Portal: Directory '{}' received successfully!",
                            dir_name
                        ));
                    } else {
                        println!("Portal: Directory '{}' received successfully!", dir_name);
                    }
                }
            }

            if let Some(pm) = &prog {
                match item {
                    TransferItem::File(f) => {
                        entry_pb = Some(pm.create_file_bar(&f.filename, f.file_size));
                    }
                    TransferItem::Directory(d) => {
                        active_dir_pb = Some(pm.create_file_bar(&d.dirname, d.total_size));
                        pending_dir_success = Some(d.dirname.clone());
                    }
                }
            }
        }

        // Determine if entry is a directory or file
        let is_dir = entry.header().entry_type().is_dir();
        let item_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".into());

        let temp_path = target_dir.join(format!(".tmp_{}_portal", item_name));

        let safe_path = path
            .components()
            .filter(|c| matches!(c, std::path::Component::Normal(_)))
            .collect::<PathBuf>();
        let mut final_path = target_dir.join(safe_path);
        trace!(
            "Resolved extraction paths: temp={:?}, final={:?}",
            temp_path, final_path
        );

        // pre-check existence
        let final_exists = try_exists(&final_path).await?;
        let temp_exists = try_exists(&temp_path).await?;

        // handle conflict
        if final_exists && global_strategy != ConflictStrategy::OverwriteAll {
            warn!("Conflict detected for path: {:?}", final_path);
            if global_strategy == ConflictStrategy::SkipAll {
                debug!("Strategy SkipAll: skipping {:?}", item_name);
                continue;
            } else if global_strategy == ConflictStrategy::RenameAll {
                final_path = get_unused_path(final_path).await;
                debug!("Strategy RenameAll: new path {:?}", final_path);
            } else {
                // check user input to know next step
                let options = vec![
                    "Overwrite",
                    "Overwrite All",
                    "Rename",
                    "Rename All",
                    "Skip",
                    "Skip All",
                ];
                let ans = Select::new(&format!("Portal: '{}' exists. Action?", item_name), options)
                    .prompt()?;
                trace!("User selected conflict resolution: {}", ans);

                match ans {
                    "Overwrite" => info!("User chose to overwrite {:?}", item_name),
                    "Overwrite All" => {
                        info!("User enabled Overwrite All strategy");
                        global_strategy = ConflictStrategy::OverwriteAll;
                    }
                    "Rename" => {
                        final_path = get_unused_path(final_path).await;
                        info!("User chose to rename to {:?}", final_path);
                    }
                    "Rename All" => {
                        info!("User enabled Rename All strategy");
                        global_strategy = ConflictStrategy::RenameAll;
                        final_path = get_unused_path(final_path).await;
                    }
                    "Skip" => {
                        info!("User skipped item {:?}", item_name);
                        continue;
                    }
                    "Skip All" => {
                        info!("User enabled Skip All strategy");
                        global_strategy = ConflictStrategy::SkipAll;
                        continue;
                    }
                    _ => unreachable!(),
                }
            }
        }

        // prepare temp folder
        trace!("Cleaning/Creating temp directory: {:?}", temp_path);
        if temp_exists {
            let _ = remove_dir_all(&temp_path).await;
        }
        create_dir_all(&temp_path).await?;

        if !is_dir {
            trace!(
                "Unpacking file to temp storage: {}/{}",
                temp_path.display(),
                item_name
            );
            let file_in_temp = temp_path.join(&item_name);
            let mut outfile = File::create(&file_in_temp).await?;

            // Top-level file bar (if present), otherwise reuse active directory bar.
            if let Some(pb) = entry_pb.take() {
                let mut writer = pb.wrap_async_write(outfile);
                tokio::io::copy(&mut entry, &mut writer).await?;
                pb.finish_and_clear();
            } else if let Some(pb) = &active_dir_pb {
                let mut reader = pb.wrap_async_read(entry);
                tokio::io::copy(&mut reader, &mut outfile).await?;
            } else {
                tokio::io::copy(&mut entry, &mut outfile).await?;
            }
        }

        // move to final location
        trace!("Moving from temp to final destination: {:?}", final_path);
        if let Some(parent) = final_path.parent() {
            create_dir_all(parent).await?;
        }
        if !is_dir {
            if final_exists {
                trace!("Overwriting existing file at {:?}", final_path);
                let _ = remove_file(&final_path).await;
            }
            rename(temp_path.join(item_name), &final_path).await?;
            let _ = remove_dir_all(&temp_path).await;
        } else {
            if final_exists {
                trace!("Overwriting existing directory at {:?}", final_path);
                let _ = remove_dir_all(&final_path).await;
            }
            rename(&temp_path, &final_path).await?;
        }
        debug!("Item finalized at target path: {:?}", final_path);

        // validate metadata
        match meta {
            PortalMeta::Item(item) => match item {
                TransferItem::File(f) => {
                    // Compare for integrity

                    if f.filename != path.to_string_lossy() {
                        error!(
                            "Filename mismatch: Expected {}, got {}",
                            f.filename,
                            path.display()
                        );
                        return Err(anyhow!("Protocol error: Top-level filename mismatch"));
                    }
                    if f.file_size != entry_size {
                        error!(
                            "Size mismatch for {}: Expected {}, got {}",
                            f.filename, f.file_size, entry_size
                        );
                        trace!(
                            "Verification failure detail: manifest_size={} vs header_size={}",
                            f.file_size, entry_size
                        );
                        return Err(anyhow!("Protocol error: Top-level file size mismatch"));
                    }
                    trace!(
                        "Self-check: file size matches manifest ({} bytes)",
                        f.file_size
                    );
                    if let Some(pm) = &prog {
                        pm.println(&format!("Portal: File '{}' received successfully!", f.filename));
                    } else {
                        println!("Portal: File '{}' received successfully!", f.filename);
                    }
                    info!("Successfully verified and saved: {}", f.filename);
                    trace!("Progress UI: completed file item '{}'", f.filename);
                }
                TransferItem::Directory(d) => {
                    if d.dirname != path.to_string_lossy().replace('\\', "/") {
                        error!(
                            "Dirname mismatch: Expected {}, got {}",
                            d.dirname,
                            path.display()
                        );
                        return Err(anyhow!("Protocol error: Top-level directory name mismatch"));
                    }
                    info!("Successfully verified and saved directory: {}", d.dirname);
                    trace!("Progress UI: completed directory item '{}'", d.dirname);
                }
            },
            PortalMeta::NestedFile(f) => {
                debug!("Verifying nested item: {}", f.filename);
                if f.filename != path.to_string_lossy().replace('\\', "/") {
                    return Err(anyhow!(
                        "Protocol error: Directory filename mismatch. Expected '{}', got '{}'",
                        f.filename,
                        path.display()
                    ));
                }
                // Compare size for integrity
                if !is_dir && f.file_size != entry_size {
                    trace!(
                        "Nested file verification failure: {} (manifest: {}, header: {})",
                        f.filename, f.file_size, entry_size
                    );
                    return Err(anyhow!(
                        "Protocol error: Directory file size mismatch for '{}'",
                        f.filename
                    ));
                }
                trace!("Nested item size verified: {} bytes", f.file_size);
                info!("Nested item verified and saved: {}", f.filename);
            }
        }
    }
    if let Some(pb) = active_dir_pb.take() {
        pb.finish_and_clear();
        if let Some(dir_name) = pending_dir_success.take() {
            if let Some(pm) = &prog {
                pm.println(format!(
                    "Portal: Directory '{}' received successfully!",
                    dir_name
                ));
            } else {
                println!("Portal: Directory '{}' received successfully!", dir_name);
            }
        }
    }
    if items_processed != total_items {
        error!(
            "Transfer failed: manifest expected {} items, only received {}",
            total_items, items_processed
        );
        return Err(anyhow!(
            "Transfer incomplete: Expected {} items, only got {}",
            total_items,
            items_processed
        ));
    }
    info!("All {} items received and verified.", items_processed);
    Ok(ReceiveSummary {
        items: history_items,
        total_bytes: history_total_bytes,
    })
}

/// helper to get path for incremental renaming
async fn get_unused_path(path: PathBuf) -> PathBuf {
    let mut n = 1;
    // Provide safe defaults instead of crashing
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| "file".into());
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let parent = path.parent().unwrap_or(std::path::Path::new(""));

    loop {
        let new_path = parent.join(format!("{} ({}){}", stem, n, ext));
        if !try_exists(&new_path).await.unwrap_or(false) {
            debug!("Generated unique path: {:?}", new_path);
            return new_path;
        }
        n += 1;
    }
}
