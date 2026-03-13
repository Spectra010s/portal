use {
    crate::metadata::TransferItem,
    anyhow::{Context, Result},
    async_walkdir::WalkDir,
    bincode::serialize,
    std::path::PathBuf,
    tokio::{fs::File, io::AsyncWrite},
    tokio_stream::StreamExt,
    tokio_tar::{Builder, EntryType, Header},
};

/// Appends a file or directory to the provided tar builder
pub async fn send_item<W>(builder: &mut Builder<W>, path: PathBuf, item: TransferItem) -> Result<()>
where
    W: AsyncWrite + Unpin + Send,
{
    match item {
        TransferItem::File(file_meta) => {
            println!(
                "Portal: Preparing to send '{}' ({} bytes)...",
                file_meta.filename, file_meta.file_size
            );

            // Send the pair: metadata and file data
            let meta_item = TransferItem::File(file_meta.clone());
            let meta_bytes = serialize(&meta_item)?;
            append_raw_meta(builder, meta_bytes).await?;

            let mut file = File::open(&path).await?;
            let mut header = Header::new_gnu();
            header.set_path(&file_meta.filename)?;
            header.set_size(file_meta.file_size);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append(&header, &mut file).await?;

            println!("Portal: File '{}' sent successfully!", file_meta.filename);
        }

        TransferItem::Directory(dir_meta) => {
            // tell user they are sending empyt dir if empty
            if dir_meta.total_size == 0 {
                println!(
                    "Portal: Note: Directory '{}' is empty. Sending structure only.",
                    dir_meta.dirname
                );
            } else {
                println!(
                    "Portal: Preparing to send directory '{}' ({} bytes)...",
                    &dir_meta.dirname, dir_meta.total_size
                );
            }

            // send the directory metadata
            let meta_bytes = serialize(&TransferItem::Directory(dir_meta.clone()))?;
            append_raw_meta(builder, meta_bytes).await?;

            // add the directory itself to the tar and ensure empty folders are created
            let mut dir_header = Header::new_gnu();
            dir_header.set_path(&dir_meta.dirname)?;
            dir_header.set_entry_type(EntryType::Directory);
            dir_header.set_mode(0o755);
            dir_header.set_size(0);
            dir_header.set_cksum();
            builder.append(&dir_header, &[][..]).await?;

            // stream the contents of the directory
            let mut entries = WalkDir::new(&path);

            while let Some(entry) = entries.next().await {
                let entry = entry.context("Portal: Failed to read directory entry")?;
                let file_type = entry.file_type().await?;
                let local_path = entry.path();
                let rel_path = local_path.strip_prefix(&path)?;
                let tar_path = format!("{}/{}", dir_meta.dirname, rel_path.display());

                if file_type.is_file() {
                    let mut file = File::open(&local_path).await?;
                    let mut header = Header::new_gnu();
                    header.set_path(&tar_path)?;
                    header.set_size(file.metadata().await?.len());
                    header.set_mode(0o644);
                    header.set_cksum();

                    builder.append(&header, &mut file).await?;

                    println!("Portal: Directory file '{}' sent succesfully!", &tar_path);
                } else if file_type.is_dir() {
                    // Send subdirectory headers to maintain tree structure
                    let mut header = Header::new_gnu();
                    header.set_path(&tar_path)?;
                    header.set_entry_type(EntryType::Directory);
                    header.set_mode(0o755);
                    header.set_size(0);
                    header.set_cksum();
                    builder.append(&header, &[][..]).await?;
                }
            }
            println!("Portal: Directory '{}' sent succesfully!", dir_meta.dirname);
        }
    }

    Ok(())
}

/// Helper to write the bincode metadata as a hidden virtual file in the tar stream
async fn append_raw_meta<W: AsyncWrite + Unpin + Send>(
    builder: &mut Builder<W>,
    bytes: Vec<u8>,
) -> Result<()> {
    let mut header = Header::new_gnu();
    header.set_path(".portal.item.meta")?;
    header.set_size(bytes.len() as u64);
    header.set_cksum();
    builder.append(&header, &bytes[..]).await?;
    Ok(())
}
