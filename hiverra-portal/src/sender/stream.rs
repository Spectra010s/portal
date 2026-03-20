use {
    crate::{
        history::{HistoryItem, HistoryItemKind},
        metadata::TransferItem,
        progress::ProgressManager,
        sender::send_item::send_item,
    },
    anyhow::{Context, Result},
    async_compression::tokio::write::GzipEncoder,
    std::path::PathBuf,
    tokio::{
        io::{AsyncWrite, AsyncWriteExt},
        net::TcpStream,
    },
    tokio_tar::Builder,
    tracing::{debug, info, trace},
};

async fn stream_items<W: AsyncWrite + Unpin + Send>(
    builder: &mut Builder<W>,
    items_to_send: Vec<(PathBuf, TransferItem)>,
    prog: &ProgressManager,
    total_items: usize,
    sent_items: &mut Vec<HistoryItem>,
    actual_bytes: &mut u64,
) -> Result<()> {
    for (index, (path, item)) in items_to_send.into_iter().enumerate() {
        debug!("Processing item {}: {:?}", index + 1, path);

        // prepare per-file progress bar and pass a clone into send_item
        match item {
            TransferItem::File(fm) => {
                trace!(
                    "Progress UI: starting file item '{}' ({} bytes)",
                    fm.filename,
                    fm.file_size
                );
                prog.set_current_item(index + 1, total_items);
                let filename = fm.filename.clone();
                let file_size = fm.file_size;
                let pb = prog.create_file_bar(&filename, file_size);
                send_item(
                    builder,
                    path,
                    TransferItem::File(fm),
                    Some(pb.clone()),
                )
                .await
                .context("Failed to append item to tarball")?;
                pb.finish_and_clear();
                prog.println(format!(
                    "Portal: File '{}' sent successfully!",
                    filename
                ));
                sent_items.push(HistoryItem {
                    name: filename.clone(),
                    bytes: file_size,
                    kind: HistoryItemKind::File,
                });
                *actual_bytes = actual_bytes.saturating_add(file_size);
                trace!("Progress UI: completed file item '{}'", filename);
                trace!("History tracker: recorded sent file '{}'", filename);
            }
            TransferItem::Directory(dm) => {
                trace!(
                    "Progress UI: starting directory item '{}' ({} bytes)",
                    dm.dirname,
                    dm.total_size
                );
                prog.set_current_item(index + 1, total_items);
                let dirname = dm.dirname.clone();
                let total_size = dm.total_size;
                if dm.total_size == 0 {
                    prog.println(format!(
                        "Portal: Note: Directory '{}' is empty. Sending structure only.",
                        dirname
                    ));
                }
                let pb = prog.create_file_bar(&dirname, total_size);
                send_item(
                    builder,
                    path,
                    TransferItem::Directory(dm),
                    Some(pb.clone()),
                )
                .await
                .context("Failed to append item to tarball")?;
                pb.finish_and_clear();
                prog.println(format!(
                    "Portal: Directory '{}' sent successfully!",
                    dirname
                ));
                sent_items.push(HistoryItem {
                    name: dirname.clone(),
                    bytes: total_size,
                    kind: HistoryItemKind::Directory,
                });
                *actual_bytes = actual_bytes.saturating_add(total_size);
                trace!("Progress UI: completed directory item '{}'", dirname);
                trace!("History tracker: recorded sent directory '{}'", dirname);
            }
        }
    }
    Ok(())
}

pub async fn send_stream(
    stream: TcpStream,
    items_to_send: Vec<(PathBuf, TransferItem)>,
    prog: &ProgressManager,
    total_items: usize,
    sent_items: &mut Vec<HistoryItem>,
    actual_bytes: &mut u64,
    no_compress: bool,
) -> Result<()> {
    if no_compress {
        debug!("Initializing Tar builder (no compression)...");
        let mut builder = Builder::new(stream);
        info!("Starting TAR stream to network (no compression)...");
        stream_items(
            &mut builder,
            items_to_send,
            prog,
            total_items,
            sent_items,
            actual_bytes,
        )
        .await?;

        // finalize the Tar archive
        debug!("Finalizing Tar archive structure...");
        builder.finish().await?;

        // flush the underlying stream to ensure bytes are actually sent
        let mut stream: TcpStream = builder.into_inner().await?;
        trace!("Flushing underlying TCP stream...");
        stream.flush().await?;
        debug!("TCP stream flush complete.");
    } else {
        debug!("Initializing Gzip encoder and Tar builder...");
        let compressor = GzipEncoder::new(stream);
        let mut builder = Builder::new(compressor);

        info!("Starting TAR stream to network...");
        stream_items(
            &mut builder,
            items_to_send,
            prog,
            total_items,
            sent_items,
            actual_bytes,
        )
        .await?;

        // finalize the Tar archive
        debug!("Finalizing Tar archive structure...");
        builder.finish().await?;

        // get the compressor back
        let mut compressor: GzipEncoder<TcpStream> = builder.into_inner().await?;

        debug!("Shutting down Gzip compressor...");
        compressor
            .shutdown()
            .await
            .context("Failed to shutdown compressor")?;
        trace!("Compressor shutdown complete.");

        // flush the underlying stream to ensure bytes are actually sent
        let mut stream = compressor.into_inner();
        trace!("Flushing underlying TCP stream...");
        stream.flush().await?;
        debug!("TCP stream flush complete.");
    }

    Ok(())
}
