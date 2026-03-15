use {
    crate::metadata::{PortalMeta, TransferItem},
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
    tracing::{debug, error, info, warn},
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
) -> Result<()>
where
    R: AsyncRead + Unpin + Send,
{
    let mut contract: Option<PortalMeta> = None;
    let mut global_strategy = ConflictStrategy::Prompt;
    let mut items_processed: u32 = 0;

    let mut entries = archive.entries()?;
    while let Some(entry_result) = entries.next().await {
        let mut entry = entry_result.context("Failed to read tar entry")?;
        let path = entry.path()?.to_path_buf();

        // Catch metadata
        if path.to_string_lossy() == ".portal.meta" {
            debug!("Caught metadata block (.portal.meta)");
            let mut meta_bytes = Vec::new();
            tokio::io::copy(&mut entry, &mut meta_bytes)
                .await
                .context("Failed to read metadata")?;
            contract = Some(deserialize(&meta_bytes)?);
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

        if let PortalMeta::Item(item) = &meta {
            items_processed += 1;

            println!(
                "Portal: Receiving item {}/{}...",
                items_processed, total_items
            );

            match item {
                TransferItem::File(f) => {
                    println!(
                        "Portal: Receiving file '{}' ({} bytes)...",
                        f.filename, f.file_size
                    );
                    info!(
                        "Incoming top-level file: {} ({} bytes)",
                        f.filename, f.file_size
                    );
                }
                TransferItem::Directory(d) => {
                    println!(
                        "Portal: Receiving directory '{}' ({} bytes)...",
                        d.dirname, d.total_size
                    );
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
        if temp_exists {
            let _ = remove_dir_all(&temp_path).await;
        }
        create_dir_all(&temp_path).await?;

        if !is_dir {
            let file_in_temp = temp_path.join(&item_name);
            let mut outfile = File::create(&file_in_temp).await?;
            tokio::io::copy(&mut entry, &mut outfile).await?;
        }

        // move to final location
        if let Some(parent) = final_path.parent() {
            create_dir_all(parent).await?;
        }
        if !is_dir {
            if final_exists {
                let _ = remove_file(&final_path).await;
            }
            rename(temp_path.join(item_name), &final_path).await?;
            let _ = remove_dir_all(&temp_path).await;
        } else {
            if final_exists {
                let _ = remove_dir_all(&final_path).await;
            }
            rename(&temp_path, &final_path).await?;
        }

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
                    if f.file_size != entry.header().size()? {
                        error!(
                            "Size mismatch for {}: Expected {}, got {}",
                            f.filename,
                            f.file_size,
                            entry.header().size()?
                        );
                        return Err(anyhow!("Protocol error: Top-level file size mismatch"));
                    }
                    println!("Portal: File '{}' received successfully!", f.filename);
                    info!("Successfully verified and saved: {}", f.filename);
                }
                TransferItem::Directory(d) => {
                    if d.dirname != path.to_string_lossy() {
                        error!(
                            "Dirname mismatch: Expected {}, got {}",
                            d.dirname,
                            path.display()
                        );
                        return Err(anyhow!("Protocol error: Top-level directory name mismatch"));
                    }
                    println!("Portal: Directory '{}' received successfully!", d.dirname);
                    info!("Successfully verified and saved directory: {}", d.dirname);
                }
            },
            PortalMeta::NestedFile(f) => {
                debug!("Verifying nested item: {}", f.filename);
                if f.filename != path.to_string_lossy() {
                    return Err(anyhow!(
                        "Protocol error: Directory filename mismatch. Expected '{}', got '{}'",
                        f.filename,
                        path.display()
                    ));
                }
                // Compare size for integrity
                if !is_dir && f.file_size != entry.header().size()? {
                    return Err(anyhow!(
                        "Protocol error: Directory file size mismatch for '{}'",
                        f.filename
                    ));
                }
                info!("Nested item verified and saved: {}", f.filename);
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
    Ok(())
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
