use {
    crate::{
        metadata::TransferItem,
        sender::send_item::send_item,
        TransferProgress,
    },
    crate::error::{PxpError, Result},
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
    progress: Option<&dyn TransferProgress>,
) -> Result<()> {
    let total = items_to_send.len();
    for (index, (path, item)) in items_to_send.into_iter().enumerate() {
        debug!("Processing item {}: {:?}", index + 1, path);

        if let Some(prog) = progress {
            let (name, bytes, is_dir) = match &item {
                TransferItem::File(fm) => (fm.filename.clone(), fm.file_size, false),
                TransferItem::Directory(dm) => (dm.dirname.clone(), dm.total_size, true),
            };

            prog.set_current_item(index + 1, total);

            if is_dir && bytes == 0 {
                prog.println(&format!(
                    "Portal: Note: Directory '{}' is empty. Sending structure only.",
                    name
                ));
            }

            let item_prog = prog.create_item_progress(&name, bytes);
            send_item(builder, path, item, Some(&*item_prog))
                .await
                .map_err(|e| PxpError::Archive(e.to_string()))?;
            item_prog.finish_and_clear();

            let kind = if is_dir { "Directory" } else { "File" };
            prog.println(&format!("Portal: {} '{}' sent successfully!", kind, name));
        } else {
            send_item(builder, path, item, None)
                .await
                .map_err(|e| PxpError::Archive(e.to_string()))?;
        }
    }
    Ok(())
}

pub async fn send_stream(
    stream: TcpStream,
    items_to_send: Vec<(PathBuf, TransferItem)>,
    no_compress: bool,
    progress: Option<&dyn TransferProgress>,
) -> Result<()> {
    if no_compress {
        debug!("Initializing Tar builder (no compression)...");
        let mut builder = Builder::new(stream);
        info!("Starting TAR stream to network (no compression)...");
        stream_items(&mut builder, items_to_send, progress).await?;

        debug!("Finalizing Tar archive structure...");
        builder.finish().await?;

        let mut stream: TcpStream = builder.into_inner().await?;
        trace!("Flushing underlying TCP stream...");
        stream.flush().await?;
        debug!("TCP stream flush complete.");
    } else {
        debug!("Initializing Gzip encoder and Tar builder...");
        let compressor = GzipEncoder::new(stream);
        let mut builder = Builder::new(compressor);

        info!("Starting TAR stream to network...");
        stream_items(&mut builder, items_to_send, progress).await?;

        debug!("Finalizing Tar archive structure...");
        builder.finish().await?;

        let mut compressor: GzipEncoder<TcpStream> = builder.into_inner().await?;

        debug!("Shutting down Gzip compressor...");
        compressor
            .shutdown()
            .await?;
        trace!("Compressor shutdown complete.");

        let mut stream = compressor.into_inner();
        trace!("Flushing underlying TCP stream...");
        stream.flush().await?;
        debug!("TCP stream flush complete.");
    }

    Ok(())
}
