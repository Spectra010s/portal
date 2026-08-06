use {
    crate::metadata::{FileMetadata, PxpMeta, TransferItem},
    crate::sender::manifest::create_file_metadata,
    crate::ItemProgress,
    crate::error::{PxpError, Result},
    async_walkdir::WalkDir,
    bincode::serialize,
    std::path::PathBuf,
    tokio::{fs::File, io::AsyncWrite},
    tokio_stream::StreamExt,
    tokio_tar::{Builder, EntryType, Header},
    tracing::{debug, info, trace, warn},
};

/// Appends a file or directory to the provided tar builder
pub async fn send_item<W>(
    builder: &mut Builder<W>,
    path: PathBuf,
    item: TransferItem,
    item_progress: Option<&dyn ItemProgress>,
) -> Result<()>
where
    W: AsyncWrite + Unpin + Send,
{
    match item {
        TransferItem::File(file_meta) => {
            trace!(
                "Streaming file payload '{}' ({} bytes)",
                file_meta.filename, file_meta.file_size
            );
            debug!("Serializing metadata for file: {}", file_meta.filename);
            let meta_bytes = serialize(&PxpMeta::Item(TransferItem::File(file_meta.clone())))?;
            trace!("Serialized file metadata size: {} bytes", meta_bytes.len());
            append_raw_meta(builder, meta_bytes).await?;

            trace!("Opening file for reading: {:?}", path);
            let file = File::open(&path).await?;
            let mut header = Header::new_gnu();
            header.set_path(&file_meta.filename)?;
            header.set_size(file_meta.file_size);
            header.set_mode(0o644);
            header.set_cksum();

            trace!("Appending file '{}' to tar archive", file_meta.filename);
            // We use the ItemProgress wrapper to wrap the file reader before handing it off to the tar builder.
            // As the tar builder pulls bytes from the stream, our wrapper intercepts those reads 
            // to dynamically update the UI progress bar. This way we don't have to manually chunk the file ourselves.
            if let Some(prog) = item_progress {
                let mut reader = prog.wrap_read(Box::new(file));
                builder.append(&header, &mut *reader).await?;
            } else {
                let mut f = file;
                builder.append(&header, &mut f).await?;
            }

            info!(
                "File '{}' transfer initiated and appended to stream.",
                file_meta.filename
            );
        }

        TransferItem::Directory(dir_meta) => {
            trace!(
                "Streaming directory payload '{}' ({} bytes)",
                dir_meta.dirname, dir_meta.total_size
            );
            if dir_meta.total_size == 0 {
                warn!(
                    "Directory '{}' is empty; sending structure only.",
                    dir_meta.dirname
                );
            }

            debug!("Serializing metadata for directory: {}", dir_meta.dirname);
            let meta_bytes =
                serialize(&PxpMeta::Item(TransferItem::Directory(dir_meta.clone())))?;
            trace!(
                "Serialized directory metadata size: {} bytes",
                meta_bytes.len()
            );
            append_raw_meta(builder, meta_bytes).await?;

            trace!(
                "Appending directory node '{}' to tar archive",
                dir_meta.dirname
            );
            let mut dir_header = Header::new_gnu();
            dir_header.set_path(&dir_meta.dirname)?;
            dir_header.set_entry_type(EntryType::Directory);
            dir_header.set_mode(0o755);
            dir_header.set_size(0);
            dir_header.set_cksum();
            builder.append(&dir_header, &[][..]).await?;

            debug!("Starting WalkDir for directory: {:?}", path);
            // We need to flatten the recursive directory structure into a linear series of tar entries.
            // WalkDir iterates through everything under the path, and for each entry, we strip the 
            // base path to figure out its relative tar path. This makes sure nested files end up 
            // in the correct folder structure on the receiver's end.
            let mut entries = WalkDir::new(&path);
            while let Some(entry) = entries.next().await {
                let entry = entry.map_err(|e| PxpError::WalkDir(e.to_string()))?;
                let file_type = entry.file_type().await?;
                let local_path = entry.path();
                let rel_path = local_path.strip_prefix(&path)?;
                let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");
                let tar_path = format!("{}/{}", dir_meta.dirname, rel_path_str);

                trace!(
                    "Processing entry: {:?} -> tar_path: {}",
                    local_path, tar_path
                );

                if file_type.is_file() {
                    debug!("Processing nested file: {}", tar_path);
                    let mut file_meta = create_file_metadata(&local_path).await?;
                    file_meta.filename = tar_path.clone();

                    trace!("Serializing nested file metadata for: {}", tar_path);
                    let meta_bytes = serialize(&PxpMeta::NestedFile(file_meta.clone()))?;
                    trace!("Nested file metadata size: {} bytes", meta_bytes.len());
                    append_raw_meta(builder, meta_bytes).await?;

                    trace!("Opening nested file: {:?}", local_path);
                    let file = File::open(&local_path).await?;
                    let mut header = Header::new_gnu();
                    header.set_path(&tar_path)?;
                    header.set_size(file.metadata().await?.len());
                    header.set_mode(0o644);
                    header.set_cksum();

                    trace!("Appending nested file '{}' to tar archive", tar_path);
                    if let Some(prog) = item_progress {
                        let mut reader = prog.wrap_read(Box::new(file));
                        builder.append(&header, &mut *reader).await?;
                    } else {
                        let mut f = file;
                        builder.append(&header, &mut f).await?;
                    }

                    info!("Directory file sent successfully: {}", &tar_path);
                } else if file_type.is_dir() {
                    debug!("Processing nested directory: {}", tar_path);
                    let sub_dir_meta = FileMetadata {
                        filename: tar_path.clone(),
                        file_size: 0,
                    };

                    trace!("Serializing nested directory metadata for: {}", tar_path);
                    let meta_bytes = serialize(&PxpMeta::NestedFile(sub_dir_meta))?;
                    trace!("Nested directory metadata size: {} bytes", meta_bytes.len());
                    append_raw_meta(builder, meta_bytes).await?;

                    trace!("Appending subdirectory entry to tar: {}", tar_path);
                    let mut header = Header::new_gnu();
                    header.set_path(&tar_path)?;
                    header.set_entry_type(EntryType::Directory);
                    header.set_mode(0o755);
                    header.set_size(0);
                    header.set_cksum();
                    builder.append(&header, &[][..]).await?;
                }
            }

            info!("Directory '{}' transfer complete.", dir_meta.dirname);
        }
    }

    Ok(())
}

// We inject a virtual `.portal.meta` file right before the actual data in the TAR stream.
// This establishes a "contract" so the receiver knows exactly what to expect next 
// (e.g., file size, original path). We do this because raw tar headers don't have enough 
// space/flexibility for our custom metadata, and this keeps the stream self-describing.
/// Helper to write the bincode metadata as a hidden virtual file in the tar stream
async fn append_raw_meta<W: AsyncWrite + Unpin + Send>(
    builder: &mut Builder<W>,
    bytes: Vec<u8>,
) -> Result<()> {
    debug!(
        "Appending metadata header (.portal.meta) - size: {} bytes",
        bytes.len()
    );
    trace!("Metadata payload content: {:?}", bytes);
    let mut header = Header::new_gnu();
    header.set_path(".portal.meta")?;
    header.set_size(bytes.len() as u64);
    header.set_cksum();
    builder.append(&header, &bytes[..]).await?;
    Ok(())
}
