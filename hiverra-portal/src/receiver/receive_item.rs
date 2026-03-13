use {
    crate::metadata::TransferItem,
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
};

#[derive(Clone, Copy, PartialEq)]
enum ConflictStrategy {
    Prompt,
    OverwriteAll,
    RenameAll,
    SkipAll,
}

pub async fn receive_item<R>(
    archive: &mut Archive<R>,
    target_dir: &PathBuf,
    total_items: u32,
) -> Result<()>
where
    R: AsyncRead + Unpin + Send,
{
    // make contract: an Option that stays until overwritten
    let mut last_contract: Option<TransferItem> = None;
    let mut global_strategy = ConflictStrategy::Prompt;

    let mut received_count = 0;
    println!("Portal: Receiving items...");

    let mut entries = archive.entries()?;

    while let Some(entry_result) = entries.next().await {
        let mut entry = entry_result.context("Failed to read tar entry")?;
        let path = entry.path()?.to_path_buf();

        // Catch metadata
        if path.to_string_lossy() == ".portal.item.meta" {
            received_count += 1;
            println!("Portal: Receiving item {}/{}", received_count, total_items);

            let mut meta_bytes = Vec::new();
            tokio::io::copy(&mut entry, &mut meta_bytes)
                .await
                .context("Failed to read metadata contract")?;

            last_contract = Some(deserialize(&meta_bytes)?);
            continue;
        }

        // keep the dir contract alive for the whole subtree
        let contract = last_contract.as_ref().ok_or_else(|| {
            anyhow!(
                "Protocol error: data entry '{}' arrived without metadata",
                path.display()
            )
        })?;

        let is_dir = entry.header().entry_type().is_dir();
        let item_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let temp_path = target_dir.join(format!(".tmp_{}_portal", item_name));
        let mut final_path = target_dir.join(&path);

        // pre-check existence once for efficiency
        let final_exists = try_exists(&final_path).await?;
        let temp_exists = try_exists(&temp_path).await?;

        //  handle conflict
        if final_exists && global_strategy != ConflictStrategy::OverwriteAll {
            if global_strategy == ConflictStrategy::SkipAll {
                continue;
            }

            if global_strategy == ConflictStrategy::RenameAll {
                final_path = get_unused_path(final_path);
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
                    "Overwrite" => (),
                    "Overwrite All" => global_strategy = ConflictStrategy::OverwriteAll,
                    "Rename" => final_path = get_unused_path(final_path),
                    "Rename All" => {
                        global_strategy = ConflictStrategy::RenameAll;
                        final_path = get_unused_path(final_path);
                    }
                    "Skip" => continue,
                    "Skip All" => {
                        global_strategy = ConflictStrategy::SkipAll;
                        continue;
                    }
                    _ => unreachable!(),
                }
            }
        }

        // prepare temporary folder
        if temp_exists {
            let _ = remove_dir_all(&temp_path).await;
        }
        create_dir_all(&temp_path).await?;

        if !is_dir {
            let file_in_temp = temp_path.join(item_name);
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

        // print status
        match contract {
            TransferItem::File(f) => {
                println!("Portal: File '{}' received successfully!", f.filename);
                last_contract = None;
            }
            TransferItem::Directory(d) => {
                if path.to_string_lossy() == d.dirname {
                    println!("Portal: Directory '{}' received successfully!", d.dirname);
                }
            }
        }
    }

    Ok(())
}

/// helper to get path for incrmeental renaming
fn get_unused_path(path: PathBuf) -> PathBuf {
    let mut n = 1;
    let stem = path.file_stem().unwrap().to_string_lossy();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let parent = path.parent().unwrap();
    loop {
        let new_path = parent.join(format!("{} ({}){}", stem, n, ext));
        if !new_path.exists() {
            return new_path;
        }
        n += 1;
    }
}
