use {
    crate::{
        metadata::{TransferItem, TransferResult},
        sender::send_item::send_item,
        TransferProgress,
    },
    crate::error::{PxpError, Result},
    async_compression::tokio::write::GzipEncoder,
    bincode,
    std::path::PathBuf,
    tokio::{
        io::{AsyncReadExt, AsyncWrite, AsyncWriteExt},
        net::TcpStream,
    },
    tokio_tar::Builder,
    tracing::{debug, info, trace, warn},
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
) -> Result<TransferResult> {
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
        debug!("TCP stream flushed. Waiting for receiver ack...");
        read_transfer_result(&mut stream).await
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
        compressor.shutdown().await?;
        trace!("Compressor shutdown complete.");

        let mut stream = compressor.into_inner();
        trace!("Flushing underlying TCP stream...");
        stream.flush().await?;
        debug!("TCP stream flushed. Waiting for receiver ack...");
        read_transfer_result(&mut stream).await
    }
}

/// Read the [`TransferResult`] acknowledgment frame the receiver sends after it
/// has finished reconciling all staged items into the target directory.
///
/// Frame layout:
/// ```text
/// [ length: u32 big-endian ][ bincode-encoded TransferResult ]
/// ```
///
/// If the receiver closes the connection without sending an ack (e.g. because
/// it is an older build that does not implement this frame), we synthesise a
/// best-effort result rather than failing the transfer. This keeps the sender
/// backward-compatible with receivers that pre-date this change.
async fn read_transfer_result(stream: &mut TcpStream) -> Result<TransferResult> {
    // Read the 4-byte length prefix.
    let mut len_buf = [0u8; 4];
    match stream.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof
            || e.kind() == std::io::ErrorKind::ConnectionReset
            || e.kind() == std::io::ErrorKind::BrokenPipe =>
        {
            // Receiver closed without acking — treat as implicit success for
            // backward compatibility, but warn so it shows up in logs.
            warn!(
                "Receiver closed connection without sending a result ack (older build?). \
                 Assuming success."
            );
            return Ok(TransferResult {
                success: true,
                items_received: 0,
                bytes_received: 0,
                error: None,
            });
        }
        Err(e) => return Err(PxpError::Io(e)),
    }

    let payload_len = u32::from_be_bytes(len_buf) as usize;
    if payload_len > 4096 {
        return Err(PxpError::Protocol(format!(
            "Transfer result frame is implausibly large ({} bytes).",
            payload_len
        )));
    }

    let mut payload = vec![0u8; payload_len];
    stream.read_exact(&mut payload).await?;

    let result: TransferResult = bincode::deserialize(&payload)?;
    debug!(
        "Received transfer result ack: success={}, items={}, bytes={}",
        result.success, result.items_received, result.bytes_received
    );

    if result.success {
        info!("Receiver confirmed all items landed successfully.");
    } else {
        warn!(
            "Receiver reported a failure: {}",
            result.error.as_deref().unwrap_or("no details")
        );
    }

    Ok(result)
}
