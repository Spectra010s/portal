use {
    crate::{
        metadata::{PxpMeta, ReceiveSummary, ReceivedItem, TransferItem},
        ConflictAction, ConflictResolver, TransferProgress,
    },
    crate::error::{PxpError, Result},
    bincode::deserialize,
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

/// Receives items from the tar archive, validates metadata, and writes to disk.
pub async fn receive_item<R>(
    archive: &mut Archive<R>,
    target_dir: &PathBuf,
    total_items: u32,
    progress: Option<&dyn TransferProgress>,
    conflict_resolver: Option<&dyn ConflictResolver>,
    summary: &mut ReceiveSummary,
) -> Result<()>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    let mut contract: Option<PxpMeta> = None;
    let mut global_strategy = ConflictStrategy::Prompt;
    let mut items_processed: u32 = 0;
    let mut active_dir_progress: Option<Box<dyn crate::ItemProgress>> = None;
    let mut pending_dir_success: Option<String> = None;
    let mut entries = archive.entries()?;
    while let Some(entry_result) = entries.next().await {
        let mut entry = entry_result.map_err(|e| PxpError::Archive(e.to_string()))?;
        let path = entry.path()?.to_path_buf();
        let entry_size = entry.header().size()?;
        trace!("--- Processing archive entry {} ---", path.display());

        // Catch metadata
        if path.to_string_lossy().replace('\\', "/") == ".portal.meta" {
            debug!("Caught metadata block (.portal.meta)");
            let mut meta_bytes = Vec::new();
            tokio::io::copy(&mut entry, &mut meta_bytes)
                .await?;
            let deserialized: PxpMeta = deserialize(&meta_bytes)?;
            trace!("Deserialized metadata content: {:?}", deserialized);
            contract = Some(deserialized);
            continue;
        }

        let meta = contract.take().ok_or_else(|| {
            error!(
                "Protocol error: {} arrived without preceding metadata",
                path.display()
            );
            PxpError::Protocol(format!("data entry '{}' arrived without metadata", path.display()))
        })?;
        trace!(
            "Matched entry '{}' with its metadata contract.",
            path.display()
        );

        let mut entry_item_progress: Option<Box<dyn crate::ItemProgress>> = None;
        if let PxpMeta::Item(item) = &meta {
            items_processed += 1;

            if let Some(prog) = &progress {
                prog.set_current_item(items_processed as usize, total_items as usize);
            }

            match item {
                TransferItem::File(f) => {
                    trace!(
                        "Starting file item '{}' ({} bytes)",
                        f.filename, f.file_size
                    );
                    summary.items.push(ReceivedItem {
                        name: f.filename.clone(),
                        bytes: f.file_size,
                        is_directory: false,
                    });
                    summary.total_bytes = summary.total_bytes.saturating_add(f.file_size);
                    info!(
                        "Incoming top-level file: {} ({} bytes)",
                        f.filename, f.file_size
                    );
                }
                TransferItem::Directory(d) => {
                    trace!(
                        "Starting directory item '{}' ({} bytes)",
                        d.dirname, d.total_size
                    );
                    summary.items.push(ReceivedItem {
                        name: d.dirname.clone(),
                        bytes: d.total_size,
                        is_directory: true,
                    });
                    summary.total_bytes = summary.total_bytes.saturating_add(d.total_size);
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
                return Err(PxpError::Security(
                    "Sender sent more items than manifest allowed".into()
                ));
            }

            // Close any active directory progress before starting a new top-level item
            if let Some(dir_prog) = active_dir_progress.take() {
                dir_prog.finish_and_clear();
                if let Some(dir_name) = pending_dir_success.take() {
                    if let Some(prog) = &progress {
                        prog.println(&format!(
                            "Portal: Directory '{}' received successfully!",
                            dir_name
                        ));
                    }
                }
            }

            if let Some(prog) = &progress {
                match item {
                    TransferItem::File(f) => {
                        entry_item_progress = Some(prog.create_item_progress(&f.filename, f.file_size));
                    }
                    TransferItem::Directory(d) => {
                        active_dir_progress = Some(prog.create_item_progress(&d.dirname, d.total_size));
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
            } else if let Some(resolver) = conflict_resolver {
                let action = resolver.resolve(&item_name)?;
                trace!("Conflict resolver returned: {:?}", action);

                match action {
                    ConflictAction::Overwrite => info!("Chose to overwrite {:?}", item_name),
                    ConflictAction::OverwriteAll => {
                        info!("Enabled Overwrite All strategy");
                        global_strategy = ConflictStrategy::OverwriteAll;
                    }
                    ConflictAction::Rename => {
                        final_path = get_unused_path(final_path).await;
                        info!("Chose to rename to {:?}", final_path);
                    }
                    ConflictAction::RenameAll => {
                        info!("Enabled Rename All strategy");
                        global_strategy = ConflictStrategy::RenameAll;
                        final_path = get_unused_path(final_path).await;
                    }
                    ConflictAction::Skip => {
                        info!("Skipped item {:?}", item_name);
                        continue;
                    }
                    ConflictAction::SkipAll => {
                        info!("Enabled Skip All strategy");
                        global_strategy = ConflictStrategy::SkipAll;
                        continue;
                    }
                }
            } else {
                // No conflict resolver provided, default to overwrite
                info!("No conflict resolver: defaulting to overwrite {:?}", item_name);
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
            let outfile = File::create(&file_in_temp).await?;

            if let Some(prog) = entry_item_progress.take() {
                let mut writer = prog.wrap_write(Box::new(outfile));
                tokio::io::copy(&mut entry, &mut *writer).await?;
                prog.finish_and_clear();
            } else if let Some(prog) = &active_dir_progress {
                let mut reader = prog.wrap_read(Box::new(entry));
                let mut outfile = outfile;
                tokio::io::copy(&mut *reader, &mut outfile).await?;
            } else {
                let mut outfile = outfile;
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
            PxpMeta::Item(item) => match item {
                TransferItem::File(f) => {
                    if f.filename != path.to_string_lossy() {
                        error!(
                            "Filename mismatch: Expected {}, got {}",
                            f.filename,
                            path.display()
                        );
                        return Err(PxpError::Protocol("Top-level filename mismatch".into()));
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
                        return Err(PxpError::Protocol("Top-level file size mismatch".into()));
                    }
                    trace!(
                        "Self-check: file size matches manifest ({} bytes)",
                        f.file_size
                    );
                    if let Some(prog) = &progress {
                        prog.println(&format!(
                            "Portal: File '{}' received successfully!",
                            f.filename
                        ));
                    }
                    info!("Successfully verified and saved: {}", f.filename);
                }
                TransferItem::Directory(d) => {
                    if d.dirname != path.to_string_lossy().replace('\\', "/") {
                        error!(
                            "Dirname mismatch: Expected {}, got {}",
                            d.dirname,
                            path.display()
                        );
                        return Err(PxpError::Protocol("Top-level directory name mismatch".into()));
                    }
                    info!("Successfully verified and saved directory: {}", d.dirname);
                }
            },
            PxpMeta::NestedFile(f) => {
                debug!("Verifying nested item: {}", f.filename);
                if f.filename != path.to_string_lossy().replace('\\', "/") {
                    return Err(PxpError::Protocol(format!(
                        "Directory filename mismatch. Expected '{}', got '{}'",
                        f.filename,
                        path.display()
                    )));
                }
                if !is_dir && f.file_size != entry_size {
                    trace!(
                        "Nested file verification failure: {} (manifest: {}, header: {})",
                        f.filename, f.file_size, entry_size
                    );
                    return Err(PxpError::Protocol(format!(
                        "Directory file size mismatch for '{}'",
                        f.filename
                    )));
                }
                trace!("Nested item size verified: {} bytes", f.file_size);
                info!("Nested item verified and saved: {}", f.filename);
            }
        }
    }
    if let Some(dir_prog) = active_dir_progress.take() {
        dir_prog.finish_and_clear();
        if let Some(dir_name) = pending_dir_success.take() {
            if let Some(prog) = &progress {
                prog.println(&format!(
                    "Portal: Directory '{}' received successfully!",
                    dir_name
                ));
            }
        }
    }
    if items_processed != total_items {
        error!(
            "Transfer failed: manifest expected {} items, only received {}",
            total_items, items_processed
        );
        return Err(PxpError::Protocol(format!(
            "Transfer incomplete: Expected {} items, only got {}",
            total_items,
            items_processed
        )));
    }
    info!("All {} items received and verified.", items_processed);
    Ok(())
}

/// helper to get path for incremental renaming
async fn get_unused_path(path: PathBuf) -> PathBuf {
    let mut n = 1;
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
